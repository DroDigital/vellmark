//! Camera paths: `assets/paths/<name>.ron`, baked under the key `path.<name>`.

use crate::{err, sorted_files, BakeError};
use serde::Deserialize;
use std::path::Path;
use vellmark_content::{CameraPath, Keyframe};

#[derive(Deserialize)]
struct Key {
    eye: (f32, f32, f32),
    /// Degrees. 0 looks north, 90 looks east.
    yaw: f32,
    /// Degrees. Positive looks up.
    pitch: f32,
    hour: f32,
    /// Seconds to reach the next keyframe.
    #[serde(default)]
    seconds: f32,
}

#[derive(Deserialize)]
struct Authored {
    keys: Vec<Key>,
}

pub fn bake_all(dir: &Path) -> Result<Vec<(String, Vec<u8>)>, BakeError> {
    let mut out = Vec::new();
    for file in sorted_files(dir, "ron")? {
        let name = file.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_owned();
        let text = std::fs::read_to_string(&file).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
        let authored: Authored = ron::from_str(&text).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
        if authored.keys.len() < 2 {
            return err(format!("{}: a camera path needs at least two keyframes", file.display()));
        }
        let keys = authored
            .keys
            .iter()
            .map(|k| Keyframe {
                eye: [k.eye.0, k.eye.1, k.eye.2],
                yaw: k.yaw.to_radians(),
                pitch: k.pitch.to_radians(),
                hour: k.hour,
                seconds: k.seconds,
            })
            .collect();
        out.push((format!("path.{name}"), CameraPath { keys }.to_bytes()));
    }
    Ok(out)
}
