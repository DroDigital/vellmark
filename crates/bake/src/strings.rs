//! Authored text: `assets/text/<language>/*.ron`, each file a map from key to text.

use crate::{err, sorted_files, BakeError};
use std::collections::BTreeMap;
use std::path::Path;
use vellmark_content::StringTableBuilder;

pub struct BakedStrings {
    pub bytes: Vec<u8>,
    pub count: usize,
    pub keys: Vec<String>,
}

pub fn bake_language(dir: &Path) -> Result<BakedStrings, BakeError> {
    let mut builder = StringTableBuilder::new();
    let files = sorted_files(dir, "ron")?;
    if files.is_empty() {
        return err(format!("no string files found in {}", dir.display()));
    }
    for file in files {
        let source = std::fs::read_to_string(&file)
            .map_err(|e| BakeError(format!("cannot read {}: {e}", file.display())))?;
        let map: BTreeMap<String, String> = ron::from_str(&source)
            .map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
        for (key, text) in map {
            check_key(&key).map_err(|m| BakeError(format!("{}: {m}", file.display())))?;
            check_text(&text).map_err(|m| BakeError(format!("{}: `{key}`: {m}", file.display())))?;
            builder.add(&key, &text).map_err(|m| BakeError(format!("{}: {m}", file.display())))?;
        }
    }
    let keys: Vec<String> = builder.keys().map(str::to_owned).collect();
    let count = keys.len();
    Ok(BakedStrings { bytes: builder.finish(), count, keys })
}

fn check_key(key: &str) -> Result<(), String> {
    let ok = !key.is_empty()
        && key.split('.').all(|part| {
            !part.is_empty() && part.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_')
        });
    if ok { Ok(()) } else { Err(format!("string key `{key}` must be lowercase words joined by dots")) }
}

/// The base language is typeset in the 95 printable ASCII characters. A line break is
/// allowed in the data and is laid out by the interface.
fn check_text(text: &str) -> Result<(), String> {
    match text.chars().find(|&c| !(c == '\n' || (' '..='~').contains(&c))) {
        Some(c) => Err(format!("character {c:?} is outside printable ASCII")),
        None => Ok(()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keys_and_text_are_checked() {
        assert!(check_key("ui.title").is_ok());
        assert!(check_key("ui.Title").is_err());
        assert!(check_key("ui..title").is_err());
        assert!(check_text("Plain words.\nTwo lines.").is_ok());
        assert!(check_text("caf\u{e9}").is_err());
        assert!(check_text("tab\there").is_err());
    }
}
