//! Vellmark's sound. The mixer runs on the audio thread: it never allocates, never locks,
//! and reads what the game wants through atomics. The game thread writes them each frame.
//!
//! The Showcase's ambience is synthesized: wind in gusts, leaves near trees, rain, the
//! river, and birdsong by day. Recorded layers join it when the approved sample packs
//! are in `third_party/inbox/` (see `third_party/DOWNLOADS.md`).

pub mod ambience;
pub mod dsp;

use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

pub use ambience::Ambience;

/// The output's sample rate. Every voice is made for it.
pub const SAMPLE_RATE: u32 = 48_000;
/// Interleaved stereo.
pub const CHANNELS: usize = 2;

/// A float the game thread writes and the audio thread reads, without a lock.
#[derive(Default)]
pub struct Param(AtomicU32);

impl Param {
    pub fn set(&self, value: f32) {
        self.0.store(value.to_bits(), Ordering::Relaxed);
    }

    pub fn get(&self) -> f32 {
        f32::from_bits(self.0.load(Ordering::Relaxed))
    }
}

/// What the ambience should be now, from the world around the listener. Every value is
/// 0 to 1 unless it says otherwise.
#[derive(Default)]
pub struct Controls {
    /// The whole mix's loudness.
    pub master: Param,
    /// The wind's strength.
    pub wind: Param,
    /// How hard it rains.
    pub rain: Param,
    /// How near and loud running water is.
    pub water: Param,
    /// How many trees and how near: their leaves move in the wind.
    pub trees: Param,
    /// Daylight, for the birds: 1 by day, 0 at night.
    pub daylight: Param,
    /// How open the place is to birds: 0 indoors or underground.
    pub open: Param,
}

impl Controls {
    pub fn new() -> Arc<Self> {
        let c = Arc::new(Self::default());
        c.master.set(1.0);
        c.open.set(1.0);
        c
    }
}

/// Everything the audio thread plays. Made on the game thread, then moved to the audio
/// thread, where `render` is called for each buffer.
pub struct Mixer {
    controls: Arc<Controls>,
    ambience: Ambience,
    master: dsp::Smooth,
}

impl Mixer {
    pub fn new(controls: Arc<Controls>, seed: u32) -> Self {
        Self { controls, ambience: Ambience::new(seed), master: dsp::Smooth::new(0.0, 0.05) }
    }

    /// Fills `out` with interleaved stereo frames. Allocates nothing and takes no lock.
    pub fn render(&mut self, out: &mut [f32]) {
        let c = &self.controls;
        self.ambience.set(ambience::Settings {
            wind: c.wind.get(),
            rain: c.rain.get(),
            water: c.water.get(),
            trees: c.trees.get(),
            daylight: c.daylight.get(),
            open: c.open.get(),
        });
        let master_target = c.master.get().clamp(0.0, 1.0);
        for frame in out.chunks_exact_mut(CHANNELS) {
            let (l, r) = self.ambience.next();
            let m = self.master.next(master_target);
            // A gentle limit, so that no mix of layers ever clips.
            frame[0] = soft_limit(l * m);
            frame[1] = soft_limit(r * m);
        }
    }
}

fn soft_limit(x: f32) -> f32 {
    if x.is_finite() {
        x / (1.0 + x.abs() * 0.6)
    } else {
        0.0
    }
}
