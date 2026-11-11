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

pub struct TerrainGpu {
    pub tile_count: u32,
    pub index_count: u32,
    pub indices: wgpu::Buffer,
    pub bind_group: wgpu::BindGroup,
}

pub fn layout(device: &wgpu::Device) -> wgpu::BindGroupLayout {
    let texture = |binding, sample_type, visibility| wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Texture { sample_type, view_dimension: wgpu::TextureViewDimension::D2Array, multisampled: false },
        count: None,
    };
    device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
        label: Some("terrain"),
        entries: &[
            wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                count: None,
            },
            texture(1, wgpu::TextureSampleType::Float { filterable: false }, wgpu::ShaderStages::VERTEX_FRAGMENT),
            texture(2, wgpu::TextureSampleType::Uint, wgpu::ShaderStages::FRAGMENT),
        ],
    })
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
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("terrain"),
            layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: table_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&height_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::TextureView(&ground_view) },
            ],
        });
        Ok(Self { tile_count: tiles.len() as u32, index_count: indices.len() as u32, indices: index_buffer, bind_group })
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
