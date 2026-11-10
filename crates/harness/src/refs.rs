//! Reference images. A small set of captures is kept in the repository at low
//! resolution. A capture that moves from its reference by more than its threshold fails
//! until someone has looked at the new image and accepted it.

use crate::image::{difference, Difference, Image};
use std::path::{Path, PathBuf};

#[derive(Clone, Copy, Debug)]
pub struct Tolerance {
    pub mean: f32,
    pub changed: f32,
}

impl Default for Tolerance {
    fn default() -> Self {
        // Loose enough for two software renderers to agree, tight enough that a wrong
        // colour, a missing object, or a moved camera cannot pass.
        Self { mean: 0.004, changed: 0.01 }
    }
}

#[derive(Debug)]
pub enum RefResult {
    Matches(Difference),
    Moved(Difference),
    Missing,
    SizeChanged,
}

impl RefResult {
    pub fn passed(&self) -> bool {
        matches!(self, Self::Matches(_))
    }
}

pub fn reference_path(refs_root: &Path, script: &str, shot: &str) -> PathBuf {
    refs_root.join(script).join(format!("{shot}.png"))
}

pub fn check(reference: &Path, capture: &Image, tolerance: Tolerance) -> RefResult {
    let Ok(expected) = Image::load(reference) else { return RefResult::Missing };
    match difference(&expected, capture) {
        Err(_) => RefResult::SizeChanged,
        Ok(d) if d.mean <= tolerance.mean && d.changed <= tolerance.changed => RefResult::Matches(d),
        Ok(d) => RefResult::Moved(d),
    }
}

pub fn accept(reference: &Path, capture: &Image) -> Result<(), String> {
    capture.save(reference)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn check_and_accept() {
        let dir = std::env::temp_dir().join(format!("vellmark-refs-{}", std::process::id()));
        let path = reference_path(&dir, "script", "shot");
        let image = Image::filled(16, 16, [90, 120, 60, 255]);
        assert!(matches!(check(&path, &image, Tolerance::default()), RefResult::Missing));
        accept(&path, &image).unwrap();
        assert!(check(&path, &image, Tolerance::default()).passed());
        let moved = Image::filled(16, 16, [90, 140, 60, 255]);
        assert!(matches!(check(&path, &moved, Tolerance::default()), RefResult::Moved(_)));
        let resized = Image::filled(8, 8, [90, 120, 60, 255]);
        assert!(matches!(check(&path, &resized, Tolerance::default()), RefResult::SizeChanged));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
