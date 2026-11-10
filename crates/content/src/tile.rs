//! The tile layout the full game streams. The Showcase loads its one area whole, but it
//! is baked into the same tiles, so streaming is added later without rebaking.

use crate::id::AssetId;

/// Side of a level-0 tile in metres. Each coarser level doubles it.
pub const TILE_METRES: f32 = 64.0;

/// Address of one tile. `x` and `z` count tiles of this level from the world origin,
/// and may be negative.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct TileCoord {
    pub level: u8,
    pub x: i16,
    pub z: i16,
}

impl TileCoord {
    pub fn containing(world_x: f32, world_z: f32, level: u8) -> Self {
        let side = Self::side_of(level);
        Self {
            level,
            x: (world_x / side).floor() as i16,
            z: (world_z / side).floor() as i16,
        }
    }

    pub fn side_of(level: u8) -> f32 {
        TILE_METRES * (1u32 << level) as f32
    }

    pub fn side(&self) -> f32 {
        Self::side_of(self.level)
    }

    /// World position of the corner with the smallest x and z.
    pub fn origin(&self) -> (f32, f32) {
        (self.x as f32 * self.side(), self.z as f32 * self.side())
    }

    /// The permanent ID of a tile's asset on a given layer, such as terrain or text
    /// pages. Tiles are addressed by where they are, never by a name.
    pub fn asset_id(&self, layer: u8) -> AssetId {
        let v = 0x7400_0000_0000_0000u64
            | (layer as u64) << 48
            | (self.level as u64) << 40
            | (self.x as u16 as u64) << 16
            | (self.z as u16 as u64);
        AssetId(v)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn containing_tile_handles_negative_ground() {
        assert_eq!(TileCoord::containing(1.0, 1.0, 0), TileCoord { level: 0, x: 0, z: 0 });
        assert_eq!(TileCoord::containing(-1.0, 65.0, 0), TileCoord { level: 0, x: -1, z: 1 });
        assert_eq!(TileCoord::containing(-1.0, 65.0, 1), TileCoord { level: 1, x: -1, z: 0 });
        assert_eq!(TileCoord { level: 1, x: -1, z: 0 }.origin(), (-128.0, 0.0));
    }

    #[test]
    fn tile_ids_are_distinct() {
        let mut seen = std::collections::BTreeSet::new();
        for level in 0..3u8 {
            for x in -4..4i16 {
                for z in -4..4i16 {
                    for layer in 0..3u8 {
                        assert!(seen.insert(TileCoord { level, x, z }.asset_id(layer)));
                    }
                }
            }
        }
    }
}
