//! Grass: real blades, built on the GPU from the terrain they stand on.
//!
//! The ground near each view is cut into square patches, and every patch with grass on
//! it is one instance in the scene's one instance buffer, culled per view in compute
//! like everything else. A patch has no mesh of its own: the vertex shader makes each
//! blade from the blade's number, reading the terrain's heights and ground to place it,
//! and bends it by the wind and by whoever walks through it. Nearer patches are drawn
//! with more blades. The blades of every level of detail are the first of one sequence,
//! so a patch that drops to fewer blades keeps the ones it had, and each blade fades out
//! by its own distance rather than when its patch changes level: nothing pops.
//!
//! Up close each blade is written as a line of characters, in `shaders/opaque.wgsl`.

use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use std::collections::HashMap;
use vellmark_content::{Ground, TerrainTile, TILE_SAMPLES};
use vellmark_gpu::{wgpu, Gpu};

use crate::world_gpu::InstanceGpu;

/// The side of a patch, in metres. A level-0 tile is sixteen patches across.
pub const PATCH: f32 = 4.0;
/// How far from a view grass is drawn. Beyond this the terrain's own colour carries it.
pub const REACH: f32 = 72.0;
/// The levels of detail.
pub const LEVELS: usize = 4;
/// Vertices and indices of one blade: three pairs up its length and a tip.
pub const BLADE_VERTICES: u32 = 7;
pub const BLADE_INDICES: u32 = 15;
/// The most things that may push the grass aside at once.
pub const MAX_PUSHERS: usize = 8;

/// How dense the grass is. The governor and the presets change these; the look holds.
#[derive(Clone, Copy, Debug)]
pub struct GrassTuning {
    /// Blades per square metre at full density.
    pub density: f32,
    /// Within this distance of the eye, in metres, the grass is at full density. Beyond
    /// it, the count falls with the square of the distance and the blades widen to keep
    /// the field's cover, so each blade stays about the same width on screen.
    pub near: f32,
}

impl Default for GrassTuning {
    fn default() -> Self {
        Self { density: 160.0, near: 8.0 }
    }
}

impl GrassTuning {
    /// The blades a patch holds at full density: the most any level draws.
    pub fn blades(&self) -> u32 {
        (self.density * PATCH * PATCH).ceil() as u32
    }

    /// The blades drawn at each level of detail. Each level holds a quarter of the last.
    pub fn level_blades(&self) -> [u32; LEVELS] {
        let full = self.blades();
        std::array::from_fn(|l| full.div_ceil(1 << (2 * l)).max(1))
    }

    /// The level of detail for a patch whose nearest point is `distance` from the
    /// nearest eye: the fewest blades that still hold every blade the density asks for
    /// anywhere in the patch.
    pub fn level_at(&self, distance: f32) -> usize {
        let mut level = 0;
        while level + 1 < LEVELS && distance >= self.near * (1u32 << (level + 1)) as f32 {
            level += 1;
        }
        level
    }
}

/// What the renderer knows about one patch of ground.
#[derive(Clone, Copy, Debug)]
struct PatchInfo {
    /// The texture layer of the level-0 tile it lies in.
    layer: u32,
    low: f32,
    high: f32,
    /// Whether any of it is barley, which stands taller.
    tall: bool,
}

/// One patch chosen for a frame.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Chosen {
    pub cell: (i32, i32),
    pub level: usize,
}

/// Matches `GrassUniform` in `shaders/opaque.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct GrassUniform {
    /// xy: the wind's direction over the ground. z: its strength, 0 to 1. w: how fast its
    /// gusts travel, in metres a second.
    pub wind: [f32; 4],
    /// x: the near distance. y: blades per square metre there. z: the side of a patch.
    /// w: the reach.
    pub shape: [f32; 4],
    /// Each pusher's place and radius.
    pub pushers: [[f32; 4]; MAX_PUSHERS],
    /// x: how many pushers. y: the blades at full density.
    pub counts: [u32; 4],
}

/// The grass of a terrain: which patches have any, and the shared blade indices.
pub struct GrassField {
    patches: HashMap<(i32, i32), PatchInfo>,
    pub indices: wgpu::Buffer,
    pub tuning: GrassTuning,
}

/// The tallest blade of each kind, in metres, with room for its lean.
const GRASS_TALL: f32 = 0.7;
const BARLEY_TALL: f32 = 1.25;

/// The index list of `blades` blades: five triangles each, wound up the blade.
fn blade_indices(blades: u32) -> Vec<u32> {
    let mut out = Vec::with_capacity((blades * BLADE_INDICES) as usize);
    for b in 0..blades {
        let v = b * BLADE_VERTICES;
        out.extend_from_slice(&[v, v + 1, v + 2, v + 2, v + 1, v + 3, v + 2, v + 3, v + 4, v + 4, v + 3, v + 5, v + 4, v + 5, v + 6]);
    }
    out
}

/// Whether blades grow on a kind of ground.
fn grows(ground: Ground) -> bool {
    matches!(ground, Ground::Grass | Ground::Field | Ground::Forest)
}

impl GrassField {
    /// `tiles` in the order of their texture layers.
    pub fn new(gpu: &Gpu, tiles: &[&TerrainTile], tuning: GrassTuning) -> Self {
        let mut patches = HashMap::new();
        let per_tile = ((TILE_SAMPLES - 1) as f32 / PATCH).round() as i32;
        for (layer, tile) in tiles.iter().enumerate() {
            if tile.coord.level != 0 {
                continue;
            }
            let (x0, z0) = tile.coord.origin();
            let step = (PATCH / tile.spacing).round() as usize;
            for pj in 0..per_tile {
                for pi in 0..per_tile {
                    let (mut low, mut high, mut any, mut tall) = (f32::MAX, f32::MIN, false, false);
                    for j in pj as usize * step..=(pj as usize + 1) * step {
                        for i in pi as usize * step..=(pi as usize + 1) * step {
                            let k = TerrainTile::index(i, j);
                            low = low.min(tile.heights[k]);
                            high = high.max(tile.heights[k]);
                            any |= grows(tile.ground[k]);
                            tall |= tile.ground[k] == Ground::Field;
                        }
                    }
                    if any {
                        let cell = ((x0 / PATCH).round() as i32 + pi, (z0 / PATCH).round() as i32 + pj);
                        patches.insert(cell, PatchInfo { layer: layer as u32, low, high, tall });
                    }
                }
            }
        }
        let indices = blade_indices(tuning.blades());
        let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("grass blades"),
            size: (indices.len() * 4).max(4) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&buffer, 0, bytemuck::cast_slice(&indices));
        Self { patches, indices: buffer, tuning }
    }

    /// The patches within reach of any eye, each at the level of detail its nearest eye
    /// asks for, at most `limit` of them, nearest first, then ordered by level.
    pub fn choose(&self, eyes: &[Vec3], limit: usize) -> Vec<Chosen> {
        let mut found: HashMap<(i32, i32), f32> = HashMap::new();
        let span = (REACH / PATCH).ceil() as i32 + 1;
        for eye in eyes {
            let centre = ((eye.x / PATCH).floor() as i32, (eye.z / PATCH).floor() as i32);
            for dz in -span..=span {
                for dx in -span..=span {
                    let cell = (centre.0 + dx, centre.1 + dz);
                    let Some(info) = self.patches.get(&cell) else { continue };
                    // The nearest point of the patch's box to the eye.
                    let lo = Vec3::new(cell.0 as f32 * PATCH, info.low, cell.1 as f32 * PATCH);
                    let hi = Vec3::new(lo.x + PATCH, info.high + if info.tall { BARLEY_TALL } else { GRASS_TALL }, lo.z + PATCH);
                    let d = eye.clamp(lo, hi).distance(*eye);
                    if d < REACH {
                        let e = found.entry(cell).or_insert(f32::MAX);
                        *e = e.min(d);
                    }
                }
            }
        }
        let mut chosen: Vec<(f32, (i32, i32))> = found.into_iter().map(|(c, d)| (d, c)).collect();
        chosen.sort_by(|a, b| a.0.total_cmp(&b.0).then(a.1.cmp(&b.1)));
        chosen.truncate(limit);
        let mut out: Vec<Chosen> = chosen.into_iter().map(|(d, cell)| Chosen { cell, level: self.tuning.level_at(d) }).collect();
        out.sort_by_key(|c| (c.level, c.cell));
        out
    }

    /// The instances of the chosen patches, from instance `first` and batch
    /// `first_group` on, and how many fall in each level.
    pub fn instances(&self, chosen: &[Chosen], first: u32, first_group: u32) -> (Vec<InstanceGpu>, [u32; LEVELS]) {
        let mut counts = [0u32; LEVELS];
        for c in chosen {
            counts[c.level] += 1;
        }
        let mut starts = [first; LEVELS];
        for l in 1..LEVELS {
            starts[l] = starts[l - 1] + counts[l - 1];
        }
        let instances = chosen
            .iter()
            .map(|c| {
                let info = self.patches[&c.cell];
                let origin = Vec3::new(c.cell.0 as f32 * PATCH, 0.0, c.cell.1 as f32 * PATCH);
                let top = info.high + if info.tall { BARLEY_TALL } else { GRASS_TALL };
                let centre = Vec3::new(origin.x + PATCH * 0.5, (info.low + top) * 0.5, origin.z + PATCH * 0.5);
                // Half the diagonal across, half the height up, and the reach of a lean.
                let radius = Vec3::new(PATCH * 0.5 + 0.8, (top - info.low) * 0.5, PATCH * 0.5 + 0.8).length();
                InstanceGpu {
                    model: Mat4::from_translation(origin).to_cols_array(),
                    state: [0.0, 0.0, info.layer as f32, 0.0],
                    bounds: [centre.x, centre.y, centre.z, radius],
                    group: [first_group + c.level as u32, starts[c.level], 0, 0],
                }
            })
            .collect();
        (instances, counts)
    }

    /// The draw of each level: its indices, then where they start and the base vertex.
    pub fn draw_headers(&self) -> [[u32; 3]; LEVELS] {
        self.tuning.level_blades().map(|b| [b * BLADE_INDICES, 0, 0])
    }

    /// This frame's wind and pushers.
    pub fn uniform(&self, _seconds: f32, pushers: &[Vec3], weather: &vellmark_sim::Weather) -> GrassUniform {
        let mut out = GrassUniform {
            wind: [weather.wind[0], weather.wind[1], weather.wind_strength, 3.5],
            shape: [self.tuning.near, self.tuning.density, PATCH, REACH],
            pushers: [[0.0; 4]; MAX_PUSHERS],
            counts: [0, self.tuning.blades(), 0, 0],
        };
        for (slot, p) in out.pushers.iter_mut().zip(pushers.iter()) {
            *slot = [p.x, p.y, p.z, 0.55];
        }
        out.counts[0] = pushers.len().min(MAX_PUSHERS) as u32;
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_level_holds_what_the_density_asks_for() {
        let t = GrassTuning::default();
        let blades = t.level_blades();
        for d in [0.0f32, 3.0, 7.9, 8.0, 15.0, 16.0, 31.0, 40.0, 70.0] {
            let level = t.level_at(d);
            // Blades per square metre wanted at this distance, over a whole patch.
            let wanted = t.density * (t.near / d.max(t.near)).powi(2) * PATCH * PATCH;
            assert!(blades[level] as f32 >= wanted - 1.0, "at {d} m level {level} has {} blades, {wanted} wanted", blades[level]);
        }
    }

    #[test]
    fn the_blade_indices_stay_within_their_blade() {
        let indices = blade_indices(3);
        assert_eq!(indices.len(), 3 * BLADE_INDICES as usize);
        for (n, tri) in indices.chunks_exact(BLADE_INDICES as usize).enumerate() {
            assert!(tri.iter().all(|i| i / BLADE_VERTICES == n as u32));
        }
    }
}
