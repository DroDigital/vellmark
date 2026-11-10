//! Per-pass GPU timers. Each pass asks for a slot when it begins; the results of a
//! frame arrive a few frames later, without ever waiting for the GPU.

use crate::Gpu;
use std::sync::mpsc::{channel, Receiver, Sender};

const MAX_PASSES: u32 = 32;

#[derive(Clone, Debug, PartialEq)]
pub struct PassTiming {
    pub name: &'static str,
    pub milliseconds: f32,
}

struct Readback {
    buffer: wgpu::Buffer,
    names: Vec<&'static str>,
}

pub struct PassTimers {
    query_set: Option<wgpu::QuerySet>,
    resolve: Option<wgpu::Buffer>,
    period_ns: f32,
    names: Vec<&'static str>,
    free: Vec<wgpu::Buffer>,
    in_flight: Vec<Readback>,
    mapped_tx: Sender<usize>,
    mapped_rx: Receiver<usize>,
    next_ticket: usize,
    tickets: Vec<(usize, Readback)>,
    latest: Vec<PassTiming>,
}

impl PassTimers {
    /// `enabled` is true in development builds and during the performance run. When it
    /// is false, or the GPU cannot time passes, every call is free and reports nothing.
    pub fn new(gpu: &Gpu, enabled: bool) -> Self {
        let (mapped_tx, mapped_rx) = channel();
        let active = enabled && gpu.timestamps;
        let query_set = active.then(|| {
            gpu.device.create_query_set(&wgpu::QuerySetDescriptor {
                label: Some("pass timers"),
                ty: wgpu::QueryType::Timestamp,
                count: MAX_PASSES * 2,
            })
        });
        let resolve = active.then(|| {
            gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("pass timers resolve"),
                size: (MAX_PASSES * 2 * 8) as u64,
                usage: wgpu::BufferUsages::QUERY_RESOLVE | wgpu::BufferUsages::COPY_SRC,
                mapped_at_creation: false,
            })
        });
        Self {
            query_set,
            resolve,
            period_ns: gpu.queue.get_timestamp_period(),
            names: Vec::new(),
            free: Vec::new(),
            in_flight: Vec::new(),
            mapped_tx,
            mapped_rx,
            next_ticket: 0,
            tickets: Vec::new(),
            latest: Vec::new(),
        }
    }

    pub fn active(&self) -> bool {
        self.query_set.is_some()
    }

    pub fn begin_frame(&mut self) {
        self.names.clear();
    }

    fn slot(&mut self, name: &'static str) -> Option<(&wgpu::QuerySet, u32)> {
        let set = self.query_set.as_ref()?;
        if self.names.len() as u32 >= MAX_PASSES {
            return None;
        }
        self.names.push(name);
        Some((set, (self.names.len() as u32 - 1) * 2))
    }

    /// The timestamp writes for a render pass named `name`.
    pub fn render_pass(&mut self, name: &'static str) -> Option<wgpu::RenderPassTimestampWrites<'_>> {
        self.slot(name).map(|(query_set, first)| wgpu::RenderPassTimestampWrites {
            query_set,
            beginning_of_pass_write_index: Some(first),
            end_of_pass_write_index: Some(first + 1),
        })
    }

    /// The timestamp writes for a compute pass named `name`.
    pub fn compute_pass(&mut self, name: &'static str) -> Option<wgpu::ComputePassTimestampWrites<'_>> {
        self.slot(name).map(|(query_set, first)| wgpu::ComputePassTimestampWrites {
            query_set,
            beginning_of_pass_write_index: Some(first),
            end_of_pass_write_index: Some(first + 1),
        })
    }

    /// Call after the last pass, before the encoder is finished.
    pub fn end_frame(&mut self, gpu: &Gpu, encoder: &mut wgpu::CommandEncoder) {
        let (Some(set), Some(resolve)) = (&self.query_set, &self.resolve) else { return };
        if self.names.is_empty() {
            return;
        }
        let count = self.names.len() as u32 * 2;
        let bytes = count as u64 * 8;
        encoder.resolve_query_set(set, 0..count, resolve, 0);
        let buffer = self.free.pop().unwrap_or_else(|| {
            gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("pass timers readback"),
                size: (MAX_PASSES * 2 * 8) as u64,
                usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
                mapped_at_creation: false,
            })
        });
        encoder.copy_buffer_to_buffer(resolve, 0, &buffer, 0, bytes);
        self.in_flight.push(Readback { buffer, names: self.names.clone() });
    }

    /// Call after the frame's commands are submitted.
    pub fn after_submit(&mut self) {
        for readback in self.in_flight.drain(..) {
            let ticket = self.next_ticket;
            self.next_ticket += 1;
            let tx = self.mapped_tx.clone();
            let bytes = readback.names.len() as u64 * 16;
            readback.buffer.slice(..bytes).map_async(wgpu::MapMode::Read, move |result| {
                if result.is_ok() {
                    let _ = tx.send(ticket);
                }
            });
            self.tickets.push((ticket, readback));
        }
    }

    /// Collects any results that have arrived. Never waits. Call once a frame, after the
    /// device has been polled.
    pub fn collect(&mut self) {
        while let Ok(ticket) = self.mapped_rx.try_recv() {
            let Some(at) = self.tickets.iter().position(|(t, _)| *t == ticket) else { continue };
            let (_, readback) = self.tickets.remove(at);
            let bytes = readback.names.len() as u64 * 16;
            {
                let view = readback.buffer.slice(..bytes).get_mapped_range().expect("timer range");
                let stamps: Vec<u64> =
                    view.chunks_exact(8).map(|c| u64::from_le_bytes(c.try_into().unwrap())).collect();
                self.latest = readback
                    .names
                    .iter()
                    .enumerate()
                    .map(|(i, &name)| PassTiming {
                        name,
                        milliseconds: stamps[i * 2 + 1].saturating_sub(stamps[i * 2]) as f32 * self.period_ns / 1.0e6,
                    })
                    .collect();
            }
            readback.buffer.unmap();
            self.free.push(readback.buffer);
        }
    }

    /// The most recent complete set of timings, in the order the passes ran.
    pub fn latest(&self) -> &[PassTiming] {
        &self.latest
    }
}
