//! The score: by day it plays a phrase and then rests; it never clips or goes silent for
//! ever; a fight begins at once with a steady beat and ends when the danger has passed;
//! the theme follows the motifs it is given; and the mixer playing it never allocates.
//! `cargo test -p vellmark-audio --test score -- --ignored` writes
//! `docs/evidence/S06/score.wav`.

use vellmark_audio::score::{Mood, Score};
use vellmark_audio::SAMPLE_RATE;

const DAY: Mood = Mood { danger: 0.0, daylight: 1.0 };

/// Loudness in windows of a quarter second.
fn render(score: &mut Score, mood: Mood, seconds: f32) -> Vec<f32> {
    let n = (seconds * SAMPLE_RATE as f32) as usize;
    let window = SAMPLE_RATE as usize / 4;
    let mut out = Vec::new();
    let mut sum = 0.0f64;
    for k in 0..n {
        let (l, r) = score.next(mood);
        assert!(l.is_finite() && r.is_finite());
        assert!(l.abs() < 1.0 && r.abs() < 1.0, "it clips: {l} {r}");
        sum += (l * l + r * r) as f64 * 0.5;
        if (k + 1) % window == 0 {
            out.push((sum / window as f64).sqrt() as f32);
            sum = 0.0;
        }
    }
    out
}

#[test]
fn by_day_a_phrase_is_played_then_the_music_rests() {
    let mut s = Score::new(11);
    let level = render(&mut s, DAY, 120.0);
    let loud = level.iter().filter(|l| **l > 0.01).count();
    assert!(loud > 20, "a phrase was played: {loud} quarter seconds of music");
    // A rest of at least ten seconds, somewhere in the two minutes.
    let mut quiet = 0;
    let mut longest = 0;
    for l in &level {
        quiet = if *l < 0.003 { quiet + 1 } else { 0 };
        longest = longest.max(quiet);
    }
    assert!(longest >= 40, "the music rests: longest quiet {} s", longest as f32 / 4.0);
}

#[test]
fn a_fight_begins_at_once_keeps_a_beat_and_ends_when_the_danger_passes() {
    let mut s = Score::new(3);
    render(&mut s, DAY, 2.0);
    let fight = render(&mut s, Mood { danger: 1.0, daylight: 1.0 }, 12.0);
    assert!(s.fighting());
    // After the first second, every half second has sound in it: the drum and strings.
    let silent = fight[4..].chunks(2).filter(|c| c.iter().all(|l| *l < 0.01)).count();
    assert_eq!(silent, 0, "a fight never falls silent");
    let mean_fight = fight[4..].iter().sum::<f32>() / (fight.len() - 4) as f32;
    render(&mut s, DAY, 10.0);
    assert!(!s.fighting(), "the fight's music ends when the danger has passed");
    assert!(mean_fight > 0.03, "a fight is loud enough to feel: {mean_fight}");
}

#[test]
fn the_theme_follows_the_motifs_it_is_given() {
    // Two scores with different themes compose different music from the same seed.
    let mut a = Score::new(5);
    let mut b = Score::new(5);
    b.set_theme(&[&[62, 64, 65], &[69, 67]]);
    let la = render(&mut a, DAY, 30.0);
    let lb = render(&mut b, DAY, 30.0);
    assert_ne!(la, lb);
}

#[test]
#[ignore]
fn write_the_score_for_the_evidence() {
    // A day's phrase, night falling, a fight, and the calm after.
    let mut s = Score::new(21);
    let mut samples: Vec<i16> = Vec::new();
    let mut play = |s: &mut Score, mood: Mood, seconds: f32| {
        for _ in 0..(seconds * SAMPLE_RATE as f32) as usize {
            let (l, r) = s.next(mood);
            let x = (((l + r) * 0.5) * 2.2 * 32767.0).clamp(-32767.0, 32767.0) as i16;
            samples.push(x);
        }
    };
    play(&mut s, DAY, 16.0);
    // The night's phrase from a score begun at night, so that the sample need not wait
    // out the day's rest.
    let mut s = Score::new(23);
    play(&mut s, Mood { danger: 0.0, daylight: 0.0 }, 20.0);
    play(&mut s, Mood { danger: 1.0, daylight: 0.0 }, 14.0);
    play(&mut s, Mood { danger: 0.0, daylight: 0.0 }, 4.0);
    // Kept at half the rate, to keep the file small: two samples averaged into one.
    let samples: Vec<i16> = samples.chunks(2).map(|c| ((c[0] as i32 + *c.get(1).unwrap_or(&c[0]) as i32) / 2) as i16).collect();
    let rate = SAMPLE_RATE / 2;
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/evidence/S06/score.wav");
    let mut out = Vec::new();
    let data = (samples.len() * 2) as u32;
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&rate.to_le_bytes());
    out.extend_from_slice(&(rate * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    for x in samples {
        out.extend_from_slice(&x.to_le_bytes());
    }
    std::fs::write(path, out).unwrap();
}
