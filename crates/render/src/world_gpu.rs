//! The scene and its materials on the GPU.

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use vellmark_content::{AssetId, GlyphAtlas, MaterialSet, Mesh, Scene, TextPages, Vertex, GLYPHS_PER_FACE, TONE_STEPS};
use vellmark_gpu::{wgpu, Gpu};

/// The most instances in a frame, placed things and grass patches together, and the most
/// batches they are drawn in.
pub const MAX_INSTANCES: usize = 8192;
pub const MAX_GROUPS: usize = 256;
pub const MAX_MATERIALS: usize = 32;

pub const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 8] =
    wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x4, 3 => Float32x2, 4 => Uint32, 5 => Uint32, 6 => Float32, 7 => Float32];

pub fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &VERTEX_ATTRIBUTES,
    }
}

/// One instance as the GPU reads it. Matches `Instance` in `opaque.wgsl`, `shadow.wgsl`,
/// and `cull.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct InstanceGpu {
    pub model: [f32; 16],
    /// x: heat, -1 to 1. y: 1 if the thing is not drawn (broken).
    pub state: [f32; 4],
    /// The sphere that holds it, in the world: centre and radius.
    pub bounds: [f32; 4],
    /// x: its group, the mesh it is drawn from. y: where the group's batch starts in a
    /// list. z: how many levels of detail it has (1 for none) in the low byte, and above
    /// it how many instances the group has, which is how far apart its coarser levels'
    /// batches are. w: where its first coarser level's batch starts in a list.
    pub group: [u32; 4],
}

/// The part of the pool one mesh takes.
#[derive(Clone, Copy, Debug)]
pub struct MeshRange {
    pub first_index: u32,
    pub index_count: u32,
    pub base_vertex: i32,
    /// The sphere that holds the mesh in its own space.
    pub centre: glam::Vec3,
    pub radius: f32,
}

/// Every instance of one mesh, which are kept together in the instance buffer.
///
/// A mesh with coarser levels of detail has a group for each level, one after another.
/// Only the finest holds instances; the culling pass moves each to the level it is seen
/// at, writing it into that level's own batch in the view's list.
#[derive(Clone, Copy, Debug)]
pub struct Group {
    pub mesh: usize,
    pub first_instance: u32,
    pub count: u32,
    /// Where the group's batch starts in a list.
    pub list_base: u32,
    /// For the finest of a chain, how many levels it has; 1 for a mesh with none; 0 for
    /// a coarser level, which holds no instances of its own.
    pub levels: u32,
}

impl Group {
    pub fn is_coarser_level(&self) -> bool {
        self.levels == 0
    }
}

/// One placed thing: what it is and where.
#[derive(Clone, Copy, Debug)]
pub struct Placed {
    pub key: AssetId,
    pub group: usize,
    pub transform: Mat4,
}

/// The scene on the GPU: every mesh in one pool of vertices and indices, and every placed
/// thing as an instance, kept together by mesh so that each mesh is one batch.
#[derive(Default)]
pub struct SceneGpu {
    pub vertices: Option<wgpu::Buffer>,
    pub indices: Option<wgpu::Buffer>,
    pub meshes: Vec<MeshRange>,
    pub groups: Vec<Group>,
    /// In the order of the instance buffer.
    pub placed: Vec<Placed>,
    pub lights: Vec<vellmark_content::Light>,
    /// The scene's authored text. The renderer uploads it when the scene is set.
    pub pages: TextPages,
    /// Where the scene's batches end in a list; what follows (grass) starts here.
    pub list_end: u32,
}

fn bounds_of(mesh: &Mesh) -> (glam::Vec3, f32) {
    if mesh.vertices.is_empty() {
        return (glam::Vec3::ZERO, 0.0);
    }
    let (mut lo, mut hi) = (glam::Vec3::splat(f32::MAX), glam::Vec3::splat(f32::MIN));
    for v in &mesh.vertices {
        let p = glam::Vec3::from(v.position);
        lo = lo.min(p);
        hi = hi.max(p);
    }
    let centre = (lo + hi) * 0.5;
    let radius = mesh.vertices.iter().map(|v| glam::Vec3::from(v.position).distance(centre)).fold(0.0, f32::max);
    (centre, radius)
}

impl SceneGpu {
    /// `mesh_of` returns a mesh by its ID, from the pack.
    pub fn new(gpu: &Gpu, scene: &Scene, pages: TextPages, levels: &vellmark_content::MeshLevels, mesh_of: &dyn Fn(AssetId) -> Option<Mesh>) -> Result<Self, String> {
        // Every distinct mesh once, in the pool; a mesh with levels of detail is followed
        // by its coarser levels.
        let mut ids: Vec<AssetId> = vec![scene.static_mesh];
        let mut chain_len: Vec<u32> = vec![1];
        for object in &scene.objects {
            if !ids.contains(&object.mesh) {
                ids.push(object.mesh);
                match levels.chain(object.mesh) {
                    Some(chain) if chain.len() > 1 => {
                        chain_len.push(chain.len() as u32);
                        for coarser in &chain[1..] {
                            ids.push(*coarser);
                            chain_len.push(0);
                        }
                    }
                    _ => chain_len.push(1),
                }
            }
        }
        let mut vertices: Vec<Vertex> = Vec::new();
        let mut indices: Vec<u32> = Vec::new();
        let mut meshes = Vec::new();
        for id in &ids {
            let mesh = mesh_of(*id).ok_or("a mesh of the scene is missing from the pack")?;
            let (centre, radius) = bounds_of(&mesh);
            meshes.push(MeshRange {
                first_index: indices.len() as u32,
                index_count: mesh.indices.len() as u32,
                base_vertex: vertices.len() as i32,
                centre,
                radius,
            });
            vertices.extend_from_slice(&mesh.vertices);
            indices.extend_from_slice(&mesh.indices);
        }
        if meshes.len() > MAX_GROUPS {
            return Err(format!("the scene has {} meshes; the limit is {MAX_GROUPS}", meshes.len()));
        }
        // Every placed thing, kept together by mesh.
        let mut by_mesh: Vec<Vec<(AssetId, Mat4)>> = vec![Vec::new(); meshes.len()];
        by_mesh[0].push((scene.static_mesh, Mat4::IDENTITY));
        for object in &scene.objects {
            let t = object.transform;
            let transform = Mat4::from_cols_array(&[
                t[0], t[4], t[8], 0.0, t[1], t[5], t[9], 0.0, t[2], t[6], t[10], 0.0, t[3], t[7], t[11], 1.0,
            ]);
            let m = ids.iter().position(|id| *id == object.mesh).unwrap();
            by_mesh[m].push((object.key, transform));
        }
        let mut groups: Vec<Group> = Vec::new();
        let mut placed = Vec::new();
        for (m, list) in by_mesh.into_iter().enumerate() {
            let first = placed.len() as u32;
            groups.push(Group { mesh: m, first_instance: first, count: list.len() as u32, list_base: first, levels: chain_len[m] });
            for (key, transform) in list {
                placed.push(Placed { key, group: m, transform });
            }
        }
        // A coarser level holds as many as its finest might send it, in its own part of
        // each list, after every placed thing's.
        let mut list_end = placed.len() as u32;
        let mut finest = 0;
        for g in 0..groups.len() {
            if groups[g].levels >= 1 {
                finest = g;
            } else {
                groups[g].count = groups[finest].count;
                groups[g].first_instance = groups[finest].first_instance;
                groups[g].list_base = list_end;
                list_end += groups[finest].count;
            }
        }
        if list_end as usize > MAX_INSTANCES {
            return Err(format!("the scene needs {list_end} places in a list; the limit is {MAX_INSTANCES}"));
        }
        let make = |label, bytes: &[u8], usage| {
            let buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size: (bytes.len().max(4)) as u64,
                usage: usage | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            });
            gpu.queue.write_buffer(&buffer, 0, bytes);
            buffer
        };
        Ok(Self {
            vertices: Some(make("mesh pool vertices", bytemuck::cast_slice(&vertices), wgpu::BufferUsages::VERTEX)),
            indices: Some(make("mesh pool indices", bytemuck::cast_slice(&indices), wgpu::BufferUsages::INDEX)),
            meshes,
            groups,
            placed,
            lights: scene.lights.clone(),
            pages,
            list_end,
        })
    }

    /// Moves a placed thing. The simulation's snapshot will drive this once doors open.
    pub fn set_transform(&mut self, key: AssetId, transform: Mat4) {
        if let Some(p) = self.placed.iter_mut().find(|p| p.key == key) {
            p.transform = transform;
        }
    }

    /// Every instance as the GPU reads it, with this frame's heat and breakage.
    pub fn instances(&self, heat: &[(u64, f32)], broken: &[u64]) -> Vec<InstanceGpu> {
        self.placed
            .iter()
            .map(|p| {
                let mesh = &self.meshes[p.group];
                let centre = p.transform.transform_point3(mesh.centre);
                let scale = p.transform.x_axis.truncate().length().max(p.transform.y_axis.truncate().length()).max(p.transform.z_axis.truncate().length());
                let heat = heat.iter().find(|(id, _)| *id == p.key.0).map_or(0.0, |(_, h)| *h);
                let hidden = if broken.contains(&p.key.0) { 1.0 } else { 0.0 };
                InstanceGpu {
                    model: p.transform.to_cols_array(),
                    state: [heat, hidden, 0.0, 0.0],
                    bounds: [centre.x, centre.y, centre.z, mesh.radius * scale],
                    group: {
                        let g = &self.groups[p.group];
                        let coarser = if g.levels > 1 { self.groups[p.group + 1].list_base } else { 0 };
                        [p.group as u32, g.list_base, g.levels.max(1) | (g.count << 8), coarser]
                    },
                }
            })
            .collect()
    }
}

/// Matches `MaterialParams` in `shaders/glyph.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MaterialParams {
    surface: [f32; 4],
    info: [u32; 4],
    family: [u32; 8],
    /// The lexicon's slots, four characters to a word of the vector.
    words: [[u32; 4]; vellmark_content::Lexicon::SLOTS],
}

/// Material maps, the glyph atlas, and the per-glyph tone table: everything a surface
/// needs beyond its own geometry, in one bind group.
/// Side of the sheet that holds a scene's authored text, in characters.
pub const PAGE_SHEET: u32 = 1024;

/// Matches `PageEntry` in `shaders/glyph.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PageParams {
    /// Left column, top row, width, height, in cells of the page's level.
    place: [i32; 4],
    /// Where the page starts on the sheet, then its level.
    sheet_level: [u32; 4],
}

pub struct SurfaceResources {
    pub layout: wgpu::BindGroupLayout,
    pub bind_group: wgpu::BindGroup,
    page_table: wgpu::Buffer,
    page_sheet: wgpu::Texture,
    /// Bit i is set if material i gives off light.
    pub glowing: u32,
    /// The trees' leaf cards' material, counted from 1; 0 if the set has none.
    pub leaves: u32,
}

fn array_texture(gpu: &Gpu, label: &str, size: u32, mips: u32, layers: u32, format: wgpu::TextureFormat) -> wgpu::Texture {
    gpu.device.create_texture(&wgpu::TextureDescriptor {
        label: Some(label),
        size: wgpu::Extent3d { width: size, height: size, depth_or_array_layers: layers },
        mip_level_count: mips,
        sample_count: 1,
        dimension: wgpu::TextureDimension::D2,
        format,
        usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
        view_formats: &[],
    })
}

fn write_level(gpu: &Gpu, texture: &wgpu::Texture, level: u32, side: u32, layers: u32, bytes: &[u8]) {
    gpu.queue.write_texture(
        wgpu::TexelCopyTextureInfo { texture, mip_level: level, origin: wgpu::Origin3d::ZERO, aspect: wgpu::TextureAspect::All },
        bytes,
        wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(side * 4), rows_per_image: Some(side) },
        wgpu::Extent3d { width: side, height: side, depth_or_array_layers: layers },
    );
}

impl SurfaceResources {
    pub fn new(gpu: &Gpu, set: &MaterialSet, glyphs: &GlyphAtlas, atlas: &crate::atlas::AtlasGpu) -> Result<Self, String> {
        let count = set.materials.len() as u32;
        if count == 0 || count as usize > MAX_MATERIALS {
            return Err(format!("{count} materials; there must be between 1 and {MAX_MATERIALS}"));
        }
        let colour = array_texture(gpu, "material colour", set.size, set.mips, count, wgpu::TextureFormat::Rgba8UnormSrgb);
        let normal = array_texture(gpu, "material normal", set.size, set.mips, count, wgpu::TextureFormat::Rgba8Unorm);
        for level in 0..set.mips {
            let side = set.size >> level;
            write_level(gpu, &colour, level, side, count, &set.texels[set.level_range(level, 0)]);
            write_level(gpu, &normal, level, side, count, &set.texels[set.level_range(level, 1)]);
        }
        let array_view = |t: &wgpu::Texture| {
            t.create_view(&wgpu::TextureViewDescriptor { dimension: Some(wgpu::TextureViewDimension::D2Array), ..Default::default() })
        };
        let (colour_view, normal_view) = (array_view(&colour), array_view(&normal));
        let sampler = gpu.device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("materials"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::MipmapFilterMode::Linear,
            anisotropy_clamp: 8,
            ..Default::default()
        });

        let mut table = [MaterialParams::zeroed(); MAX_MATERIALS];
        for (slot, m) in table.iter_mut().zip(&set.materials) {
            slot.surface = [m.tile_metres, m.contrast, m.relief, m.emissive];
            slot.info = [m.family.len() as u32, m.lexicon.elements.len() as u32, m.lexicon.hot.len() as u32, m.lexicon.cold.len() as u32];
            for (word, letters) in slot.words.iter_mut().zip(m.lexicon.slots()) {
                for (i, chunk) in letters.chunks_exact(4).enumerate() {
                    word[i] = u32::from_le_bytes(chunk.try_into().unwrap());
                }
            }
            for (i, c) in m.family.iter().enumerate() {
                slot.family[i] = *c as u32;
            }
        }
        let table_buffer = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("material table"),
            size: std::mem::size_of_val(&table) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        gpu.queue.write_buffer(&table_buffer, 0, bytemuck::cast_slice(&table));

        // One row for each glyph: sixteen thresholds, then the least and most coverage
        // the glyph can reach, then its mean distance.
        let width = TONE_STEPS as u32 + 3;
        let rows = (glyphs.faces.len() * GLYPHS_PER_FACE) as u32;
        let mut tone = Vec::with_capacity((width * rows) as usize);
        for face in &glyphs.faces {
            for g in &face.glyphs {
                tone.extend_from_slice(&g.tone);
                tone.extend_from_slice(&[g.min_cover, g.max_cover, g.mean_distance]);
            }
        }
        let tone_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("glyph tone"),
            size: wgpu::Extent3d { width, height: rows, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        gpu.queue.write_texture(
            tone_texture.as_image_copy(),
            bytemuck::cast_slice(&tone),
            wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(width * 4), rows_per_image: Some(rows) },
            wgpu::Extent3d { width, height: rows, depth_or_array_layers: 1 },
        );
        let tone_view = tone_texture.create_view(&Default::default());

        let page_table = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("text page table"),
            size: (std::mem::size_of::<PageParams>() * TextPages::MAX_PAGES) as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let page_sheet = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("text pages"),
            size: wgpu::Extent3d { width: PAGE_SHEET, height: PAGE_SHEET, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let page_view = page_sheet.create_view(&Default::default());

        // Read in the vertex stage too: the wind writes its sentences into the grass there.
        let texture_array = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2Array,
                multisampled: false,
            },
            count: None,
        };
        let sampler_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
            ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
            count: None,
        };
        let layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("surface"),
            entries: &[
                texture_array(0),
                texture_array(1),
                sampler_entry(2),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                texture_array(4),
                sampler_entry(5),
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: false, min_binding_size: None },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("surface"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&colour_view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&normal_view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&sampler) },
                wgpu::BindGroupEntry { binding: 3, resource: table_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&atlas.view) },
                wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::Sampler(&atlas.sampler) },
                wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(&tone_view) },
                wgpu::BindGroupEntry { binding: 7, resource: page_table.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 8, resource: wgpu::BindingResource::TextureView(&page_view) },
            ],
        });
        let glowing = set.materials.iter().enumerate().filter(|(_, m)| m.emissive > 0.0).fold(0u32, |bits, (i, _)| bits | 1 << i);
        let leaves = set.index_of(AssetId::from_key("material.leaves")).map_or(0, |i| i + 1);
        let resources = Self { layout, bind_group, page_table, page_sheet, glowing, leaves };
        resources.set_pages(gpu, &TextPages::default());
        Ok(resources)
    }

    /// Replaces the authored text with a scene's. Text travels with its scene, so this
    /// is called whenever the scene changes.
    pub fn set_pages(&self, gpu: &Gpu, pages: &TextPages) {
        let mut table = [PageParams::zeroed(); TextPages::MAX_PAGES];
        for (slot, e) in table.iter_mut().zip(&pages.entries) {
            slot.place = [e.left, e.top, e.width as i32, e.height as i32];
            slot.sheet_level = [e.sheet_x, e.sheet_y, e.level, 0];
        }
        gpu.queue.write_buffer(&self.page_table, 0, bytemuck::cast_slice(&table));
        if pages.sheet_size == PAGE_SHEET && !pages.sheet.is_empty() {
            gpu.queue.write_texture(
                self.page_sheet.as_image_copy(),
                &pages.sheet,
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(PAGE_SHEET), rows_per_image: Some(PAGE_SHEET) },
                wgpu::Extent3d { width: PAGE_SHEET, height: PAGE_SHEET, depth_or_array_layers: 1 },
            );
        }
    }
}
