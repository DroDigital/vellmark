//! The murmur: a line spoken under its text as wordless speech, shaped by its speaker's
//! voice (pitch, size, pace, breath, gravel) and by the line itself. Each vowel of the
//! text is a syllable's vowel, its consonants hiss, click, or hum, its spaces and
//! punctuation are its pauses, and the phrase falls in pitch, or rises at a question.
//! Synthesized: a glottal pulse through three formants. The game sets a line in a slot
//! the audio thread reads without a lock; nothing allocates once made.

use std::sync::atomic::{AtomicU32, AtomicU8, Ordering};

use crate::dsp::{Rng, Svf};
use crate::SAMPLE_RATE;

/// The most characters of a line the murmur speaks; a longer line is spoken that far.
pub const MURMUR_CHARS: usize = 320;

/// A voice as the murmur speaks it: pitch in hertz, size (1 an adult's middle), syllables
/// a second, breath and gravel 0 to 1.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct MurmurVoice {
    pub pitch: f32,
    pub size: f32,
    pub rate: f32,
    pub breath: f32,
    pub gravel: f32,
}

/// Where the game sets the line to be murmured.
pub struct MurmurSlot {
    text: [AtomicU8; MURMUR_CHARS],
    len: AtomicU32,
    voice: [AtomicU32; 5],
    gain: AtomicU32,
    version: AtomicU32,
}

impl Default for MurmurSlot {
    fn default() -> Self {
        Self {
            text: std::array::from_fn(|_| AtomicU8::new(0)),
            len: AtomicU32::new(0),
            voice: std::array::from_fn(|_| AtomicU32::new(0)),
            gain: AtomicU32::new(0),
            version: AtomicU32::new(0),
        }
    }
}

impl MurmurSlot {
    /// Murmurs a line in a voice, cutting short any line under way.
    pub fn say(&self, text: &str, voice: MurmurVoice, gain: f32) {
        let bytes: Vec<u8> = text.bytes().filter(|b| b.is_ascii()).take(MURMUR_CHARS).collect();
        for (slot, b) in self.text.iter().zip(&bytes) {
            slot.store(*b, Ordering::Relaxed);
        }
        self.len.store(bytes.len() as u32, Ordering::Relaxed);
        for (slot, v) in self.voice.iter().zip([voice.pitch, voice.size, voice.rate, voice.breath, voice.gravel]) {
            slot.store(v.to_bits(), Ordering::Relaxed);
        }
        self.gain.store(gain.to_bits(), Ordering::Relaxed);
        self.version.fetch_add(1, Ordering::Release);
    }

    /// Stops the murmur.
    pub fn hush(&self) {
        self.len.store(0, Ordering::Relaxed);
        self.version.fetch_add(1, Ordering::Release);
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
enum Segment {
    Vowel { f1: f32, f2: f32 },
    /// Unvoiced noise in a band: s, f, h.
    Hiss { band: f32 },
    /// A stop: a short click, then nothing.
    Click,
    /// Voiced and closed: m, n, l, r, w.
    Hum,
    Pause,
}

/// Vowel formants for an adult's middle-sized voice, in hertz.
fn vowel(c: u8) -> Option<(f32, f32)> {
    Some(match c.to_ascii_lowercase() {
        b'a' => (780.0, 1250.0),
        b'e' => (520.0, 1850.0),
        b'i' => (310.0, 2250.0),
        b'o' => (520.0, 920.0),
        b'u' => (330.0, 860.0),
        b'y' => (320.0, 2050.0),
        _ => return None,
    })
}

pub struct Murmur {
    text: [u8; MURMUR_CHARS],
    len: usize,
    at: usize,
    voice: MurmurVoice,
    gain: f32,
    version: u32,
    segment: Segment,
    left: f32,
    phase: f32,
    pitch: f32,
    f1: f32,
    f2: f32,
    level: f32,
    filters: [Svf; 3],
    rng: Rng,
}

impl Murmur {
    pub fn new(seed: u32) -> Self {
        Self {
            text: [0; MURMUR_CHARS],
            len: 0,
            at: 0,
            voice: MurmurVoice { pitch: 140.0, size: 1.0, rate: 4.0, breath: 0.1, gravel: 0.0 },
            gain: 0.0,
            version: 0,
            segment: Segment::Pause,
            left: 0.0,
            phase: 0.0,
            pitch: 140.0,
            f1: 500.0,
            f2: 1500.0,
            level: 0.0,
            filters: Default::default(),
            rng: Rng::new(seed),
        }
    }

    /// Takes a new line from the slot, if one was set.
    pub fn take(&mut self, slot: &MurmurSlot) {
        let version = slot.version.load(Ordering::Acquire);
        if version == self.version {
            return;
        }
        self.version = version;
        self.len = (slot.len.load(Ordering::Relaxed) as usize).min(MURMUR_CHARS);
        for (b, s) in self.text.iter_mut().zip(&slot.text) {
            *b = s.load(Ordering::Relaxed);
        }
        let f = |k: usize| f32::from_bits(slot.voice[k].load(Ordering::Relaxed));
        self.voice = MurmurVoice { pitch: f(0).max(40.0), size: f(1).max(0.5), rate: f(2).max(1.0), breath: f(3), gravel: f(4) };
        self.gain = f32::from_bits(slot.gain.load(Ordering::Relaxed));
        self.at = 0;
        self.left = 0.0;
        self.segment = Segment::Pause;
    }

    pub fn speaking(&self) -> bool {
        self.at < self.len || self.left > 0.0
    }

    /// The next stretch of the line: what it is and how long it lasts, in seconds.
    fn next_segment(&mut self) -> Option<(Segment, f32)> {
        let syllable = 1.0 / self.voice.rate;
        while self.at < self.len {
            let c = self.text[self.at];
            self.at += 1;
            let size = self.voice.size;
            let seg = match c {
                b' ' => (Segment::Pause, syllable * 0.12),
                b',' | b';' | b':' | b'-' => (Segment::Pause, syllable * 0.9),
                b'.' | b'!' | b'?' => (Segment::Pause, syllable * 1.6),
                _ if vowel(c).is_some() => {
                    let (f1, f2) = vowel(c).unwrap();
                    // A vowel run is one syllable's vowel.
                    while self.at < self.len && vowel(self.text[self.at]).is_some() {
                        self.at += 1;
                    }
                    (Segment::Vowel { f1: f1 / size, f2: f2 / size }, syllable * 0.62)
                }
                b's' | b'S' | b'z' | b'Z' | b'c' | b'C' | b'x' | b'X' => (Segment::Hiss { band: 5200.0 / size }, syllable * 0.18),
                b'f' | b'F' | b'v' | b'V' | b'h' | b'H' => (Segment::Hiss { band: 2600.0 / size }, syllable * 0.14),
                b'p' | b'P' | b't' | b'T' | b'k' | b'K' | b'b' | b'B' | b'd' | b'D' | b'g' | b'G' | b'q' | b'Q' | b'j' | b'J' => (Segment::Click, syllable * 0.12),
                b'm' | b'M' | b'n' | b'N' | b'l' | b'L' | b'r' | b'R' | b'w' | b'W' => (Segment::Hum, syllable * 0.16),
                _ => continue,
            };
            return Some(seg);
        }
        None
    }

    /// One sample.
    pub fn next(&mut self) -> f32 {
        if self.left <= 0.0 {
            match self.next_segment() {
                Some((seg, length)) => {
                    self.segment = seg;
                    self.left = length;
                    // Each syllable's pitch: the phrase falls, a question rises, and no two
                    // syllables are quite alike.
                    let progress = self.at as f32 / self.len.max(1) as f32;
                    let question = self.text[..self.len].iter().rev().find(|c| !c.is_ascii_whitespace()) == Some(&b'?');
                    let contour = if question && progress > 0.8 { 1.0 + (progress - 0.8) * 1.5 } else { 1.12 - 0.2 * progress };
                    self.pitch = self.voice.pitch * contour * (1.0 + self.rng.signed() * 0.04);
                }
                None => {
                    self.level *= 0.995;
                    return 0.0;
                }
            }
        }
        let dt = 1.0 / SAMPLE_RATE as f32;
        self.left -= dt;
        let noise = self.rng.signed();
        let (voiced, unvoiced, f1, f2, band, target) = match self.segment {
            Segment::Vowel { f1, f2 } => (1.0, self.voice.breath * 0.4, f1, f2, 0.0, 1.0),
            Segment::Hum => (0.6, 0.0, 280.0 / self.voice.size, 1200.0 / self.voice.size, 0.0, 0.6),
            Segment::Hiss { band } => (0.0, 1.0, self.f1, self.f2, band, 0.35),
            Segment::Click => (0.0, 1.0, self.f1, self.f2, 1800.0, if self.left > 0.08 / self.voice.rate { 0.6 } else { 0.0 }),
            Segment::Pause => (0.0, 0.0, self.f1, self.f2, 0.0, 0.0),
        };
        // Glide the formants and the level, so that nothing clicks between sounds.
        self.f1 += (f1 - self.f1) * 0.004;
        self.f2 += (f2 - self.f2) * 0.004;
        self.level += (target - self.level) * 0.006;
        // The glottal pulse: a sharp-edged saw, roughened by gravel.
        let wobble = 1.0 + (self.rng.signed() * self.voice.gravel * 0.08);
        self.phase = (self.phase + self.pitch * wobble * dt).fract();
        let pulse = (1.0 - self.phase).powi(3) * 2.0 - 0.5;
        let creak = if self.voice.gravel > 0.0 && self.phase < 0.02 { 1.0 + self.voice.gravel * self.rng.signed() } else { 1.0 };
        let source = pulse * creak * voiced + noise * unvoiced * if band > 0.0 { 0.0 } else { 1.0 };
        let size = self.voice.size;
        let a = self.filters[0].next(source, self.f1, 5.0).band;
        let b = self.filters[1].next(source, self.f2, 7.0).band * 0.6;
        let c = self.filters[2].next(source, 2600.0 / size, 9.0).band * 0.25;
        let hiss = if band > 0.0 { self.filters[2].next(noise, band, 2.0).band * unvoiced } else { 0.0 };
        // The formants ring up the pulse: kept to about the loudness of the other cues.
        (a + b + c + hiss) * self.level * self.gain * 0.2
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn speak(text: &str, voice: MurmurVoice) -> Vec<f32> {
        let slot = MurmurSlot::default();
        let mut m = Murmur::new(3);
        slot.say(text, voice, 0.5);
        m.take(&slot);
        let mut out = Vec::new();
        while m.speaking() && out.len() < SAMPLE_RATE as usize * 30 {
            out.push(m.next());
        }
        for _ in 0..SAMPLE_RATE / 4 {
            out.push(m.next());
        }
        out
    }

    const WICK: MurmurVoice = MurmurVoice { pitch: 108.0, size: 0.94, rate: 3.6, breath: 0.12, gravel: 0.35 };
    const TAMSIN: MurmurVoice = MurmurVoice { pitch: 315.0, size: 1.34, rate: 5.2, breath: 0.1, gravel: 0.0 };

    #[test]
    fn a_line_is_murmured_for_about_as_long_as_it_would_be_said_and_then_stops() {
        let line = "Now that is a poor way to start the day.";
        let x = speak(line, WICK);
        let seconds = x.len() as f32 / SAMPLE_RATE as f32;
        // Eleven syllables at 3.6 a second, with its pauses: two to five seconds.
        assert!((2.0..5.0).contains(&seconds), "{seconds} s");
        let loud = x.iter().fold(0.0f32, |m, s| m.max(s.abs()));
        assert!(x.iter().all(|s| s.is_finite() && s.abs() < 1.5), "peak {loud}");
        assert!(loud > 0.02, "it is heard: {loud}");
        assert!(x[x.len() - 2000..].iter().all(|s| s.abs() < 0.01), "and it stops");
    }

    #[test]
    fn voices_differ_and_a_quick_voice_is_quicker() {
        let line = "Are you dead? Take your time.";
        let (wick, tamsin) = (speak(line, WICK), speak(line, TAMSIN));
        assert!(tamsin.len() < wick.len(), "Tamsin speaks quicker than Wick");
    }

    #[test]
    fn a_new_line_cuts_short_the_last_and_hush_stops_it() {
        let slot = MurmurSlot::default();
        let mut m = Murmur::new(1);
        slot.say("A long line that would take a long while to say, and then some more.", WICK, 0.5);
        m.take(&slot);
        for _ in 0..4800 {
            m.next();
        }
        slot.hush();
        m.take(&slot);
        assert!(!m.speaking());
    }
}
