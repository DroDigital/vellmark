//! Vellmark's sound. The mixer runs on the audio thread: it never allocates, never locks,
//! and reads what the game wants through atomics. The game thread writes them each frame.
//!
//! The Showcase's ambience is synthesized: wind in gusts, leaves near trees, rain, the
//! river, and birdsong by day. Recorded layers join it when the approved sample packs
//! are in `third_party/inbox/` (see `third_party/DOWNLOADS.md`).

pub mod ambience;
pub mod cues;
pub mod dsp;
pub mod score;
pub mod murmur;

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
    /// One-shot sounds: motifs, blows, and the rest.
    pub cues: cues::Cues,
    /// The one-shot sounds' loudness, from the settings.
    pub effects: Param,
    /// The surroundings' loudness, from the settings.
    pub ambience: Param,
    /// The score's loudness, from the settings.
    pub music: Param,
    /// How near danger is, 0 to 1, for the score.
    pub danger: Param,
    /// The Vale's theme as MIDI notes, each motif ended by a 0, and a count that changes
    /// when it is set.
    pub theme: [std::sync::atomic::AtomicU32; score::THEME_NOTES],
    pub theme_version: std::sync::atomic::AtomicU32,
    /// The line being murmured.
    pub murmur: murmur::MurmurSlot,
}

impl Controls {
    /// Sets the theme from the Vale's motifs, in order.
    pub fn set_theme(&self, motifs: &[&[u8]]) {
        use std::sync::atomic::Ordering;
        let mut k = 0;
        for slot in &self.theme {
            slot.store(0, Ordering::Relaxed);
        }
        for m in motifs {
            for &n in m.iter() {
                if k + 1 < score::THEME_NOTES {
                    self.theme[k].store(n as u32, Ordering::Relaxed);
                    k += 1;
                }
            }
            k += 1;
        }
        self.theme_version.fetch_add(1, Ordering::Release);
    }
}

impl Controls {
    pub fn new() -> Arc<Self> {
        let c = Arc::new(Self::default());
        c.master.set(1.0);
        c.open.set(1.0);
        c.effects.set(1.0);
        c.ambience.set(1.0);
        c.music.set(1.0);
        c
    }
}

/// Everything the audio thread plays. Made on the game thread, then moved to the audio
/// thread, where `render` is called for each buffer.
pub struct Mixer {
    controls: Arc<Controls>,
    ambience: Ambience,
    master: dsp::Smooth,
    cues: cues::Player,
    score: score::Score,
    theme_version: u32,
    murmur: murmur::Murmur,
    /// The score's share while someone speaks: it steps back under speech.
    duck: dsp::Smooth,
}

impl Mixer {
    pub fn new(controls: Arc<Controls>, seed: u32) -> Self {
        Self { controls, ambience: Ambience::new(seed), master: dsp::Smooth::new(0.0, 0.05), cues: cues::Player::new(), score: score::Score::new(seed ^ 0x5c0e), theme_version: 0, murmur: murmur::Murmur::new(seed ^ 0x3f1d), duck: dsp::Smooth::new(1.0, 0.4) }
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
        self.cues.take(&c.cues);
        let effects = c.effects.get().clamp(0.0, 1.0);
        let surroundings = c.ambience.get().clamp(0.0, 1.0);
        let music = c.music.get().clamp(0.0, 1.0);
        let mood = score::Mood { danger: c.danger.get(), daylight: c.daylight.get() };
        // A new theme, read without a lock: the notes, then the count that says so.
        let version = c.theme_version.load(std::sync::atomic::Ordering::Acquire);
        if version != self.theme_version {
            self.theme_version = version;
            let mut notes = [0u8; score::THEME_NOTES];
            for (n, slot) in notes.iter_mut().zip(&c.theme) {
                *n = slot.load(std::sync::atomic::Ordering::Relaxed) as u8;
            }
            self.score.set_theme_notes(notes);
        }
        self.murmur.take(&c.murmur);
        for frame in out.chunks_exact_mut(CHANNELS) {
            let (l, r) = self.ambience.next();
            let (l, r) = (l * surroundings, r * surroundings);
            let (ml, mr) = self.score.next(mood);
            let duck = self.duck.next(if self.murmur.speaking() { 0.55 } else { 1.0 });
            let (l, r) = (l + ml * music * duck, r + mr * music * duck);
            let said = self.murmur.next() * effects;
            let (l, r) = (l + said, r + said);
            let (el, er) = self.cues.next();
            let (l, r) = (l + el * effects, r + er * effects);
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
