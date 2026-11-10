//! Data formats and the pack reader.
//!
//! RON and plain text are authoring formats only. The bake writes the binary formats
//! defined here, and the game reads nothing else at runtime.

pub mod camera_path;
pub mod id;
pub mod pack;
pub mod strings;
pub mod tile;

pub use camera_path::{CameraPath, Keyframe};
pub use id::{fnv1a64, AssetId, AssetKind, StringId};
pub use pack::{Pack, PackError, PackWriter, PACK_VERSION};
pub use strings::{StringTable, StringTableBuilder};
pub use tile::{TileCoord, TILE_METRES};
