//! Data formats and the pack reader.
//!
//! RON and plain text are authoring formats only. The bake writes the binary formats
//! defined here, and the game reads nothing else at runtime.

pub mod levels;
pub mod camera_path;
pub mod glyph_atlas;
pub mod grade;
pub mod id;
pub mod pack;
pub mod scene;
pub mod strings;
pub mod terrain;
pub mod tile;

pub use camera_path::{CameraPath, Keyframe};
pub use grade::{Grade, GradeKey};
pub use levels::{MeshLevels, MESH_LEVELS_KEY};
pub use glyph_atlas::{glyph_index, Face, FaceInfo, GlyphAtlas, GlyphInfo, FACES, GLYPHS_PER_FACE, TONE_STEPS};
pub use id::{fnv1a64, AssetId, AssetKind, StringId};
pub use scene::{Breakable, Lexicon, Light, Material, MaterialSet, Mesh, Object, PageEntry, Scene, SolidBox, TextPages, Vertex};
pub use pack::{Pack, PackError, PackWriter, PACK_VERSION};
pub use strings::{StringTable, StringTableBuilder};
pub use terrain::{Ground, TerrainTile, TERRAIN_LAYER, TILE_SAMPLES};
pub use tile::{TileCoord, TILE_METRES};
