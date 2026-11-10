//! Every pipeline is made through this registry, during startup or loading. Creating
//! one during play is counted, logged, and fails the stutter test.

use std::sync::atomic::{AtomicU32, Ordering};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PipelinePhase {
    /// Startup or a loading screen. Pipelines may be made.
    Loading,
    /// The player is in the world. Pipelines may not be made.
    Play,
}

pub struct PipelineRegistry {
    phase: PipelinePhase,
    created: u32,
    created_during_play: AtomicU32,
    names_during_play: std::sync::Mutex<Vec<String>>,
}

impl Default for PipelineRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl PipelineRegistry {
    pub fn new() -> Self {
        Self {
            phase: PipelinePhase::Loading,
            created: 0,
            created_during_play: AtomicU32::new(0),
            names_during_play: Default::default(),
        }
    }

    pub fn phase(&self) -> PipelinePhase {
        self.phase
    }

    /// Called when loading ends. From here on, making a pipeline is a fault.
    pub fn seal(&mut self) {
        self.phase = PipelinePhase::Play;
    }

    /// Called when a loading screen begins, or when a development build reloads a
    /// shader from disk. Shipping builds never reload.
    pub fn unseal(&mut self) {
        self.phase = PipelinePhase::Loading;
    }

    fn note(&mut self, label: Option<&str>) {
        self.created += 1;
        if self.phase == PipelinePhase::Play {
            self.created_during_play.fetch_add(1, Ordering::Relaxed);
            let name = label.unwrap_or("unnamed").to_owned();
            eprintln!("fault: pipeline `{name}` was created during play");
            self.names_during_play.lock().unwrap().push(name);
        }
    }

    pub fn render_pipeline(
        &mut self,
        device: &wgpu::Device,
        descriptor: &wgpu::RenderPipelineDescriptor<'_>,
    ) -> wgpu::RenderPipeline {
        self.note(descriptor.label);
        device.create_render_pipeline(descriptor)
    }

    pub fn compute_pipeline(
        &mut self,
        device: &wgpu::Device,
        descriptor: &wgpu::ComputePipelineDescriptor<'_>,
    ) -> wgpu::ComputePipeline {
        self.note(descriptor.label);
        device.create_compute_pipeline(descriptor)
    }

    pub fn created(&self) -> u32 {
        self.created
    }

    /// The number the performance report and the stutter test read. It must be zero.
    pub fn created_during_play(&self) -> u32 {
        self.created_during_play.load(Ordering::Relaxed)
    }

    pub fn names_created_during_play(&self) -> Vec<String> {
        self.names_during_play.lock().unwrap().clone()
    }
}
