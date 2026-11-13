//! Checks that read the source tree.
//!
//! **No player-facing text in code.** Text reaches the player only through a
//! [`vellmark_content::StringId`], written `sid!("key")`. The check here finds every
//! such key in the game's crates and requires it to exist in the baked table, so a
//! missing or misspelt key fails the build's tests and never reaches a player.
//! Development tools draw their own text through `dev_text`, which exists only in
//! modules named `dev` and is compiled out of shipping builds; the check refuses a
//! `dev_text` call anywhere else.

use std::path::{Path, PathBuf};

fn rust_files(dir: &Path, out: &mut Vec<PathBuf>) {
    let Ok(read) = std::fs::read_dir(dir) else { return };
    let mut entries: Vec<PathBuf> = read.filter_map(|e| e.ok().map(|e| e.path())).collect();
    entries.sort();
    for path in entries {
        if path.is_dir() {
            rust_files(&path, out);
        } else if path.extension().and_then(|e| e.to_str()) == Some("rs") {
            out.push(path);
        }
    }
}

/// The crates whose code runs in the shipped game.
pub const GAME_CRATES: &[&str] =
    &["app", "platform", "storefront", "gpu", "render", "world", "sim", "ui", "audio", "content"];

pub struct KeyUse {
    pub key: String,
    pub file: PathBuf,
    pub line: usize,
}

/// Every `sid!("...")` in the game's crates.
pub fn string_keys_in_code(repo: &Path) -> Vec<KeyUse> {
    let mut uses = Vec::new();
    for name in GAME_CRATES {
        let mut files = Vec::new();
        rust_files(&repo.join("crates").join(name).join("src"), &mut files);
        for file in files {
            let Ok(text) = std::fs::read_to_string(&file) else { continue };
            for (i, line) in text.lines().enumerate() {
                // Unit tests sit at the end of a file and may use keys that do not exist.
                if line.trim() == "#[cfg(test)]" {
                    break;
                }
                let mut rest = line.split("//").next().unwrap_or("");
                while let Some(at) = rest.find("sid!(\"") {
                    rest = &rest[at + 6..];
                    if let Some(end) = rest.find('"') {
                        uses.push(KeyUse { key: rest[..end].to_owned(), file: file.clone(), line: i + 1 });
                        rest = &rest[end..];
                    }
                }
            }
        }
    }
    uses
}

/// Every call to `dev_text` outside a `dev` module. There must be none.
pub fn dev_text_outside_dev_modules(repo: &Path) -> Vec<(PathBuf, usize)> {
    let mut found = Vec::new();
    for name in GAME_CRATES {
        let mut files = Vec::new();
        rust_files(&repo.join("crates").join(name).join("src"), &mut files);
        for file in files {
            let in_dev = file.components().any(|c| c.as_os_str() == "dev")
                || file.file_stem().is_some_and(|s| s == "dev");
            if in_dev {
                continue;
            }
            let Ok(text) = std::fs::read_to_string(&file) else { continue };
            for (i, line) in text.lines().enumerate() {
                let code = line.split("//").next().unwrap_or("");
                // The definition itself lives in the interface crate, behind the `dev` feature.
                if code.contains("dev_text(") && !code.contains("fn dev_text(") {
                    found.push((file.clone(), i + 1));
                }
            }
        }
    }
    found
}
