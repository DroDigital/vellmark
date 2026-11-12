//! The terrain on the GPU.
//!
//! Every tile is drawn from one shared grid of indices: the vertex shader reads the
//! tile's heights from a texture, one layer per tile, so a tile costs no vertex buffer
//! of its own. The heights are stored with a border of one sample taken from the
//! neighbouring tiles, so that normals are the same on both sides of a seam. Each tile
//! also has its ground, a class per sample, which the surface shader blends between.
//!
//! The coarser rings are drawn whole and cut where a finer ring draws, by a test in the
//! fragment shader against the finer ring's extent; the rings' borders lie on the coarse
//! grid's lines, so a cell is always wholly in or wholly out.

use bytemuck::{Pod, Zeroable};
use std::collections::HashMap;
use vellmark_content::{AssetId, MaterialSet, TerrainTile, TileCoord, TILE_SAMPLES};
use vellmark_gpu::{wgpu, Gpu};

/// The most tiles drawn: three rings of 8 × 8.
pub const MAX_TILES: usize = 192;
/// Samples along a side of a tile's textures: the tile's own and a border each side.
const BORDERED: u32 = TILE_SAMPLES as u32 + 2;

/// Matches `TerrainTable` in `shaders/opaque.wgsl` and `shaders/shadow.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct TerrainTable {
    /// The material of each ground class, in the order of `Ground`, then rock.
    materials: [[u32; 4]; 3],
    /// For each tile: its least corner x and z, its spacing, and its texture layer;
    /// then the half-width of the finer ring it must not draw over (0 for none).
    tiles: [[f32; 8]; MAX_TILES],
}

/// The material keys of the ground classes, in the order of `Ground`, then rock.
const GROUND_MATERIALS: [&str; 9] = ["grass", "straw", "gravel", "earth", "mud", "gravel", "moss", "erasure", "rock"];
/// The level stored where a sample is dry. Matches `DRY` in `shaders/opaque.wgsl`.
const DRY: f32 = -1.0e6;
/// Where the water's material is in the table.
const WATER_SLOT: usize = 9;

pub struct TerrainGpu {
    pub tile_count: u32,
    pub index_count: u32,
    pub indices: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
    /// The grass that grows on it, and its wind and pushers for the frame.
    pub grass: crate::grass::GrassField,
    pub grass_uniform: wgpu::Buffer,
    /// What the water pass reads: the terrain's own resources, the water's level at each
    /// sample, and which tiles have any water.
    pub table: wgpu::Buffer,
    pub height_view: wgpu::TextureView,
    pub ground_view: wgpu::TextureView,
    pub sky_view: wgpu::TextureView,
    pub water_view: wgpu::TextureView,
    pub water_tiles: wgpu::Buffer,
    pub water_tile_count: u32,
    /// The finest tiles by their place on the grid, and the layer each is in.
    finest: HashMap<(i32, i32), u32>,
    finest_side: f32,
}

/// The most tiles that hold water, four to an entry of the list.
pub const MAX_WATER_TILES: usize = MAX_TILES;

fn texture_entry(binding: u32, sample_type: wgpu::TextureSampleType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Texture { sample_type, view_dimension: wgpu::TextureViewDimension::D2Array, multisampled: false },
        count: None,
    }
}

fn uniform_entry(binding: u32) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
        ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
        count: None,
    }
}

fn terrain_entries() -> Vec<wgpu::BindGroupLayoutEntry> {
    vec![
        uniform_entry(0),
        texture_entry(1, wgpu::TextureSampleType::Float { filterable: false }),
        texture_entry(2, wgpu::TextureSampleType::Uint),
        texture_entry(3, wgpu::TextureSampleType::Float { filterable: false }),
        uniform_entry(4),
    ]
}

pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("terrain"), entries: &terrain_entries() })
}

/// The water pass's: the terrain's, then the water's level at each sample, the list of
/// tiles with water, the lit scene before the water, its depth, and a sampler.
pub fn water_layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let mut entries = terrain_entries();
    entries.push(texture_entry(5, wgpu::TextureSampleType::Float { filterable: false }));
    entries.push(uniform_entry(6));
    entries.push(wgpu::BindGroupLayoutEntry {
        binding: 7,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Float { filterable: true }, view_dimension: wgpu::TextureViewDimension::D2, multisampled: false },
        count: None,
    });
    entries.push(wgpu::BindGroupLayoutEntry {
        binding: 8,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth, view_dimension: wgpu::TextureViewDimension::D2, multisampled: true },
        count: None,
    });
    entries.push(wgpu::BindGroupLayoutEntry {
        binding: 9,
        visibility: wgpu::ShaderStages::FRAGMENT,
        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
        count: None,
    });
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("water"), entries: &entries })
}

impl TerrainGpu {
    /// The four finest tiles nearest a place, as the rain reads them: each tile's least
    /// corner, then its layer, then 1, or all 0 where there is none.
    pub fn tiles_around(&self, x: f32, z: f32) -> [[f32; 4]; 4] {
        let side = self.finest_side;
        let (fx, fz) = (x / side - 0.5, z / side - 0.5);
        let (x0, z0) = (fx.floor() as i32, fz.floor() as i32);
        let mut out = [[0.0; 4]; 4];
        for (k, (dx, dz)) in [(0, 0), (1, 0), (0, 1), (1, 1)].into_iter().enumerate() {
            let c = (x0 + dx, z0 + dz);
            if let Some(layer) = self.finest.get(&c) {
                out[k] = [c.0 as f32 * side, c.1 as f32 * side, *layer as f32, 1.0];
            }
        }
        out
    }

    /// The water pass's bind group, for the scene `behind` the water and its `depth`.
    pub fn water_group(&self, device: &wgpu::Device, layout: &wgpu::BindGroupLayout, behind: &wgpu::TextureView, depth: &wgpu::TextureView, sampler: &wgpu::Sampler) -> wgpu::BindGroup {
        device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("water"),
            layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: self.table.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&self.height_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&self.ground_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&self.sky_view) },
                wgpu::BindGroupEntry { binding: 4, resource: self.grass_uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&self.water_view) },
                wgpu::BindGroupEntry { binding: 6, resource: self.water_tiles.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::TextureView(behind) },
                wgpu::BindGroupEntry { binding: 8, resource: wgpu::BindingResource::TextureView(depth) },
                wgpu::BindGroupEntry { binding: 9, resource: wgpu::BindingResource::Sampler(sampler) },
            ],
        })
    }
}

/// The shared grid: two triangles a cell, split from the least corner to the greatest
/// as `TerrainTile::height_at` assumes, wound to face up.
fn grid_indices() -> Vec<u32> {
    let n = TILE_SAMPLES as u32;
    let mut out = Vec::with_capacity(((n - 1) * (n - 1) * 6) as usize);
    for j in 0..n - 1 {
        for i in 0..n - 1 {
            let (a, b, c, d) = (j * n + i, j * n + i + 1, (j + 1) * n + i, (j + 1) * n + i + 1);
            out.extend_from_slice(&[a, d, b, a, c, d]);
        }
    }
    out
}

impl TerrainGpu {
    pub fn new(gpu: &Gpu, layout: &wgpu::BindGroupLayout, tiles: &[&TerrainTile], materials: &MaterialSet) -> Result<Self, String> {
        if tiles.len() > MAX_TILES {
            return Err(format!("the terrain has {} tiles; the limit is {MAX_TILES}", tiles.len()));
        }
        let by_coord: HashMap<TileCoord, &TerrainTile> = tiles.iter().map(|t| (t.coord, *t)).collect();
        let last = TILE_SAMPLES as i32 - 1;
        let mut heights = Vec::with_capacity((BORDERED * BORDERED) as usize * tiles.len());
        let mut ground = Vec::with_capacity(heights.capacity());
        let mut sky = Vec::with_capacity(heights.capacity());
        let mut water = Vec::with_capacity(heights.capacity());
        for tile in tiles {
            for j in -1..=last + 1 {
                for i in -1..=last + 1 {
                    // A border sample comes from the neighbour that holds it, or, at the
                    // outer edge of everything, from this tile's own edge.
                    let (dx, ii) = if i < 0 { (-1, last - 1) } else if i > last { (1, 1) } else { (0, i) };
                    let (dz, jj) = if j < 0 { (-1, last - 1) } else if j > last { (1, 1) } else { (0, j) };
                    let neighbour = by_coord.get(&TileCoord { level: tile.coord.level, x: tile.coord.x + dx, z: tile.coord.z + dz });
                    let (source, si, sj) = match neighbour {
                        Some(n) if dx != 0 || dz != 0 => (*n, ii, jj),
                        _ => (*tile, i.clamp(0, last), j.clamp(0, last)),
                    };
                    let k = TerrainTile::index(si as usize, sj as usize);
                    heights.push(source.heights[k]);
                    ground.push(source.ground[k] as u8);
                    sky.push(source.sky[k]);
                    // Dry is a level far below anything, which the shader can test without NaN.
                    water.push(if source.water[k].is_nan() { DRY } else { source.water[k] });
                }
            }
        }

        // Each ring is cut where the next finer ring draws.
        let mut extent: HashMap<u8, f32> = HashMap::new();
        for t in tiles {
            let (x0, z0) = t.coord.origin();
            let far = [x0, z0, x0 + t.coord.side(), z0 + t.coord.side()].iter().fold(0.0f32, |m, v| m.max(v.abs()));
            let e = extent.entry(t.coord.level).or_insert(0.0);
            *e = e.max(far);
        }
        let mut table = TerrainTable::zeroed();
        for (slot, key) in GROUND_MATERIALS.iter().enumerate() {
            let index = materials
                .index_of(AssetId::from_key(&format!("material.{key}")))
                .ok_or_else(|| format!("the terrain needs the material `{key}`"))?;
            table.materials[slot / 4][slot % 4] = index;
        }
        // Then the water's own material, for its letters and its tint.
        table.materials[WATER_SLOT / 4][WATER_SLOT % 4] =
            materials.index_of(AssetId::from_key("material.water")).ok_or("the terrain needs the material `water`")?;
        for (layer, t) in tiles.iter().enumerate() {
            let (x0, z0) = t.coord.origin();
            let hole = extent.iter().filter(|(l, _)| **l < t.coord.level).max_by_key(|(l, _)| **l).map_or(0.0, |(_, e)| *e);
            table.tiles[layer] = [x0, z0, t.spacing, layer as f32, hole, 0.0, 0.0, 0.0];
        }

        let device = &gpu.device;
        let make = |label, format, bytes: &[u8], texel: u32| {
            let texture = device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d { width: BORDERED, height: BORDERED, depth_or_array_layers: tiles.len().max(1) as u32 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            });
            if !bytes.is_empty() {
                gpu.queue.write_texture(
                    wgpu::TexelCopyTextureInfo { texture: &texture, mip_level: 0, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
                    bytes,
                    wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(BORDERED * texel), rows_per_image: Some(BORDERED) },
                    wgpu::Extent3d { width: BORDERED, height: BORDERED, depth_or_array_layers: tiles.len() as u32 },
                );
            }
            texture.create_view(&wgpu::TextureViewDescriptor { dimension: Some(wgpu::TextureViewDimension::D2Array), ..Default::default() })
        };
        let height_view = make("terrain heights", wgpu::TextureFormat::R32Float, bytemuck::cast_slice(&heights), 4);
        let ground_view = make("terrain ground", wgpu::TextureFormat::R8Uint, &ground, 1);
        let sky_view = make("terrain sky", wgpu::TextureFormat::R8Unorm, &sky, 1);
        let water_view = make("terrain water", wgpu::TextureFormat::R32Float, bytemuck::cast_slice(&water), 4);
        // The tiles with any water, by their place in the table.
        let wet: Vec<u32> = tiles.iter().enumerate().filter(|(_, t)| t.water.iter().any(|w| !w.is_nan())).map(|(i, _)| i as u32).collect();
        let mut list = [[0u32; 4]; MAX_WATER_TILES / 4];
        for (i, t) in wet.iter().enumerate() {
            list[i / 4][i % 4] = *t;
        }
        let water_tiles = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("water tiles"),
            size: std::mem::size_of_val(&list) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&water_tiles, 0, bytemuck::cast_slice(&list));
        let table_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("terrain table"),
            size: std::mem::size_of::<TerrainTable>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&table_buffer, 0, bytemuck::bytes_of(&table));
        let indices = grid_indices();
        let index_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("terrain grid"),
            size: (indices.len() * 4) as u64,
            usage: wgpu::BufferUsages::INDEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&index_buffer, 0, bytemuck::cast_slice(&indices));
        let grass = crate::grass::GrassField::new(gpu, tiles, crate::grass::GrassTuning::default());
        let grass_uniform = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("grass"),
            size: std::mem::size_of::<crate::grass::GrassUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&grass_uniform, 0, bytemuck::bytes_of(&grass.uniform(0.0, &[], &Default::default())));
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("terrain"),
            layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: table_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&height_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&ground_view) },
                wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&sky_view) },
                wgpu::BindGroupEntry { binding: 4, resource: grass_uniform.as_entire_binding() },
            ],
        });
        Ok(Self {
            tile_count: tiles.len() as u32,
            index_count: indices.len() as u32,
            indices: index_buffer,
            bind_group,
            grass,
            grass_uniform,
            table: table_buffer,
            height_view,
            ground_view,
            sky_view,
            water_view,
            water_tiles,
            water_tile_count: wet.len() as u32,
            finest: tiles
                .iter()
                .enumerate()
                .filter(|(_, t)| t.coord.level == 0)
                .map(|(i, t)| {
                    let (x0, z0) = t.coord.origin();
                    (((x0 / t.coord.side()).round() as i32, (z0 / t.coord.side()).round() as i32), i as u32)
                })
                .collect(),
            finest_side: tiles.iter().find(|t| t.coord.level == 0).map_or(64.0, |t| t.coord.side()),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grid_faces_up() {
        let indices = grid_indices();
        let n = TILE_SAMPLES as u32;
        let at = |k: u32| glam::Vec3::new((k % n) as f32, 0.0, (k / n) as f32);
        for tri in indices.chunks_exact(3) {
            let (a, b, c) = (at(tri[0]), at(tri[1]), at(tri[2]));
            assert!((b - a).cross(c - a).y > 0.0);
        }
    }
}
