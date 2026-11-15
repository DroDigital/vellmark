//! The credits, from `assets/data/credits.ron` and the crate index at the head of
//! `THIRD_PARTY_NOTICES.txt`.

use std::path::Path;

use serde::Deserialize;
use vellmark_content::Credits;

use crate::BakeError;

#[derive(Deserialize)]
struct Typeface {
    name: String,
    author: String,
    license: String,
}

#[derive(Deserialize)]
struct Authored {
    made_by: Vec<String>,
    typefaces: Vec<Typeface>,
}

/// `notices` is the text of `THIRD_PARTY_NOTICES.txt`, if it has been written.
pub fn bake_credits(file: &Path, notices: Option<&str>) -> Result<Credits, BakeError> {
    let text = std::fs::read_to_string(file).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
    let authored: Authored = ron::from_str(&text).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
    let mut crates = Vec::new();
    if let Some(notices) = notices {
        let index = notices.split("CRATES AND THEIR LICENSES").nth(1).and_then(|rest| rest.split("LICENSE AND NOTICE TEXTS").next()).unwrap_or_default();
        for line in index.lines() {
            if let Some((name, license)) = line.split_once(": ") {
                crates.push((name.trim().to_string(), license.trim().to_string()));
            }
        }
    }
    Ok(Credits { made_by: authored.made_by, typefaces: authored.typefaces.into_iter().map(|t| (t.name, t.author, t.license)).collect(), crates })
}
