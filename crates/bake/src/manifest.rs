//! `third_party/manifest.ron`: every third-party file, with where it came from, who made
//! it, its license, when that was checked, and its SHA-256. The bake fails on any file
//! that is not listed, has changed, or carries a license the rules do not allow.

use crate::sha256::{hex, sha256};
use crate::{BakeError, err};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// Meshes, textures, animation, and sound. CC0 only.
    Art,
    /// Typefaces. SIL Open Font License only.
    Font,
    /// A license or notice text kept beside the files it covers.
    LicenseText,
    /// Shader or code references that are ported. Permissive licenses only.
    Code,
}

#[derive(Debug, Deserialize)]
pub struct Entry {
    /// Path below `third_party/`, with forward slashes.
    pub path: String,
    pub kind: Kind,
    pub source_url: String,
    pub author: String,
    /// SPDX identifier.
    pub license: String,
    /// The day the license was read at its source, as YYYY-MM-DD.
    pub date_checked: String,
    pub sha256: String,
}

#[derive(Debug, Deserialize)]
pub struct Manifest {
    pub files: Vec<Entry>,
}

const CODE_LICENSES: &[&str] = &[
    "MIT",
    "Apache-2.0",
    "BSD-2-Clause",
    "BSD-3-Clause",
    "Zlib",
    "ISC",
    "BSL-1.0",
    "Unicode-3.0",
    "CC0-1.0",
    "MPL-2.0",
];

/// Folders and files under `third_party/` that are not shipped inputs: the owner's
/// drop box, the AI quarantine, and the lists themselves.
const NOT_INPUTS: &[&str] = &[
    "manifest.ron",
    "DOWNLOADS.md",
    "ai_provenance.ron",
    "README.md",
    ".gitattributes",
];
const NOT_INPUT_DIRS: &[&str] = &["inbox", "quarantine"];

pub fn license_allowed(kind: Kind, license: &str) -> bool {
    match kind {
        Kind::Art => license == "CC0-1.0",
        Kind::Font => license == "OFL-1.1",
        Kind::Code => CODE_LICENSES.contains(&license),
        Kind::LicenseText => {
            license == "OFL-1.1" || license == "CC0-1.0" || CODE_LICENSES.contains(&license)
        }
    }
}

fn valid_date(d: &str) -> bool {
    let b = d.as_bytes();
    b.len() == 10
        && b[4] == b'-'
        && b[7] == b'-'
        && b.iter()
            .enumerate()
            .all(|(i, c)| i == 4 || i == 7 || c.is_ascii_digit())
}

fn walk(root: &Path, dir: &Path, out: &mut Vec<String>) -> Result<(), BakeError> {
    let read = std::fs::read_dir(dir)
        .map_err(|e| BakeError(format!("cannot read {}: {e}", dir.display())))?;
    for entry in read {
        let path: PathBuf = entry.map_err(|e| BakeError(e.to_string()))?.path();
        let rel = path
            .strip_prefix(root)
            .unwrap()
            .to_string_lossy()
            .replace('\\', "/");
        if path.is_dir() {
            if dir == root && NOT_INPUT_DIRS.contains(&rel.as_str()) {
                continue;
            }
            walk(root, &path, out)?;
        } else if !(dir == root && NOT_INPUTS.contains(&rel.as_str())) {
            out.push(rel);
        }
    }
    Ok(())
}

/// Checks the folder against its manifest. Returns how many files were checked.
/// A missing `third_party` folder is valid: nothing third-party is in use.
pub fn validate(third_party: &Path) -> Result<usize, BakeError> {
    if !third_party.exists() {
        return Ok(0);
    }
    let manifest_path = third_party.join("manifest.ron");
    let source = std::fs::read_to_string(&manifest_path)
        .map_err(|e| BakeError(format!("cannot read {}: {e}", manifest_path.display())))?;
    let manifest: Manifest = ron::from_str(&source)
        .map_err(|e| BakeError(format!("{}: {e}", manifest_path.display())))?;

    let mut problems = Vec::new();
    let mut listed: BTreeMap<&str, &Entry> = BTreeMap::new();
    for e in &manifest.files {
        if listed.insert(e.path.as_str(), e).is_some() {
            problems.push(format!("{}: listed twice", e.path));
        }
        if !license_allowed(e.kind, &e.license) {
            problems.push(format!(
                "{}: license `{}` is not allowed for {:?}",
                e.path, e.license, e.kind
            ));
        }
        if !e.source_url.starts_with("https://") {
            problems.push(format!("{}: source_url must be an https address", e.path));
        }
        if e.author.trim().is_empty() {
            problems.push(format!("{}: author is empty", e.path));
        }
        if !valid_date(&e.date_checked) {
            problems.push(format!("{}: date_checked must be YYYY-MM-DD", e.path));
        }
    }

    let mut on_disk = Vec::new();
    walk(third_party, third_party, &mut on_disk)?;
    on_disk.sort();
    for rel in &on_disk {
        match listed.remove(rel.as_str()) {
            None => problems.push(format!("{rel}: on disk but not in the manifest")),
            Some(e) => {
                let bytes = std::fs::read(third_party.join(rel))
                    .map_err(|e| BakeError(format!("cannot read {rel}: {e}")))?;
                let actual = hex(&sha256(&bytes));
                if actual != e.sha256.to_ascii_lowercase() {
                    problems.push(format!(
                        "{rel}: SHA-256 is {actual}, the manifest says {}",
                        e.sha256
                    ));
                }
            }
        }
    }
    for rel in listed.keys() {
        problems.push(format!("{rel}: in the manifest but not on disk"));
    }

    if problems.is_empty() {
        Ok(on_disk.len())
    } else {
        err(format!(
            "third_party/manifest.ron does not match the folder:\n  {}",
            problems.join("\n  ")
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn license_rules() {
        assert!(license_allowed(Kind::Art, "CC0-1.0"));
        assert!(!license_allowed(Kind::Art, "CC-BY-4.0"));
        assert!(!license_allowed(Kind::Art, "MIT"));
        assert!(license_allowed(Kind::Font, "OFL-1.1"));
        assert!(!license_allowed(Kind::Font, "CC0-1.0"));
        assert!(license_allowed(Kind::Code, "MIT"));
        assert!(!license_allowed(Kind::Code, "GPL-3.0-only"));
        assert!(!license_allowed(Kind::Art, "CC-BY-NC-4.0"));
    }

    #[test]
    fn dates() {
        assert!(valid_date("2026-11-10"));
        assert!(!valid_date("10/11/2026"));
        assert!(!valid_date("2026-11-1"));
    }

    fn temp(name: &str) -> PathBuf {
        let dir =
            std::env::temp_dir().join(format!("vellmark-manifest-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("fonts")).unwrap();
        dir
    }

    fn entry(path: &str, bytes: &[u8], license: &str) -> String {
        format!(
            "(path: \"{path}\", kind: Font, source_url: \"https://example.org/f\", author: \"A\", license: \"{license}\", date_checked: \"2026-11-10\", sha256: \"{}\")",
            hex(&sha256(bytes))
        )
    }

    #[test]
    fn folder_must_match_manifest() {
        let dir = temp("match");
        std::fs::write(dir.join("fonts/a.ttf"), b"font").unwrap();
        std::fs::write(dir.join("DOWNLOADS.md"), b"list").unwrap();
        std::fs::create_dir_all(dir.join("inbox")).unwrap();
        std::fs::write(dir.join("inbox/pending.zip"), b"zip").unwrap();

        std::fs::write(
            dir.join("manifest.ron"),
            format!("(files: [{}])", entry("fonts/a.ttf", b"font", "OFL-1.1")),
        )
        .unwrap();
        assert_eq!(validate(&dir).unwrap(), 1);

        // Changed on disk.
        std::fs::write(dir.join("fonts/a.ttf"), b"other").unwrap();
        assert!(validate(&dir).unwrap_err().0.contains("SHA-256"));
        std::fs::write(dir.join("fonts/a.ttf"), b"font").unwrap();

        // Unlisted file.
        std::fs::write(dir.join("fonts/b.ttf"), b"x").unwrap();
        assert!(
            validate(&dir)
                .unwrap_err()
                .0
                .contains("not in the manifest")
        );
        std::fs::remove_file(dir.join("fonts/b.ttf")).unwrap();

        // Listed but missing, and a banned license.
        std::fs::write(
            dir.join("manifest.ron"),
            format!(
                "(files: [{}, {}])",
                entry("fonts/a.ttf", b"font", "GPL-3.0-only"),
                entry("fonts/c.ttf", b"", "OFL-1.1")
            ),
        )
        .unwrap();
        let message = validate(&dir).unwrap_err().0;
        assert!(message.contains("not allowed"));
        assert!(message.contains("not on disk"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
