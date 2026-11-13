//! Data formats and the pack reader.
//!
//! RON and plain text are authoring formats only. The bake writes the binary formats
//! defined here, and the game reads nothing else at runtime.

pub mod camera_path;
pub mod glyph_atlas;
pub mod grade;
pub mod id;
pub mod levels;
pub mod pack;
pub mod readables;
pub mod records;
pub mod scene;
pub mod strings;
pub mod terrain;
pub mod tile;
pub mod words;

pub use camera_path::{CameraPath, Keyframe};
pub use glyph_atlas::{
    FACES, Face, FaceInfo, GLYPHS_PER_FACE, GlyphAtlas, GlyphInfo, TONE_STEPS, glyph_index,
};
pub use grade::{Grade, GradeKey};
pub use id::{AssetId, AssetKind, StringId, fnv1a64};
pub use levels::{MESH_LEVELS_KEY, MeshLevels};
pub use pack::{PACK_VERSION, Pack, PackError, PackWriter};
pub use readables::{
    InscribedThing, InscribedThings, Readable, Readables, readables_key, targets_key,
};
pub use records::{TREE_RECORDS_KEY, TreeRecord, TreeRecords};
pub use scene::{
    Breakable, Lexicon, Light, Material, MaterialSet, Mesh, Object, PageEntry, Scene, SolidBox,
    TextPages, Vertex,
};
pub use strings::{StringTable, StringTableBuilder};
pub use terrain::{Ground, TERRAIN_LAYER, TILE_SAMPLES, TerrainTile};
pub use tile::{TILE_METRES, TileCoord};
pub use words::{Compound, EffectLook, Outcome, Typo, WORDS_KEY, Word, WordTable};
