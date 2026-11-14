//! The words' motifs, from their sheets: no two alike, and a listening sample of each
//! word, then its typo going wrong, then the sounds of a fight, in mono.

use vellmark_audio::cues::{midi, Cue, CueKind, Timbre};
use vellmark_audio::{Controls, Mixer, SAMPLE_RATE};

/// Each sheet's word, class, and motif, read plainly from `assets/data/words`.
fn sheets() -> Vec<(String, String, Vec<u8>)> {
    let dir = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/data/words");
    let mut out = Vec::new();
    let mut files: Vec<_> = std::fs::read_dir(&dir).unwrap().filter_map(|e| e.ok()).map(|e| e.path()).filter(|p| p.extension().is_some_and(|x| x == "ron") && p.file_stem().is_some_and(|s| s != "targets")).collect();
    files.sort();
    for f in files {
        let text = std::fs::read_to_string(&f).unwrap();
        let field = |key: &str| text.lines().find_map(|l| l.trim().strip_prefix(key)).map(|v| v.trim().trim_end_matches(',').trim_matches('"').to_string());
        let word = field("word:").unwrap();
        let class = field("class:").unwrap();
        let motif = field("motif:").unwrap();
        let notes = motif.trim_matches(|c| c == '[' || c == ']').split(',').map(|n| midi(n.trim().trim_matches('"')).unwrap()).collect();
        out.push((word, class, notes));
    }
    out
}

#[test]
fn every_word_has_a_motif_of_its_own() {
    let s = sheets();
    assert_eq!(s.len(), 7, "the seven words");
    for (i, a) in s.iter().enumerate() {
        assert!((2..=6).contains(&a.2.len()), "{}: a motif of two to six notes", a.0);
        for b in &s[i + 1..] {
            assert_ne!(a.2, b.2, "{} and {} share a motif", a.0, b.0);
        }
    }
}

/// Writes `docs/evidence/S03/motifs.wav`. `cargo test -p vellmark-audio -- --ignored`.
#[test]
#[ignore]
fn write_a_listening_sample() {
    let controls = Controls::new();
    controls.master.set(0.9);
    let mut mixer = Mixer::new(controls.clone(), 7);
    let mut samples = Vec::new();
    let mut buffer = vec![0.0f32; 960];
    let mut play = |cue: Option<Cue>, seconds: f32| {
        if let Some(c) = cue {
            controls.cues.push(c);
        }
        for _ in 0..(seconds * SAMPLE_RATE as f32 / 480.0) as usize {
            mixer.render(&mut buffer);
            samples.extend(buffer.chunks_exact(2).map(|v| ((v[0] + v[1]) * 0.5).clamp(-1.0, 1.0) * 32767.0).map(|v| v as i16));
        }
    };
    for (_, class, motif) in sheets() {
        let mut notes = [0u8; 6];
        for (n, m) in notes.iter_mut().zip(&motif) {
            *n = *m;
        }
        let cue = Cue { kind: CueKind::Motif, gain: 0.8, notes, timbre: Timbre::of_class(&class), charge: 0.6, ..Default::default() };
        play(Some(cue), 1.6);
        play(Some(Cue { wrong: true, ..cue }), 1.8);
    }
    for kind in [CueKind::Swing, CueKind::Hit, CueKind::SwingHeavy, CueKind::HitHeavy, CueKind::Block, CueKind::Bash, CueKind::Hurt, CueKind::Loose, CueKind::Thunk, CueKind::Gather, CueKind::Split, CueKind::Spill, CueKind::Refusal, CueKind::Break, CueKind::Mend] {
        play(Some(Cue { kind, gain: 0.8, notes: [62, 0, 0, 0, 0, 0], ..Default::default() }), 0.9);
    }
    play(None, 1.0);
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/evidence/S03/motifs.wav");
    let mut out = Vec::new();
    let data = (samples.len() * 2) as u32;
    out.extend_from_slice(b"RIFF");
    out.extend_from_slice(&(36 + data).to_le_bytes());
    out.extend_from_slice(b"WAVEfmt ");
    out.extend_from_slice(&16u32.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&1u16.to_le_bytes());
    out.extend_from_slice(&SAMPLE_RATE.to_le_bytes());
    out.extend_from_slice(&(SAMPLE_RATE * 2).to_le_bytes());
    out.extend_from_slice(&2u16.to_le_bytes());
    out.extend_from_slice(&16u16.to_le_bytes());
    out.extend_from_slice(b"data");
    out.extend_from_slice(&data.to_le_bytes());
    for s in samples {
        out.extend_from_slice(&s.to_le_bytes());
    }
    std::fs::write(path, out).unwrap();
}
