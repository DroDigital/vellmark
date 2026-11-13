//! Terrain tiles: the ground of one tile as a grid of heights, with where water stands
//! and what the ground is at each sample.
//!
//! A tile at level `n` is 64 m × 2^n across and holds 65 × 65 samples, so neighbouring
//! tiles share their edge samples exactly and the ground never opens a seam. Level 0 is
//! a sample a metre; the far ring uses coarser levels.
//!
//! ```text
//! magic "VLMKTER1", level u8, 3 bytes zero, x i16, z i16,
//! samples u32 (65), spacing f32 (metres), lowest f32, highest f32,
//! heights:  samples² f32, row by row from the tile's least x and z, x fastest
//! water:    samples² f32, the height of the water's surface, NaN where it is dry
//! ground:   samples² u8, a `Ground`
//! sky:      samples² u8, the share of the sky seen from the sample, 0 to 255, cosine
//!           weighted: 255 on open ground, less in a hollow or at the foot of a slope;
//!           then zero bytes to a multiple of four
//! ```

use crate::tile::TileCoord;

const MAGIC: &[u8; 8] = b"VLMKTER2";

/// Samples along each side of every terrain tile.
pub const TILE_SAMPLES: usize = 65;

/// The layer of a tile's asset ID that holds its terrain.
pub const TERRAIN_LAYER: u8 = 0;

/// What the ground is at a sample. The number is part of the format and never reused.
/// Slope and height still choose the rock and scree on top of this when it is shaded.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
pub enum Ground {
    /// Pasture and verge.
    Grass = 0,
    /// Barley.
    Field = 1,
    /// A beaten road or path.
    Road = 2,
    /// Bare earth: the green's trodden middle, a farmyard, a grave.
    Earth = 3,
    /// Mud and shingle at the water's edge.
    Bank = 4,
    /// The bed of a river or a pond.
    Bed = 5,
    /// The floor of a wood. Trees stand here.
    Forest = 6,
    /// The Erasure: bare paper, far to the north.
    Erasure = 7,
}

impl Ground {
    pub const ALL: [Ground; 8] =
        [Ground::Grass, Ground::Field, Ground::Road, Ground::Earth, Ground::Bank, Ground::Bed, Ground::Forest, Ground::Erasure];

    pub fn from_u8(v: u8) -> Option<Self> {
        Self::ALL.get(v as usize).copied()
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct TerrainTile {
    pub coord: TileCoord,
    /// Metres between samples: the tile's side over 64.
    pub spacing: f32,
    pub heights: Vec<f32>,
    /// The water's surface at each sample, NaN where it is dry.
    pub water: Vec<f32>,
    pub ground: Vec<Ground>,
    /// The share of the sky seen from each sample, 0 to 255.
    pub sky: Vec<u8>,
}

impl TerrainTile {
    pub fn new(coord: TileCoord) -> Self {
        let n = TILE_SAMPLES * TILE_SAMPLES;
        Self {
            coord,
            spacing: coord.side() / (TILE_SAMPLES - 1) as f32,
            heights: vec![0.0; n],
            water: vec![f32::NAN; n],
            ground: vec![Ground::Grass; n],
            sky: vec![255; n],
        }
    }

    pub fn index(i: usize, j: usize) -> usize {
        j * TILE_SAMPLES + i
    }

    /// The world position of a sample, x and z.
    pub fn sample_position(&self, i: usize, j: usize) -> (f32, f32) {
        let (x0, z0) = self.coord.origin();
        (x0 + i as f32 * self.spacing, z0 + j as f32 * self.spacing)
    }

    /// The height of the ground at a world position inside the tile, between samples
    /// as the two triangles of each cell are drawn: split from the least corner to the
    /// greatest.
    pub fn height_at(&self, x: f32, z: f32) -> f32 {
        let (x0, z0) = self.coord.origin();
        let last = (TILE_SAMPLES - 1) as f32;
        let u = ((x - x0) / self.spacing).clamp(0.0, last);
        let v = ((z - z0) / self.spacing).clamp(0.0, last);
        let (i, j) = ((u.floor() as usize).min(TILE_SAMPLES - 2), (v.floor() as usize).min(TILE_SAMPLES - 2));
        let (fu, fv) = (u - i as f32, v - j as f32);
        let h = |a: usize, b: usize| self.heights[Self::index(a, b)];
        if fu >= fv {
            h(i, j) + (h(i + 1, j) - h(i, j)) * fu + (h(i + 1, j + 1) - h(i + 1, j)) * fv
        } else {
            h(i, j) + (h(i + 1, j + 1) - h(i, j + 1)) * fu + (h(i, j + 1) - h(i, j)) * fv
        }
    }

    pub fn lowest(&self) -> f32 {
        self.heights.iter().copied().fold(f32::INFINITY, f32::min)
    }

    pub fn highest(&self) -> f32 {
        self.heights.iter().copied().fold(f32::NEG_INFINITY, f32::max)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let n = TILE_SAMPLES * TILE_SAMPLES;
        let mut out = Vec::with_capacity(40 + n * 9 + 4);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&[self.coord.level, 0, 0, 0]);
        out.extend_from_slice(&self.coord.x.to_le_bytes());
        out.extend_from_slice(&self.coord.z.to_le_bytes());
        out.extend_from_slice(&(TILE_SAMPLES as u32).to_le_bytes());
        for v in [self.spacing, self.lowest(), self.highest()] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        for v in self.heights.iter().chain(&self.water) {
            // One NaN, always, so that the same tile is always the same bytes.
            let v = if v.is_nan() { f32::NAN } else { *v };
            out.extend_from_slice(&v.to_le_bytes());
        }
        out.extend(self.ground.iter().map(|g| *g as u8));
        out.extend_from_slice(&self.sky);
        while out.len() % 4 != 0 {
            out.push(0);
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        let bad = |what: &str| format!("terrain tile: {what}");
        if bytes.len() < 32 || &bytes[..8] != MAGIC {
            return Err(bad("not a terrain tile"));
        }
        let u32_at = |at: usize| u32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let f32_at = |at: usize| f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let coord = TileCoord {
            level: bytes[8],
            x: i16::from_le_bytes([bytes[12], bytes[13]]),
            z: i16::from_le_bytes([bytes[14], bytes[15]]),
        };
        if u32_at(16) as usize != TILE_SAMPLES {
            return Err(bad("wrong number of samples"));
        }
        let n = TILE_SAMPLES * TILE_SAMPLES;
        let start = 32;
        if bytes.len() < start + n * 10 {
            return Err(bad("too short"));
        }
        let heights = (0..n).map(|k| f32_at(start + k * 4)).collect();
        let water = (0..n).map(|k| f32_at(start + (n + k) * 4)).collect();
        let ground = bytes[start + n * 8..start + n * 9]
            .iter()
            .map(|&g| Ground::from_u8(g).ok_or_else(|| bad("unknown ground")))
            .collect::<Result<_, _>>()?;
        let sky = bytes[start + n * 9..start + n * 10].to_vec();
        Ok(Self { coord, spacing: f32_at(20), heights, water, ground, sky })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_tile_reads_back_as_it_was_written() {
        let mut tile = TerrainTile::new(TileCoord { level: 3, x: -2, z: 1 });
        for (k, h) in tile.heights.iter_mut().enumerate() {
            *h = (k as f32 * 0.37).sin() * 20.0;
        }
        tile.water[100] = 3.5;
        tile.ground[7] = Ground::Erasure;
        let bytes = tile.to_bytes();
        assert_eq!(bytes.len() % 4, 0);
        let back = TerrainTile::from_bytes(&bytes).unwrap();
        assert_eq!(back.coord, tile.coord);
        assert_eq!(back.spacing, 8.0);
        assert_eq!(back.heights, tile.heights);
        assert_eq!(back.ground, tile.ground);
        assert_eq!(back.water[100], 3.5);
        assert!(back.water[101].is_nan());
        assert_eq!(back.to_bytes(), bytes);
    }

    #[test]
    fn height_between_samples_follows_the_drawn_triangles() {
        let mut tile = TerrainTile::new(TileCoord { level: 0, x: 0, z: 0 });
        tile.heights[TerrainTile::index(1, 0)] = 1.0;
        tile.heights[TerrainTile::index(1, 1)] = 2.0;
        assert_eq!(tile.height_at(0.0, 0.0), 0.0);
        assert_eq!(tile.height_at(1.0, 1.0), 2.0);
        assert!((tile.height_at(0.5, 0.25) - (0.5 + 0.25)).abs() < 1e-6);
        // Off the tile, the edge is held.
        assert_eq!(tile.height_at(-5.0, 0.0), 0.0);
    }
}
