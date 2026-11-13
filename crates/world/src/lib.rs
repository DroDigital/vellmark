//! The world as it is loaded: the terrain's tiles, and what can be asked of them.
//!
//! The Showcase loads its one area whole. The full game streams the same tiles in and
//! out around each player; the questions asked of them stay the same.

use std::collections::HashMap;

use vellmark_content::{AssetKind, Pack, TerrainTile, TileCoord};

/// Every terrain tile of a place, at every level.
#[derive(Default)]
pub struct Terrain {
    tiles: HashMap<TileCoord, TerrainTile>,
    /// The levels present, finest first.
    levels: Vec<u8>,
}

impl Terrain {
    /// Reads every terrain tile in the pack.
    pub fn from_pack(pack: &Pack) -> Result<Self, String> {
        let mut terrain = Self::default();
        for (_, bytes) in pack.of_kind(AssetKind::Tile) {
            let tile = TerrainTile::from_bytes(bytes)?;
            terrain.insert(tile);
        }
        Ok(terrain)
    }

    pub fn insert(&mut self, tile: TerrainTile) {
        if !self.levels.contains(&tile.coord.level) {
            self.levels.push(tile.coord.level);
            self.levels.sort_unstable();
        }
        self.tiles.insert(tile.coord, tile);
    }

    pub fn is_empty(&self) -> bool {
        self.tiles.is_empty()
    }

    pub fn tiles(&self) -> impl Iterator<Item = &TerrainTile> {
        self.tiles.values()
    }

    pub fn tile(&self, coord: TileCoord) -> Option<&TerrainTile> {
        self.tiles.get(&coord)
    }

    /// The finest tile that holds a point.
    fn finest_at(&self, x: f32, z: f32) -> Option<&TerrainTile> {
        self.levels
            .iter()
            .find_map(|&level| self.tiles.get(&TileCoord::containing(x, z, level)))
    }

    /// The height of the ground at a point, from the finest tile that holds it.
    pub fn height_at(&self, x: f32, z: f32) -> Option<f32> {
        self.finest_at(x, z).map(|t| t.height_at(x, z))
    }

    /// The height of the water's surface at a point, if there is water there.
    pub fn water_at(&self, x: f32, z: f32) -> Option<f32> {
        let tile = self.finest_at(x, z)?;
        let (x0, z0) = tile.coord.origin();
        let i =
            (((x - x0) / tile.spacing).round() as usize).min(vellmark_content::TILE_SAMPLES - 1);
        let j =
            (((z - z0) / tile.spacing).round() as usize).min(vellmark_content::TILE_SAMPLES - 1);
        let w = tile.water[TerrainTile::index(i, j)];
        (!w.is_nan()).then_some(w)
    }
}

impl std::fmt::Debug for Terrain {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Terrain({} tiles)", self.tiles.len())
    }
}

impl vellmark_sim::Ground for Terrain {
    fn height(&self, x: f32, z: f32) -> Option<f32> {
        self.height_at(x, z)
    }

    fn water(&self, x: f32, z: f32) -> Option<f32> {
        self.water_at(x, z)
    }

    /// The finest level's tiles: the ground a body walks on.
    fn patches(&self) -> Vec<vellmark_sim::HeightPatch> {
        let Some(&finest) = self.levels.first() else {
            return Vec::new();
        };
        let mut tiles: Vec<&TerrainTile> = self
            .tiles
            .values()
            .filter(|t| t.coord.level == finest)
            .collect();
        tiles.sort_by_key(|t| (t.coord.x, t.coord.z));
        tiles
            .into_iter()
            .map(|t| {
                let (x0, z0) = t.coord.origin();
                vellmark_sim::HeightPatch {
                    origin: glam::Vec2::new(x0, z0),
                    spacing: t.spacing,
                    samples: vellmark_content::TILE_SAMPLES,
                    heights: t.heights.clone(),
                }
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_finest_tile_answers() {
        let mut terrain = Terrain::default();
        let mut coarse = TerrainTile::new(TileCoord {
            level: 3,
            x: 0,
            z: 0,
        });
        coarse.heights.iter_mut().for_each(|h| *h = 10.0);
        let mut fine = TerrainTile::new(TileCoord {
            level: 0,
            x: 0,
            z: 0,
        });
        fine.heights.iter_mut().for_each(|h| *h = 2.0);
        terrain.insert(coarse);
        terrain.insert(fine);
        assert_eq!(terrain.height_at(10.0, 10.0), Some(2.0));
        assert_eq!(terrain.height_at(100.0, 100.0), Some(10.0));
        assert_eq!(terrain.height_at(-1.0, 0.0), None);
    }
}
