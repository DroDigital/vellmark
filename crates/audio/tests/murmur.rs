//! A listening sample of the murmur for the owner's ear, in the voices the cast file sets:
//! Wick's first lines, then a line each in Mabel's and Tamsin's, all from the script.
//! `cargo test -p vellmark-audio --test murmur -- --ignored` writes
//! `docs/evidence/S06/murmur.wav`.

use vellmark_audio::murmur::{Murmur, MurmurSlot, MurmurVoice};
use vellmark_audio::SAMPLE_RATE;

#[test]
#[ignore]
fn write_a_listening_sample() {
    // As the bake resolves them from assets/data/voices.ron.
    let wick = MurmurVoice { pitch: 108.0, size: 0.94, rate: 3.6, breath: 0.12, gravel: 0.35 };
    let mabel = MurmurVoice { pitch: 205.0, size: 1.08, rate: 4.4, breath: 0.15, gravel: 0.0 };
    let tamsin = MurmurVoice { pitch: 314.0, size: 1.345, rate: 5.2, breath: 0.1, gravel: 0.0 };
    let lines = [
        (wick, "Now that is a poor way to start the day."),
        (wick, "Are you dead? Take your time. I get paid either way, or I would, if the form had a box for you."),
        (mabel, "You'll eat something. It isn't a question. Tamsin! Lay another place!"),
        (tamsin, "Everybody does. On your sleeves and that. It says who you are, if you get close."),
    ];
    let slot = MurmurSlot::default();
    let mut m = Murmur::new(7);
    let mut samples: Vec<i16> = Vec::new();
    for (voice, line) in lines {
        slot.say(line, voice, 0.9);
        m.take(&slot);
        while m.speaking() {
            samples.push((m.next() * 30000.0).clamp(-32767.0, 32767.0) as i16);
        }
        for _ in 0..SAMPLE_RATE / 2 {
            samples.push(0);
        }
    }
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/evidence/S06/murmur.wav");
    let rate = SAMPLE_RATE / 2;
    let samples: Vec<i16> = samples.chunks(2).map(|c| ((c[0] as i32 + *c.get(1).unwrap_or(&c[0]) as i32) / 2) as i16).collect();
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
