//! Loading the packs and the strings, and saying so in words when something is missing.

use crate::paths;
use vellmark_content::{fnv1a64, sid, AssetId, AssetKind, GlyphAtlas, Pack, StringId, StringTable};
use vellmark_gpu::GpuError;

static SYSTEM_STRINGS: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/strings.bin"));

pub struct Content {
    pub base: Pack,
    pub strings: StringTable,
    pub glyphs: GlyphAtlas,
}

/// The strings compiled into the executable, for messages shown before the packs load.
pub fn system_strings() -> StringTable {
    StringTable::from_bytes(SYSTEM_STRINGS).expect("embedded strings are valid")
}

fn string_table_id() -> AssetId {
    AssetId(fnv1a64(b"strings.en"))
}

pub fn load_content() -> Result<Content, String> {
    // Development builds bake on launch, so that the packs always match the data.
    #[cfg(feature = "dev")]
    if let Some(root) = paths::repo_root() {
        let inputs = vellmark_bake::BakeInputs { assets: root.join("assets"), third_party: root.join("third_party"), cache: Some(root.join("build/bake-cache")) };
        vellmark_bake::bake_to_dir(&inputs, &root.join("build/packs")).map_err(|e| e.to_string())?;
    }
    let mut tried = Vec::new();
    for dir in paths::pack_dirs() {
        let path = dir.join("base.vpk");
        match std::fs::read(&path) {
            Ok(bytes) => {
                let base = Pack::from_bytes(bytes).map_err(|e| format!("{}: {e}", path.display()))?;
                let table = base
                    .get(AssetKind::StringTable, string_table_id())
                    .ok_or_else(|| format!("{}: no string table", path.display()))?;
                let strings = StringTable::from_bytes(table).map_err(|e| format!("{}: {e}", path.display()))?;
                let glyphs = base
                    .get(AssetKind::GlyphAtlas, AssetId(fnv1a64(b"glyphs.atlas")))
                    .ok_or_else(|| format!("{}: no glyph atlas", path.display()))?;
                let glyphs = GlyphAtlas::from_bytes(glyphs).map_err(|e| format!("{}: {e}", path.display()))?;
                return Ok(Content { base, strings, glyphs });
            }
            Err(e) => tried.push(format!("{}: {e}", path.display())),
        }
    }
    Err(format!("no pack found. Tried:\n  {}", tried.join("\n  ")))
}

/// The message a player sees for a GPU failure.
pub fn gpu_error_string(error: &GpuError) -> StringId {
    match error {
        GpuError::NoAdapter(_) => sid!("sys.gpu.none"),
        GpuError::TooOld(_) => sid!("sys.gpu.too_old"),
        GpuError::Failed(_) => sid!("sys.gpu.failed"),
    }
}

/// Tells the player, in a system dialog, why the game cannot run. The detail goes to
/// the log and to standard error.
pub fn fatal(strings: &StringTable, message: StringId, detail: &str) {
    crate::log::line(&format!("fatal: {detail}"));
    vellmark_platform::message_box(&strings.text(sid!("sys.window_title")), &strings.text(message));
}
