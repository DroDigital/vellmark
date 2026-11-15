//! The ambience answers the world: louder in a stronger wind, brighter in rain, a river
//! where there is water, birds only by day, silence when there is nothing, and never a
//! sample out of range.

use vellmark_audio::{ambience::Settings, Ambience, Controls, Mixer, SAMPLE_RATE};

fn render(s: Settings, seconds: f32) -> (Vec<f32>, u32) {
    let mut a = Ambience::new(7);
    a.set(s);
    let n = (seconds * SAMPLE_RATE as f32) as usize;
    let mut out = Vec::with_capacity(n * 2);
    for _ in 0..n {
        let (l, r) = a.next();
        out.push(l);
        out.push(r);
    }
    (out, a.calls_begun())
}

/// The loudness of the last half, once every layer has faded in.
fn rms(x: &[f32]) -> f32 {
    let tail = &x[x.len() / 2..];
    (tail.iter().map(|v| v * v).sum::<f32>() / tail.len() as f32).sqrt()
}

/// How often the signal crosses zero: high for bright sounds, low for dull ones.
fn brightness(x: &[f32]) -> f32 {
    let tail: Vec<f32> = x[x.len() / 2..].iter().step_by(2).copied().collect();
    tail.windows(2).filter(|w| (w[0] < 0.0) != (w[1] < 0.0)).count() as f32 / tail.len() as f32
}

#[test]
fn nothing_is_silence() {
    let (out, _) = render(Settings { open: 1.0, ..Default::default() }, 2.0);
    assert!(rms(&out) < 1.0e-6, "{}", rms(&out));
}

#[test]
fn a_stronger_wind_is_louder() {
    let calm = rms(&render(Settings { wind: 0.2, ..Default::default() }, 6.0).0);
    let gale = rms(&render(Settings { wind: 0.9, ..Default::default() }, 6.0).0);
    assert!(calm > 1.0e-4 && gale > calm * 2.0, "calm {calm}, gale {gale}");
}

#[test]
fn rain_is_brighter_than_wind_and_the_river_is_heard() {
    let (wind, _) = render(Settings { wind: 0.6, ..Default::default() }, 4.0);
    let (rain, _) = render(Settings { rain: 0.8, ..Default::default() }, 4.0);
    assert!(brightness(&rain) > brightness(&wind) * 1.5, "rain {} wind {}", brightness(&rain), brightness(&wind));
    let (river, _) = render(Settings { water: 1.0, ..Default::default() }, 4.0);
    assert!(rms(&river) > 1.0e-3, "{}", rms(&river));
}

#[test]
fn birds_sing_by_day_and_not_at_night_or_in_rain() {
    let (_, day) = render(Settings { daylight: 1.0, open: 1.0, wind: 0.2, ..Default::default() }, 30.0);
    let (_, night) = render(Settings { daylight: 0.0, open: 1.0, wind: 0.2, ..Default::default() }, 30.0);
    let (_, storm) = render(Settings { daylight: 1.0, open: 1.0, rain: 1.0, ..Default::default() }, 30.0);
    assert!(day >= 4, "{day} calls by day");
    assert_eq!(night, 0);
    assert_eq!(storm, 0);
}

#[test]
fn everything_at_once_never_leaves_the_range() {
    let controls = Controls::new();
    for p in [&controls.wind, &controls.rain, &controls.water, &controls.trees, &controls.daylight] {
        p.set(1.0);
    }
    let mut mixer = Mixer::new(controls, 3);
    let mut buffer = vec![0.0f32; 1024];
    for _ in 0..(SAMPLE_RATE as usize * 10 / 512) {
        mixer.render(&mut buffer);
        assert!(buffer.iter().all(|v| v.is_finite() && v.abs() <= 1.0));
    }
    assert!(buffer.iter().any(|v| v.abs() > 1.0e-3));
}

/// Writes a listening sample for the evidence: four seconds of a breezy morning among
/// trees with birds, then four of rain beside the river. `cargo test -p vellmark-audio
/// -- --ignored`.
#[test]
#[ignore]
fn write_a_listening_sample() {
    let controls = Controls::new();
    let mut mixer = Mixer::new(controls.clone(), 11);
    let mut samples = Vec::new();
    let mut buffer = vec![0.0f32; 960];
    for block in 0..(8 * SAMPLE_RATE as usize / 480) {
        let t = block as f32 * 480.0 / SAMPLE_RATE as f32;
        let morning = t < 4.0;
        controls.wind.set(if morning { 0.45 } else { 0.6 });
        controls.trees.set(if morning { 0.8 } else { 0.2 });
        controls.daylight.set(1.0);
        controls.rain.set(if morning { 0.0 } else { 0.85 });
        controls.water.set(if morning { 0.0 } else { 0.9 });
        mixer.render(&mut buffer);
        samples.extend(buffer.iter().map(|v| (v.clamp(-1.0, 1.0) * 32767.0) as i16));
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/evidence/S02/ambience.wav");
    let mut out = Vec::new();
    let data = (samples.len() * 2) as u32;
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    for v in [16u32] {
        out.extend_from_slice(&v.to_le_bytes());
    }
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&(SAMPLE_RATE * 4).to_le_bytes());
    out.extend_from_slice(&4u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    std::fs::write(path, out).unwrap();
}
