//! The score: a modal folk score composed as it plays, from the Vale's main theme, which
//! is the motifs of its four words played in turn (OPEN, WHOLE, LIT, CALM: it begins and
//! ends on D). By day a plucked string takes a phrase of it over a soft bowed chord and a
//! drone, then the music rests a while; by night a choir hums it lower and slower; in
//! danger the drone grows, low strings drive in eighths, a drum keeps the beat, and the
//! theme comes back on bowed strings, faster. All of it is synthesized here: no
//! recordings, so nothing can be claimed. Nothing allocates once made, and nothing locks.

use crate::dsp::{pan, Rng, Smooth, Svf};
use crate::SAMPLE_RATE;

/// The most notes in the theme, motifs and their ends together.
pub const THEME_NOTES: usize = 24;
const VOICES: usize = 16;
const QUEUE: usize = 64;
/// A plucked string's longest delay line: low enough for D2.
const PLUCK_LINE: usize = 2048;

fn hz(midi: f32) -> f32 {
    440.0 * 2f32.powf((midi - 69.0) / 12.0)
}

/// D Dorian, as pitch classes: D E F G A B C.
const MODE: [u8; 7] = [2, 4, 5, 7, 9, 11, 0];

/// The mode's chords, as roots and their thirds and fifths in pitch classes: Dm, F, C, Am,
/// G, Em. The first that holds a melody note is taken, so Dm is home.
const CHORDS: [[u8; 3]; 6] = [[2, 5, 9], [5, 9, 0], [0, 4, 7], [9, 0, 4], [7, 11, 2], [4, 7, 11]];

#[derive(Clone, Copy, Debug, Default, PartialEq)]
enum Kind {
    #[default]
    Pluck,
    Bow,
    Choir,
    Drum,
}

#[derive(Clone, Copy, Debug, Default)]
struct Note {
    /// In beats from the score's start.
    at: f64,
    kind: Kind,
    midi: u8,
    /// Its length in seconds.
    length: f32,
    gain: f32,
    pan: f32,
}

struct Voice {
    kind: Kind,
    freq: f32,
    t: f32,
    length: f32,
    gain: f32,
    pan: f32,
    phase: f32,
    vibrato: f32,
    filters: [Svf; 3],
    line: Box<[f32]>,
    line_len: usize,
    line_at: usize,
    rng: Rng,
}

impl Voice {
    fn new(seed: u32) -> Self {
        Self {
            kind: Kind::Pluck,
            freq: 0.0,
            t: 0.0,
            length: 0.0,
            gain: 0.0,
            pan: 0.0,
            phase: 0.0,
            vibrato: 0.0,
            filters: Default::default(),
            line: vec![0.0; PLUCK_LINE].into_boxed_slice(),
            line_len: 1,
            line_at: 0,
            rng: Rng::new(seed),
        }
    }

    fn start(&mut self, n: &Note) {
        self.kind = n.kind;
        self.freq = hz(n.midi as f32);
        self.t = 0.0;
        self.length = n.length;
        self.gain = n.gain;
        self.pan = n.pan;
        self.phase = 0.0;
        self.vibrato = self.rng.unit();
        self.filters = Default::default();
        if n.kind == Kind::Pluck {
            // A string plucked: its delay line filled with a burst of noise, softened.
            self.line_len = ((SAMPLE_RATE as f32 / self.freq) as usize).clamp(2, PLUCK_LINE);
            let mut last = 0.0;
            for k in 0..self.line_len {
                let x = self.rng.signed();
                last = last * 0.5 + x * 0.5;
                self.line[k] = last;
            }
            self.line_at = 0;
        }
    }

    fn sounding(&self) -> bool {
        self.t < self.length + 1.5
    }

    fn next(&mut self) -> f32 {
        let dt = 1.0 / SAMPLE_RATE as f32;
        self.t += dt;
        let release = if self.t > self.length { (1.0 - (self.t - self.length) / 1.5).max(0.0) } else { 1.0 };
        let x = match self.kind {
            Kind::Pluck => {
                // Karplus-Strong: the line averaged as it goes round, losing a little.
                let a = self.line[self.line_at];
                let b = self.line[(self.line_at + 1) % self.line_len];
                let y = (a + b) * 0.5 * 0.996;
                self.line[self.line_at] = y;
                self.line_at = (self.line_at + 1) % self.line_len;
                let body = self.filters[0].next(a, 2400.0, 0.7).low;
                body * 0.9
            }
            Kind::Bow => {
                // A bowed string: a saw that swells in, with a slow vibrato, warmed.
                let vib = 1.0 + (self.t * 5.2 + self.vibrato * 6.0).sin() * 0.004 * (self.t * 0.8).min(1.0);
                self.phase = (self.phase + self.freq * vib * dt).fract();
                let saw = self.phase * 2.0 - 1.0;
                let attack = (self.t / 0.35).min(1.0);
                let lp = self.filters[0].next(saw, self.freq * 3.2 + 500.0, 0.7).low;
                self.filters[1].next(lp, 280.0, 0.6).high * attack * 0.55
            }
            Kind::Choir => {
                // A voice humming "ah": a pulse through three formants.
                let vib = 1.0 + (self.t * 4.6 + self.vibrato * 6.0).sin() * 0.006;
                self.phase = (self.phase + self.freq * vib * dt).fract();
                let src = if self.phase < 0.12 { 1.0 } else { -0.136 };
                let attack = (self.t / 0.6).min(1.0);
                let f1 = self.filters[0].next(src, 760.0, 6.0).band;
                let f2 = self.filters[1].next(src, 1180.0, 7.0).band;
                let f3 = self.filters[2].next(src, 2600.0, 9.0).band;
                (f1 + f2 * 0.6 + f3 * 0.25) * attack * 0.9
            }
            Kind::Drum => {
                // A frame drum: a falling thump and a little skin.
                let pitch = self.freq * (1.0 + 2.5 * (-self.t * 30.0).exp());
                self.phase = (self.phase + pitch * dt).fract();
                let body = (self.phase * std::f32::consts::TAU).sin() * (-self.t * 9.0).exp();
                let skin = self.filters[0].next(self.rng.signed(), 900.0, 1.2).band * (-self.t * 40.0).exp();
                body + skin * 0.4
            }
        };
        x * self.gain * release
    }
}

/// What the score follows, set by the game each frame.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Mood {
    /// 0 calm to 1 a fight.
    pub danger: f32,
    /// 1 by day, 0 at night.
    pub daylight: f32,
}

/// The score, played on the audio thread.
pub struct Score {
    rng: Rng,
    /// The theme as MIDI notes, each motif ended by a 0.
    theme: [u8; THEME_NOTES],
    queue: [Option<Note>; QUEUE],
    voices: [Voice; VOICES],
    /// In beats.
    beat: f64,
    /// Beats composed up to.
    composed: f64,
    /// Beats until the next phrase, while it rests.
    resting_until: f64,
    danger: Smooth,
    daylight: Smooth,
    drone: f32,
    drone_phase: [f32; 3],
    drone_filter: Svf,
    fighting: bool,
    /// Where the last phrase ends, in beats: the drone sounds until a little after.
    phrase_until: f64,
}

impl Score {
    pub fn new(seed: u32) -> Self {
        let mut s = Self {
            rng: Rng::new(seed),
            theme: [0; THEME_NOTES],
            queue: [None; QUEUE],
            voices: std::array::from_fn(|k| Voice::new(seed ^ (k as u32 * 0x9e37_79b9))),
            beat: 0.0,
            composed: 0.0,
            // The first phrase comes after a short while, not at once.
            resting_until: 6.0,
            danger: Smooth::new(0.0, 2.5),
            daylight: Smooth::new(1.0, 20.0),
            drone: 0.0,
            drone_phase: [0.0; 3],
            drone_filter: Svf::default(),
            fighting: false,
            phrase_until: 0.0,
        };
        s.set_theme(&[&[62, 69, 74], &[62, 65, 64, 62], &[69, 72, 76, 79], &[69, 67, 64, 62]]);
        s
    }

    /// The theme: the Vale's motifs, in the order they are played.
    pub fn set_theme(&mut self, motifs: &[&[u8]]) {
        self.theme = [0; THEME_NOTES];
        let mut k = 0;
        for m in motifs {
            for &n in m.iter() {
                if k + 1 < THEME_NOTES {
                    self.theme[k] = n;
                    k += 1;
                }
            }
            k += 1;
        }
    }

    /// The theme as it is kept: MIDI notes, each motif ended by a 0.
    pub fn set_theme_notes(&mut self, notes: [u8; THEME_NOTES]) {
        if notes.iter().any(|n| *n != 0) {
            self.theme = notes;
        }
    }

    /// The theme's motifs, each as its notes.
    fn motifs(&self) -> impl Iterator<Item = &[u8]> {
        self.theme.split(|n| *n == 0).filter(|m| !m.is_empty())
    }

    fn tempo(&self, danger: f32, daylight: f32) -> f32 {
        if self.fighting {
            126.0
        } else {
            (56.0 + daylight * 14.0) * (1.0 + danger * 0.3)
        }
    }

    fn schedule(&mut self, n: Note) {
        if let Some(slot) = self.queue.iter_mut().find(|s| s.is_none()) {
            *slot = Some(n);
        }
    }

    /// The chord that holds a note: the first of the mode's that does.
    fn chord_for(note: u8) -> [u8; 3] {
        *CHORDS.iter().find(|c| c.contains(&(note % 12))).unwrap_or(&CHORDS[0])
    }

    /// Composes the next phrase or bar, from `self.composed`.
    fn compose(&mut self, danger: f32, daylight: f32) {
        let at = self.composed;
        if self.fighting {
            // A bar of 3/4 in a fight: drum, low strings in eighths, and every other
            // bar a motif high on bowed strings.
            let count = self.motifs().count().max(1);
            let pick = (self.rng.next_u32() as usize) % count;
            let mut motif = [0u8; 8];
            let mut len = 0;
            if let Some(m) = self.motifs().nth(pick) {
                for (k, n) in m.iter().take(8).enumerate() {
                    motif[k] = *n;
                    len = k + 1;
                }
            }
            let root = Self::chord_for(motif[0])[0];
            let low = 38 + ((root as i32 + 12 - 2) % 12) as u8;
            let beat_s = 60.0 / 126.0;
            for b in 0..3 {
                let accent = if b == 0 { 1.0 } else { 0.55 };
                self.schedule(Note { at: at + b as f64, kind: Kind::Drum, midi: 38, length: 0.25, gain: 0.26 * accent, pan: 0.0 });
                for half in 0..2 {
                    let midi = if (b * 2 + half) % 3 == 2 { low + 7 } else { low };
                    self.schedule(Note { at: at + b as f64 + half as f64 * 0.5, kind: Kind::Bow, midi, length: beat_s * 0.42, gain: 0.08, pan: -0.3 });
                }
            }
            if self.rng.unit() < 0.6 {
                let mut t = 0.0;
                for k in 0..len {
                    let beats = if k + 1 == len { 1.5 } else { 0.5 };
                    self.schedule(Note { at: at + t, kind: Kind::Bow, midi: motif[k] + 12, length: beats as f32 * beat_s * 0.95, gain: 0.1, pan: 0.35 });
                    t += beats;
                }
            }
            self.composed = at + 3.0;
            return;
        }
        if at < self.resting_until {
            self.composed = self.resting_until;
            return;
        }
        // A phrase: the whole theme, two motifs, or one, as it was or turned in the mode.
        let roll = self.rng.unit();
        let count = self.motifs().count().max(1);
        let (first, take) = if roll < 0.35 { (0, count) } else if roll < 0.75 { ((self.rng.next_u32() as usize) % count, 2) } else { ((self.rng.next_u32() as usize) % count, 1) };
        let night = daylight < 0.4;
        let turn = self.rng.unit() < 0.2;
        let shift: i32 = if night { -12 } else if self.rng.unit() < 0.25 { 12 } else { 0 };
        let melody = if night { Kind::Choir } else { Kind::Pluck };
        let beat_s = 60.0 / self.tempo(danger, daylight);
        let mut t = 0.0;
        for m in 0..take {
            let mut notes = [0u8; 8];
            let mut len = 0;
            if let Some(motif) = self.motifs().nth((first + m) % count) {
                for (k, n) in motif.iter().take(8).enumerate() {
                    notes[k] = *n;
                    len = k + 1;
                }
            }
            if len == 0 {
                continue;
            }
            // Turned: each note mirrored round the first, along the mode.
            if turn {
                let pivot = notes[0];
                for k in 1..len {
                    let up = notes[k] as i32 - pivot as i32;
                    let steps = (up as f32 / 1.7).round() as i32;
                    notes[k] = Self::step(pivot, -steps);
                }
            }
            let rhythm: &[f64] = match len {
                1 | 2 => &[3.0, 3.0],
                3 => &[2.0, 1.0, 3.0],
                4 => &[1.0, 1.0, 1.0, 3.0],
                _ => &[1.0, 0.5, 0.5, 1.0, 3.0, 1.0, 1.0, 1.0],
            };
            // The chord beneath the motif's first two bars, on bowed strings or the choir.
            let chord = Self::chord_for(notes[0]);
            let pad = if night { Kind::Choir } else { Kind::Bow };
            for (k, pc) in chord.iter().enumerate() {
                // Voiced upward from D3.
                let midi = 50 + (*pc as i32 - 2).rem_euclid(12) as u8;
                self.schedule(Note { at: at + t, kind: pad, midi, length: (6.0 * beat_s) as f32, gain: 0.05, pan: (k as f32 - 1.0) * 0.4 });
            }
            let mut u = 0.0;
            for k in 0..len {
                let midi = (notes[k] as i32 + shift).clamp(36, 96) as u8;
                let beats = rhythm.get(k).copied().unwrap_or(1.0);
                let gain = if melody == Kind::Pluck { 0.3 } else { 0.11 };
                self.schedule(Note { at: at + t + u, kind: melody, midi, length: (beats as f32 * beat_s * if melody == Kind::Pluck { 2.0 } else { 0.95 }), gain, pan: 0.15 });
                u += beats;
            }
            t += 6.0;
        }
        self.composed = at + t;
        self.phrase_until = self.composed;
        // Then a rest: long by day, longer by night, short when danger is near.
        let rest_s = if danger > 0.2 { self.rng.range(2.0, 6.0) } else { self.rng.range(18.0, 50.0) * if night { 1.3 } else { 1.0 } };
        self.resting_until = self.composed + rest_s as f64 / beat_s as f64;
    }

    /// A note moved by `steps` along the mode, staying in it.
    fn step(note: u8, steps: i32) -> u8 {
        let mut n = note as i32;
        let dir = steps.signum();
        let mut left = steps.abs();
        while left > 0 {
            n += dir;
            if MODE.contains(&((n.rem_euclid(12)) as u8)) {
                left -= 1;
            }
        }
        n.clamp(24, 108) as u8
    }

    /// One stereo frame.
    pub fn next(&mut self, mood: Mood) -> (f32, f32) {
        let danger = self.danger.next(mood.danger.clamp(0.0, 1.0));
        let daylight = self.daylight.next(mood.daylight.clamp(0.0, 1.0));
        // A fight begins at once, and ends when the danger has passed.
        if !self.fighting && mood.danger > 0.6 {
            self.fighting = true;
            self.composed = self.beat.ceil();
            self.queue = [None; QUEUE];
        } else if self.fighting && danger < 0.15 {
            self.fighting = false;
            self.resting_until = self.beat + 8.0;
        }
        let tempo = self.tempo(danger, daylight);
        self.beat += tempo as f64 / 60.0 / SAMPLE_RATE as f64;
        if self.composed < self.beat + 3.0 {
            self.compose(danger, daylight);
        }
        // Notes come due.
        for k in 0..QUEUE {
            if let Some(n) = self.queue[k] {
                if n.at <= self.beat {
                    self.queue[k] = None;
                    if let Some(v) = self.voices.iter_mut().find(|v| !v.sounding()) {
                        v.start(&n);
                    } else if let Some(v) = self.voices.iter_mut().max_by(|a, b| a.t.total_cmp(&b.t)) {
                        v.start(&n);
                    }
                }
            }
        }
        let (mut l, mut r) = (0.0, 0.0);
        for v in self.voices.iter_mut().filter(|v| v.sounding()) {
            let x = v.next();
            let (a, b) = pan(x, v.pan);
            l += a;
            r += b;
        }
        // The drone: D and A, low, with a buzz that grows with the danger; it sounds while
        // music plays and fades in the rests.
        let playing = self.beat < self.phrase_until + 2.0;
        let target = if self.fighting { 0.07 } else if playing { 0.05 + 0.03 * danger } else { 0.0 };
        self.drone += (target - self.drone) * (1.0 / (SAMPLE_RATE as f32 * 2.0));
        if self.drone > 1e-4 {
            let dt = 1.0 / SAMPLE_RATE as f32;
            let mut d = 0.0;
            for (k, f) in [hz(38.0), hz(45.0), hz(50.0)].into_iter().enumerate() {
                self.drone_phase[k] = (self.drone_phase[k] + f * dt).fract();
                d += (self.drone_phase[k] * 2.0 - 1.0) * [1.0, 0.7, 0.4][k];
            }
            let buzz = 1.0 + danger * 0.8;
            let warm = self.drone_filter.next(d * buzz, 380.0 + danger * 900.0, 0.8).low;
            l += warm * self.drone;
            r += warm * self.drone;
        }
        (l, r)
    }

    /// Whether any note sounds, for tests.
    pub fn sounding(&self) -> usize {
        self.voices.iter().filter(|v| v.sounding()).count()
    }

    pub fn fighting(&self) -> bool {
        self.fighting
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_note_composed_is_in_the_mode() {
        let mut s = Score::new(9);
        for round in 0..400 {
            s.fighting = round % 3 == 0;
            s.resting_until = 0.0;
            s.compose(if s.fighting { 1.0 } else { 0.0 }, if round % 2 == 0 { 1.0 } else { 0.0 });
            for n in s.queue.iter_mut().filter_map(|n| n.take()) {
                if n.kind != Kind::Drum {
                    assert!(MODE.contains(&(n.midi % 12)), "{:?} is not in D Dorian", n);
                }
            }
        }
    }
}
