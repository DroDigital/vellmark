//! One-shot sounds: a word's motif, a blow, a swing, an arrow, an Inkling. The game thread
//! pushes cues into a queue of fixed size; the audio thread takes them and plays each on
//! a voice from a fixed pool. Nothing here allocates once made, and nothing locks.
//!
//! Every sound is synthesized. A motif is played as bells whose timbre is the word's
//! class; a typo's motif goes wrong on its last note, flat and sour.

use std::sync::atomic::{AtomicU32, Ordering};

use crate::dsp::{pan, Rng, Sine, Svf};
use crate::SAMPLE_RATE;

/// The most cues waiting at once; more are dropped.
const SLOTS: usize = 64;
/// The most voices sounding at once; a new one takes the quietest.
const VOICES: usize = 24;
/// The most notes in a motif.
pub const MOTIF_NOTES: usize = 6;

/// What to play.
#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct Cue {
    pub kind: CueKind,
    /// Where it is heard from: -1 left, 1 right.
    pub pan: f32,
    /// How loud, 0 to 1, after distance.
    pub gain: f32,
    /// For a motif: its notes as MIDI numbers, 0 for none; its timbre; how charged.
    pub notes: [u8; MOTIF_NOTES],
    pub timbre: Timbre,
    pub charge: f32,
    /// The motif goes wrong on its last note.
    pub wrong: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum CueKind {
    #[default]
    Motif,
    /// A word refused: a dull knock under its first note.
    Refusal,
    /// Steel through the air: light or heavy.
    Swing,
    SwingHeavy,
    /// Steel into ink: a wet strike, and a heavier one.
    Hit,
    HitHeavy,
    /// A blow taken on a guard: wood or steel ringing.
    Block,
    /// A blow taken: a dull thump.
    Hurt,
    /// A shield's bash.
    Bash,
    /// The bowstring, and an arrow striking home.
    Loose,
    Thunk,
    /// An Inkling gathering itself: a rising hiss. Splitting: a splat. Dying: it runs out.
    Gather,
    Split,
    Spill,
    /// A thing falling to its letters: a dry crumble. Standing whole: a rising shimmer.
    Break,
    Mend,
}

/// The sound of a word's class.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum Timbre {
    /// Bright bells: Access.
    #[default]
    Bell,
    /// Struck glass: Integrity.
    Glass,
    /// Warm reeds: Temperature.
    Reed,
    /// A celesta: Light.
    Celesta,
    /// A soft swell: Mind.
    Swell,
    /// Struck metal: Size, Weight.
    Metal,
}

impl Timbre {
    /// The timbre of a word's class, by the class's name.
    pub fn of_class(class: &str) -> Self {
        match class {
            "Access" => Self::Bell,
            "Integrity" => Self::Glass,
            "Temperature" => Self::Reed,
            "Light" => Self::Celesta,
            "Mind" => Self::Swell,
            _ => Self::Metal,
        }
    }

    /// Modulator ratio, brightness, and decay in seconds.
    fn shape(self) -> (f32, f32, f32) {
        match self {
            Self::Bell => (3.5, 2.2, 1.6),
            Self::Glass => (5.0, 1.4, 2.2),
            Self::Reed => (1.0, 2.6, 0.9),
            Self::Celesta => (4.0, 1.0, 1.1),
            Self::Swell => (2.0, 0.8, 1.8),
            Self::Metal => (2.76, 3.0, 1.3),
        }
    }
}

/// A note's name, `D4` or `F#3`, as a MIDI number.
pub fn midi(name: &str) -> Option<u8> {
    let b = name.as_bytes();
    let base = match b.first()?.to_ascii_uppercase() {
        b'C' => 0,
        b'D' => 2,
        b'E' => 4,
        b'F' => 5,
        b'G' => 7,
        b'A' => 9,
        b'B' => 11,
        _ => return None,
    };
    let (shift, rest) = match b.get(1) {
        Some(b'#') => (1, &name[2..]),
        Some(b'b') => (-1, &name[2..]),
        _ => (0, &name[1..]),
    };
    let octave: i32 = rest.parse().ok()?;
    u8::try_from((octave + 1) * 12 + base + shift).ok()
}

fn hz(midi: f32) -> f32 {
    440.0 * 2f32.powf((midi - 69.0) / 12.0)
}

/// A cue packed into words that can be written and read without a lock.
struct Slot([AtomicU32; 8]);

impl Slot {
    const fn new() -> Self {
        Self([const { AtomicU32::new(0) }; 8])
    }
}

/// A queue of cues from one writer, the game thread, to one reader, the audio thread.
pub struct Cues {
    slots: [Slot; SLOTS],
    written: AtomicU32,
    read: AtomicU32,
}

impl Default for Cues {
    fn default() -> Self {
        Self { slots: [const { Slot::new() }; SLOTS], written: AtomicU32::new(0), read: AtomicU32::new(0) }
    }
}

impl Cues {
    /// Called only from the game thread. A full queue drops the cue.
    pub fn push(&self, cue: Cue) {
        let w = self.written.load(Ordering::Relaxed);
        let r = self.read.load(Ordering::Acquire);
        if w.wrapping_sub(r) as usize >= SLOTS {
            return;
        }
        let s = &self.slots[w as usize % SLOTS].0;
        let notes = u32::from_le_bytes([cue.notes[0], cue.notes[1], cue.notes[2], cue.notes[3]]);
        let more = u32::from_le_bytes([cue.notes[4], cue.notes[5], cue.timbre as u8, cue.wrong as u8]);
        s[0].store(cue.kind as u32, Ordering::Relaxed);
        s[1].store(cue.pan.to_bits(), Ordering::Relaxed);
        s[2].store(cue.gain.to_bits(), Ordering::Relaxed);
        s[3].store(notes, Ordering::Relaxed);
        s[4].store(more, Ordering::Relaxed);
        s[5].store(cue.charge.to_bits(), Ordering::Relaxed);
        self.written.store(w.wrapping_add(1), Ordering::Release);
    }

    /// Called only from the audio thread.
    fn pop(&self) -> Option<Cue> {
        let r = self.read.load(Ordering::Relaxed);
        if r == self.written.load(Ordering::Acquire) {
            return None;
        }
        let s = &self.slots[r as usize % SLOTS].0;
        let kind = KINDS.get(s[0].load(Ordering::Relaxed) as usize).copied().unwrap_or_default();
        let n = s[3].load(Ordering::Relaxed).to_le_bytes();
        let m = s[4].load(Ordering::Relaxed).to_le_bytes();
        let timbre = TIMBRES.get(m[2] as usize).copied().unwrap_or_default();
        let cue = Cue {
            kind,
            pan: f32::from_bits(s[1].load(Ordering::Relaxed)),
            gain: f32::from_bits(s[2].load(Ordering::Relaxed)),
            notes: [n[0], n[1], n[2], n[3], m[0], m[1]],
            timbre,
            charge: f32::from_bits(s[5].load(Ordering::Relaxed)),
            wrong: m[3] != 0,
        };
        self.read.store(r.wrapping_add(1), Ordering::Release);
        Some(cue)
    }
}

const KINDS: [CueKind; 17] = [
    CueKind::Motif,
    CueKind::Refusal,
    CueKind::Swing,
    CueKind::SwingHeavy,
    CueKind::Hit,
    CueKind::HitHeavy,
    CueKind::Block,
    CueKind::Hurt,
    CueKind::Bash,
    CueKind::Loose,
    CueKind::Thunk,
    CueKind::Gather,
    CueKind::Split,
    CueKind::Spill,
    CueKind::Break,
    CueKind::Mend,
    CueKind::Motif,
];
const TIMBRES: [Timbre; 6] = [Timbre::Bell, Timbre::Glass, Timbre::Reed, Timbre::Celesta, Timbre::Swell, Timbre::Metal];

/// One sounding voice.
#[derive(Clone, Default)]
struct Voice {
    cue: Cue,
    /// Seconds since it began.
    t: f32,
    /// How long it sounds, after which it is free.
    length: f32,
    carrier: [Sine; 2],
    modulator: [Sine; 2],
    filter: Svf,
    rng: Option<Rng>,
}

impl Voice {
    fn level(&self) -> f32 {
        if self.length <= 0.0 {
            0.0
        } else {
            self.cue.gain * (1.0 - self.t / self.length).max(0.0)
        }
    }

    fn start(&mut self, cue: Cue, seed: u32) {
        let notes = cue.notes.iter().filter(|n| **n > 0).count() as f32;
        let (_, _, decay) = cue.timbre.shape();
        let length = match cue.kind {
            CueKind::Motif => NOTE_GAP * notes.max(1.0) + decay * 1.3,
            CueKind::Refusal => 0.7,
            CueKind::Swing => 0.25,
            CueKind::SwingHeavy => 0.4,
            CueKind::Hit => 0.3,
            CueKind::HitHeavy => 0.5,
            CueKind::Block => 0.9,
            CueKind::Hurt => 0.35,
            CueKind::Bash => 0.4,
            CueKind::Loose => 0.35,
            CueKind::Thunk => 0.25,
            CueKind::Gather => 0.7,
            CueKind::Split => 0.35,
            CueKind::Spill => 1.2,
            CueKind::Break => 1.1,
            CueKind::Mend => 1.8,
        };
        *self = Voice { cue, t: 0.0, length, rng: Some(Rng::new(seed)), ..Default::default() };
    }

    /// One sample, mono, before panning.
    fn next(&mut self) -> f32 {
        let dt = 1.0 / SAMPLE_RATE as f32;
        let t = self.t;
        self.t += dt;
        if t >= self.length {
            self.length = 0.0;
            return 0.0;
        }
        let noise = self.rng.as_mut().map_or(0.0, |r| r.signed());
        let c = self.cue;
        match c.kind {
            CueKind::Motif => self.motif(t),
            CueKind::Refusal => {
                // A knock, and the first note, smothered.
                let knock = self.carrier[0].next(90.0 - 40.0 * t) * (-t * 14.0).exp();
                let first = c.notes[0] as f32;
                let note = if first > 0.0 { self.carrier[1].next(hz(first - 1.0)) * (-t * 6.0).exp() * 0.25 } else { 0.0 };
                knock * 0.8 + note
            }
            CueKind::Swing | CueKind::SwingHeavy => {
                // Air past a blade: noise swept up and down through a band.
                let heavy = c.kind == CueKind::SwingHeavy;
                let p = t / self.length;
                let f = if heavy { 300.0 + 900.0 * (p * std::f32::consts::PI).sin() } else { 600.0 + 2200.0 * (p * std::f32::consts::PI).sin() };
                let band = self.filter.next(noise, f, 2.5).band;
                band * (p * std::f32::consts::PI).sin().powi(2) * if heavy { 0.9 } else { 0.6 }
            }
            CueKind::Hit | CueKind::HitHeavy => {
                // Steel into ink: a wet slap over a low body.
                let heavy = c.kind == CueKind::HitHeavy;
                let body = self.carrier[0].next(if heavy { 70.0 } else { 110.0 } * (1.0 - t * 1.5).max(0.4)) * (-t * if heavy { 9.0 } else { 14.0 }).exp();
                let slap = self.filter.next(noise, 1400.0 - 2000.0 * t, 1.2).low * (-t * 30.0).exp();
                body * if heavy { 1.0 } else { 0.7 } + slap * 1.4
            }
            CueKind::Block => {
                // A guard taking a blow: steel ringing over a knock.
                let ring = (self.carrier[0].next(820.0) + self.carrier[1].next(1253.0) * 0.6) * (-t * 4.0).exp() * 0.35;
                let knock = self.filter.next(noise, 500.0, 1.0).band * (-t * 40.0).exp();
                ring + knock * 1.2
            }
            CueKind::Hurt => self.carrier[0].next(85.0 - 30.0 * t) * (-t * 11.0).exp() + self.filter.next(noise, 300.0, 0.8).low * (-t * 25.0).exp(),
            CueKind::Bash => self.carrier[0].next(60.0) * (-t * 9.0).exp() * 1.1 + self.filter.next(noise, 900.0, 1.0).band * (-t * 30.0).exp(),
            CueKind::Loose => {
                // The string: a twang that falls.
                let f = 180.0 * (1.0 - t * 0.4);
                (self.carrier[0].next(f) + self.carrier[1].next(f * 2.01) * 0.4) * (-t * 12.0).exp() * 0.7 + self.filter.next(noise, 2000.0, 1.0).band * (-t * 40.0).exp() * 0.5
            }
            CueKind::Thunk => self.carrier[0].next(140.0 - 80.0 * t) * (-t * 25.0).exp() + self.filter.next(noise, 700.0, 1.5).band * (-t * 35.0).exp(),
            CueKind::Gather => {
                // A hiss that rises as it gathers.
                let p = t / self.length;
                self.filter.next(noise, 800.0 + 3000.0 * p, 3.0).band * p * 0.9
            }
            CueKind::Split => self.filter.next(noise, 900.0 - 600.0 * t, 1.0).low * (-t * 12.0).exp() * 1.4,
            CueKind::Spill => {
                // Running out: a long falling gurgle.
                let wobble = self.modulator[0].next(13.0) * 0.5 + 0.5;
                self.filter.next(noise, 400.0 + 500.0 * wobble * (1.0 - t / self.length), 4.0).band * (1.0 - t / self.length) * 0.8
            }
            CueKind::Break => {
                // Letters falling: a dry crackle that thins.
                let crackle = if noise.abs() > 0.97 - 0.2 * (-t * 3.0).exp() { noise } else { 0.0 };
                self.filter.next(crackle, 3500.0, 0.7).high * (-t * 2.5).exp() * 1.5
            }
            CueKind::Mend => {
                // Letters coming home: a shimmer of bells that rises into a chord.
                let mut s = 0.0;
                for (k, ratio) in [1.0f32, 1.25, 1.5, 2.0].iter().enumerate() {
                    let begin = k as f32 * 0.12;
                    if t > begin {
                        let u = t - begin;
                        s += (u * hz(72.0) * ratio * std::f32::consts::TAU).sin() * (-u * 2.2).exp() * (u * 30.0).min(1.0);
                    }
                }
                s * 0.25
            }
        }
    }

    fn motif(&mut self, t: f32) -> f32 {
        let c = self.cue;
        let (ratio, brightness, decay) = c.timbre.shape();
        let notes = c.notes.iter().filter(|n| **n > 0).count();
        let mut s = 0.0;
        // At most the two latest notes still ring loudly; earlier ones are let fade by
        // their envelope alone.
        for (k, note) in c.notes.iter().enumerate().filter(|(_, n)| **n > 0) {
            let begin = k as f32 * NOTE_GAP;
            if t < begin {
                continue;
            }
            let u = t - begin;
            let last = k + 1 == notes;
            let mut pitch = *note as f32;
            if c.wrong && last {
                // The wrong note: a semitone flat and sliding further, beating against itself.
                pitch -= 1.0 + 0.4 * (u * 3.0).min(1.0);
            }
            let f = hz(pitch);
            let env = (u * 200.0).min(1.0) * (-u / decay).exp();
            // Two-operator FM: the brightness falls as the note rings.
            let index = brightness * (0.4 + c.charge * 0.6) * (-u * 3.0).exp() + 0.3;
            let m = (u * f * ratio * std::f32::consts::TAU).sin() * index;
            let mut v = (u * f * std::f32::consts::TAU + m).sin();
            if c.wrong && last {
                // Sour: a second partial a little off.
                v = v * 0.7 + (u * f * 1.013 * std::f32::consts::TAU).sin() * 0.5;
            }
            s += v * env;
        }
        s * 0.32
    }
}

/// Seconds between a motif's notes.
pub const NOTE_GAP: f32 = 0.13;

/// The voices and a small room for them to ring in.
pub struct Player {
    voices: [Voice; VOICES],
    seed: u32,
    /// A small room: two feedback delays.
    room: Box<[f32]>,
    room_at: usize,
}

impl Player {
    pub fn new() -> Self {
        Self { voices: std::array::from_fn(|_| Voice::default()), seed: 0x51ed, room: vec![0.0; 4096].into_boxed_slice(), room_at: 0 }
    }

    /// Takes waiting cues and gives each a voice: a free one, or else the quietest.
    pub fn take(&mut self, cues: &Cues) {
        while let Some(cue) = cues.pop() {
            let free = (0..VOICES).min_by(|a, b| self.voices[*a].level().total_cmp(&self.voices[*b].level())).unwrap_or(0);
            self.seed = self.seed.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            self.voices[free].start(cue, self.seed);
        }
    }

    /// One stereo frame.
    pub fn next(&mut self) -> (f32, f32) {
        let (mut l, mut r) = (0.0, 0.0);
        for v in self.voices.iter_mut().filter(|v| v.length > 0.0) {
            let x = v.next() * v.cue.gain;
            let (a, b) = pan(x, v.cue.pan);
            l += a;
            r += b;
        }
        // The room: what was sent a moment ago comes back, softer, crossed.
        let n = self.room.len();
        let back_l = self.room[(self.room_at + n - 2711) % n];
        let back_r = self.room[(self.room_at + n - 3527) % n];
        self.room[self.room_at] = (l + r) * 0.5 + (back_l + back_r) * 0.38;
        self.room_at = (self.room_at + 1) % n;
        (l + back_r * 0.3, r + back_l * 0.3)
    }

    /// How many voices are sounding.
    pub fn sounding(&self) -> usize {
        self.voices.iter().filter(|v| v.length > 0.0).count()
    }
}

impl Default for Player {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn render(cue: Cue, seconds: f32) -> Vec<f32> {
        let cues = Cues::default();
        let mut p = Player::new();
        cues.push(cue);
        p.take(&cues);
        (0..(seconds * SAMPLE_RATE as f32) as usize).map(|_| p.next().0).collect()
    }

    /// The strongest frequency between `lo` and `hi`, by a plain DFT over a window.
    fn peak(x: &[f32], lo: f32, hi: f32) -> f32 {
        let mut best = (0.0, lo);
        let mut f = lo;
        while f < hi {
            let (mut re, mut im) = (0.0f32, 0.0f32);
            for (i, s) in x.iter().enumerate() {
                let a = std::f32::consts::TAU * f * i as f32 / SAMPLE_RATE as f32;
                re += s * a.cos();
                im += s * a.sin();
            }
            let m = re * re + im * im;
            if m > best.0 {
                best = (m, f);
            }
            f += 2.0;
        }
        best.1
    }

    #[test]
    fn note_names_are_midi_numbers() {
        assert_eq!(midi("A4"), Some(69));
        assert_eq!(midi("C4"), Some(60));
        assert_eq!(midi("F#3"), Some(54));
        assert_eq!(midi("Bb2"), Some(46));
        assert_eq!(midi("H2"), None);
    }

    #[test]
    fn a_motif_plays_its_notes_and_a_typo_goes_wrong_on_the_last() {
        let d4 = midi("D4").unwrap();
        let a4 = midi("A4").unwrap();
        let cue = Cue { kind: CueKind::Motif, gain: 1.0, notes: [d4, a4, 0, 0, 0, 0], ..Default::default() };
        let right = render(cue, 0.6);
        let wrong = render(Cue { wrong: true, ..cue }, 0.6);
        let first = &right[100..(NOTE_GAP * SAMPLE_RATE as f32) as usize];
        assert!((peak(first, 250.0, 350.0) - 293.7).abs() < 4.0, "the first note is D4");
        let at = (NOTE_GAP * SAMPLE_RATE as f32) as usize + 2400;
        assert!((peak(&right[at..at + 4800], 400.0, 480.0) - 440.0).abs() < 4.0, "the second is A4");
        let sour = peak(&wrong[at..at + 4800], 380.0, 480.0);
        assert!(sour < 430.0, "the typo's last note is flat: {sour}");
    }

    #[test]
    fn every_cue_sounds_stays_bounded_and_ends() {
        for kind in KINDS {
            let x = render(Cue { kind, gain: 1.0, notes: [62, 65, 64, 62, 0, 0], ..Default::default() }, 4.0);
            let loudest = x.iter().fold(0.0f32, |m, s| m.max(s.abs()));
            assert!(loudest > 0.01 && loudest < 4.0, "{kind:?}: {loudest}");
            assert!(x.iter().all(|s| s.is_finite()));
            assert!(x[x.len() - 2400..].iter().all(|s| s.abs() < 0.05), "{kind:?} rings on");
        }
    }

    #[test]
    fn a_full_queue_drops_cues_rather_than_wait() {
        let cues = Cues::default();
        for _ in 0..SLOTS + 10 {
            cues.push(Cue { kind: CueKind::Hit, gain: 1.0, ..Default::default() });
        }
        let mut p = Player::new();
        p.take(&cues);
        assert_eq!(p.sounding(), VOICES);
        assert!(cues.pop().is_none());
    }
}
