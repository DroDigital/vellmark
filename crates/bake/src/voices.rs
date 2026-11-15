//! The murmur's voices, from `assets/data/voices.ron`: each speaker's preset over its base,
//! with a crowd voice's own fixed offset, resolved to the numbers the murmur speaks with.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use vellmark_content::{Voice, VoiceTable};

use crate::BakeError;

#[derive(Deserialize)]
struct Base {
    pitch: f32,
    size: f32,
    rate: f32,
    breath: f32,
    gravel: f32,
}

#[derive(Deserialize)]
struct Preset {
    base: String,
    /// Semitones, percent, and percent.
    pitch: i32,
    formant: i32,
    rate: i32,
}

#[derive(Deserialize)]
struct Speaker {
    preset: String,
    #[serde(default)]
    offset: bool,
}

#[derive(Deserialize)]
struct Authored {
    bases: BTreeMap<String, Base>,
    presets: BTreeMap<String, Preset>,
    speakers: BTreeMap<String, Speaker>,
}

/// A crowd voice's fixed offset from its key: pitch within two semitones (never less
/// than one, to keep it from the named voice it stands on), formant within 6%, and rate
/// within 8%, as the cast file says.
fn offset(key: &str) -> (f32, f32, f32) {
    let h = vellmark_content::fnv1a64(key.as_bytes());
    let unit = |shift: u32| ((h >> shift) & 0xffff) as f32 / 65535.0 * 2.0 - 1.0;
    let semis = unit(0);
    let semis = semis.signum() * (1.0 + semis.abs());
    (semis, unit(16) * 0.06, unit(32) * 0.08)
}

pub fn bake_voices(file: &Path) -> Result<VoiceTable, BakeError> {
    let text = std::fs::read_to_string(file).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
    let authored: Authored = ron::from_str(&text).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
    let mut table = VoiceTable::default();
    for (key, s) in &authored.speakers {
        let preset = authored.presets.get(&s.preset).ok_or_else(|| BakeError(format!("{}: `{key}` has no preset `{}`", file.display(), s.preset)))?;
        let base = authored.bases.get(&preset.base).ok_or_else(|| BakeError(format!("{}: no base `{}`", file.display(), preset.base)))?;
        let (semis, formant, rate) = if s.offset { offset(key) } else { (0.0, 0.0, 0.0) };
        table.speakers.push((
            key.clone(),
            Voice {
                pitch: base.pitch * 2f32.powf((preset.pitch as f32 + semis) / 12.0),
                size: base.size * (1.0 + preset.formant as f32 / 100.0 + formant),
                rate: base.rate * (preset.rate as f32 / 100.0) * (1.0 + rate),
                breath: base.breath,
                gravel: base.gravel,
            },
        ));
    }
    Ok(table)
}
