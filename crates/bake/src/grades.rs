//! Colour grades: `assets/grades/<place>.ron` baked as `grade.<place>`.

use serde::Deserialize;
use std::path::Path;
use vellmark_content::{Grade, GradeKey};

use crate::{err, BakeError};

#[derive(Deserialize)]
struct Key {
    hour: f32,
    lift: (f32, f32, f32),
    gamma: (f32, f32, f32),
    gain: (f32, f32, f32),
    saturation: f32,
    fog: (f32, f32, f32),
}

/// Every grade in the folder, as (key, bytes).
pub fn bake_all(folder: &Path) -> Result<Vec<(String, Vec<u8>)>, BakeError> {
    let mut out = Vec::new();
    for file in crate::sorted_files(folder, "ron")? {
        let name = file.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_owned();
        let text = std::fs::read_to_string(&file).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
        let keys: Vec<Key> = ron::from_str(&text).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
        if keys.is_empty() || keys.windows(2).any(|w| w[1].hour <= w[0].hour) || keys.iter().any(|k| !(0.0..24.0).contains(&k.hour)) {
            return err(format!("{}: keys must be in order of the hour, from 0 to 24", file.display()));
        }
        let t = |v: (f32, f32, f32)| [v.0, v.1, v.2];
        let grade = Grade {
            keys: keys
                .iter()
                .map(|k| GradeKey {
                    hour: k.hour,
                    lift: t(k.lift),
                    gamma: t(k.gamma),
                    gain: t(k.gain),
                    saturation: k.saturation,
                    fog_density: k.fog.0,
                    fog_floor: k.fog.1,
                    fog_falloff: k.fog.2,
                })
                .collect(),
        };
        out.push((format!("grade.{name}"), grade.to_bytes()));
    }
    Ok(out)
}
