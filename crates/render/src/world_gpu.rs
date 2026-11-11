//! The scene and its materials on the GPU.

use bytemuck::{Pod, Zeroable};
use glam::Mat4;
use vellmark_content::{AssetId, GlyphAtlas, MaterialSet, Mesh, Scene, TextPages, Vertex, GLYPHS_PER_FACE, TONE_STEPS};
use vellmark_gpu::{wgpu, Gpu};

pub const MAX_OBJECTS: usize = 256;
pub const MAX_MATERIALS: usize = 32;

pub const VERTEX_ATTRIBUTES: [wgpu::VertexAttribute; 7] =
    wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x3, 2 => Float32x4, 3 => Float32x2, 4 => Uint32, 5 => Uint32, 6 => Float32];

pub fn vertex_layout() -> wgpu::VertexBufferLayout<'static> {
    wgpu::VertexBufferLayout {
        array_stride: std::mem::size_of::<Vertex>() as u64,
        step_mode: wgpu::VertexStepMode::Vertex,
        attributes: &VERTEX_ATTRIBUTES,
    }
}

pub struct MeshGpu {
    pub vertices: wgpu::Buffer,
    pub indices: wgpu::Buffer,
    pub index_count: u32,
}

impl MeshGpu {
    pub fn new(gpu: &Gpu, mesh: &Mesh) -> Self {
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
        Self {
            vertices: make("mesh vertices", bytemuck::cast_slice(&mesh.vertices), wgpu::BufferUsages::VERTEX),
            indices: make("mesh indices", bytemuck::cast_slice(&mesh.indices), wgpu::BufferUsages::INDEX),
            index_count: mesh.indices.len() as u32,
        }
    }
}

/// One thing to draw: a mesh and where it stands. Slot 0 is everything that never moves.
pub struct Drawable {
    pub key: AssetId,
    pub mesh: MeshGpu,
    pub transform: Mat4,
}

#[derive(Default)]
pub struct SceneGpu {
    pub drawables: Vec<Drawable>,
    pub lights: Vec<vellmark_content::Light>,
    /// The scene's authored text. The renderer uploads it when the scene is set.
    pub pages: TextPages,
}

impl SceneGpu {
    /// `mesh_of` returns a mesh by its ID, from the pack.
    pub fn new(gpu: &Gpu, scene: &Scene, pages: TextPages, mesh_of: &dyn Fn(AssetId) -> Option<Mesh>) -> Result<Self, String> {
        let mut drawables = Vec::new();
        let static_mesh = mesh_of(scene.static_mesh).ok_or("the scene's static mesh is missing")?;
        drawables.push(Drawable { key: scene.static_mesh, mesh: MeshGpu::new(gpu, &static_mesh), transform: Mat4::IDENTITY });
        for object in &scene.objects {
            let mesh = mesh_of(object.mesh).ok_or("an object's mesh is missing")?;
            let t = object.transform;
            let transform = Mat4::from_cols_array(&[
                t[0], t[4], t[8], 0.0, t[1], t[5], t[9], 0.0, t[2], t[6], t[10], 0.0, t[3], t[7], t[11], 1.0,
            ]);
            drawables.push(Drawable { key: object.key, mesh: MeshGpu::new(gpu, &mesh), transform });
        }
        if drawables.len() > MAX_OBJECTS {
            return Err(format!("the scene has {} objects; the limit is {MAX_OBJECTS}", drawables.len()));
        }
        Ok(Self { drawables, lights: scene.lights.clone(), pages })
    }

    /// Moves an object. The simulation's snapshot will drive this once doors open.
    pub fn set_transform(&mut self, key: AssetId, transform: Mat4) {
        if let Some(d) = self.drawables.iter_mut().find(|d| d.key == key) {
            d.transform = transform;
        }
    }
}

/// Matches `MaterialParams` in `shaders/glyph.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct MaterialParams {
    surface: [f32; 4],
    info: [u32; 4],
    family: [u32; 8],
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
            slot.info = [m.family.len() as u32, 0, 0, 0];
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

        let texture_array = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
            ty: wgpu::BindingType::Texture {
                sample_type: wgpu::TextureSampleType::Float { filterable: true },
                view_dimension: wgpu::TextureViewDimension::D2Array,
                multisampled: false,
            },
            count: None,
        };
        let sampler_entry = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::FRAGMENT,
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
        let resources = Self { layout, bind_group, page_table, page_sheet, glowing };
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
