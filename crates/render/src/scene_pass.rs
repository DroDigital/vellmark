//! The passes that draw the world: sun shadows, the depth pre-pass, lit surfaces with
//! the sky behind them, and the step from the lit scene to the screen.

use crate::frame::{
    cascade_matrix, view_uniform, Atmosphere, ViewUniform, CASCADES, CASCADE_FAR, MAX_VIEWS, SHADOW_SIZE,
};
use crate::lights::{cube_face_matrix, Lights};
use crate::world_gpu::{vertex_layout, InstanceGpu, SceneGpu, SurfaceResources, MAX_GROUPS, MAX_INSTANCES};
use crate::{FrameInput, ShaderLibrary};
use glam::{Mat4, Vec3};
use vellmark_gpu::{wgpu, Gpu, PassTimers, PipelineRegistry};

pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const SAMPLES: u32 = 4;

/// Views whose shadows are drawn. Further views borrow the first view's.
const SHADOWED_VIEWS: usize = 2;
/// Rain columns around each view. Matches `RAIN_SIDE` squared in `shaders/opaque.wgsl`.
const RAIN_DROPS: u32 = 160 * 160;

const BLOOM_LEVELS: usize = 5;
const CUBE_FACES: usize = 6;
/// The level of detail a tree casts its shadows from.
const SHADOW_LEVEL: usize = 2;
/// The size of one cascade or cube-face uniform: a matrix and a row of flags.
const CASCADE_UNIFORM: u64 = 80;

/// One loose letter as the GPU is given it. Matches `LetterInput` in `opaque.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct LetterInstance {
    position: [f32; 3],
    rotation: [f32; 4],
    material: u32,
    seed: u32,
}

const LETTER_ATTRIBUTES: [wgpu::VertexAttribute; 4] =
    wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x4, 2 => Uint32, 3 => Uint32];

struct Targets {
    size: (u32, u32),
    colour_msaa: wgpu::TextureView,
    depth_msaa: wgpu::TextureView,
    resolved: wgpu::TextureView,
    resolved_texture: wgpu::Texture,
    /// The lit scene as it was before the water, which the water pass reads.
    behind_texture: wgpu::Texture,
    behind: wgpu::TextureView,
    post_bind_group: wgpu::BindGroup,
    /// Each level of the bloom chain, largest first, as a target and as a source.
    bloom_levels: Vec<wgpu::TextureView>,
    bloom_sources: Vec<wgpu::BindGroup>,
    scene_source: wgpu::BindGroup,
    contact_group: wgpu::BindGroup,
    pyramid: crate::cull::PyramidTargets,
}

struct Pipelines {
    shadow: wgpu::RenderPipeline,
    depth: wgpu::RenderPipeline,
    opaque: wgpu::RenderPipeline,
    sky: wgpu::RenderPipeline,
    letters: wgpu::RenderPipeline,
    letter_shadow: wgpu::RenderPipeline,
    terrain_shadow: wgpu::RenderPipeline,
    terrain_depth: wgpu::RenderPipeline,
    terrain: wgpu::RenderPipeline,
    grass_depth: wgpu::RenderPipeline,
    grass: wgpu::RenderPipeline,
    tree_shadow: wgpu::RenderPipeline,
    tree_depth: wgpu::RenderPipeline,
    tree: wgpu::RenderPipeline,
    water: wgpu::RenderPipeline,
    rain: wgpu::RenderPipeline,
    post: wgpu::RenderPipeline,
    bloom_first: wgpu::RenderPipeline,
    bloom_down: wgpu::RenderPipeline,
    bloom_up: wgpu::RenderPipeline,
}

pub struct ScenePass {
    pipelines: Pipelines,
    frame_layout: wgpu::BindGroupLayout,
    object_layout: wgpu::BindGroupLayout,
    cascade_layout: wgpu::BindGroupLayout,
    post_layout: wgpu::BindGroupLayout,
    bloom_layout: wgpu::BindGroupLayout,
    terrain_layout: wgpu::BindGroupLayout,
    water_layout: wgpu::BindGroupLayout,
    water_sampler: wgpu::Sampler,
    /// Made for the terrain and the targets of the moment; remade when either changes.
    water_group: Option<wgpu::BindGroup>,
    /// The trees' leaf cards' material, counted from 1; 0 if there is none.
    leaves: u32,
    lights: Lights,
    view_buffer: wgpu::Buffer,
    view_stride: u64,
    instance_buffer: wgpu::Buffer,
    visible_buffer: wgpu::Buffer,
    draw_buffer: wgpu::Buffer,
    object_stride: u64,
    object_bind_group: wgpu::BindGroup,
    cascade_buffer: wgpu::Buffer,
    cascade_stride: u64,
    cascade_bind_group: wgpu::BindGroup,
    post_buffer: wgpu::Buffer,
    grade: vellmark_content::Grade,
    /// The sentences the wind writes, and the gust now writing one, if any.
    wind_sentences: Vec<String>,
    gust: Option<Gust>,
    post_sampler: wgpu::Sampler,
    shadow_sampler: wgpu::Sampler,
    shadow_layers: Vec<wgpu::TextureView>,
    frame_bind_groups: Vec<wgpu::BindGroup>,
    fog: crate::fog::FogVolume,
    contact: crate::contact::ContactOcclusion,
    culling: crate::cull::Culling,
    view_size: u64,
    targets: Option<Targets>,
    target_format: wgpu::TextureFormat,
    /// One instance for each loose letter, and the step they were last sent at.
    letter_buffer: wgpu::Buffer,
    letters_sent: Option<(u64, usize)>,
    /// The grass patches last sent, after how many placed things.
    grass_sent: Option<(usize, Vec<crate::grass::Chosen>)>,
    /// Each shadowed view's far cascade, as it was last drawn.
    far_shadows: [Option<FarShadow>; SHADOWED_VIEWS],
    /// For tests: the sun's shadow maps and the lights' cubes drawn last frame.
    pub shadows_drawn: (usize, usize),
    /// Counts the far cascade's checks, for the governor's rota.
    frame_count: u64,
    /// What the preset asks of the frame.
    pub quality: crate::Quality,
}

/// A gust that writes one of the wind's sentences across the grass: which, where its
/// line starts, the way it runs, and its letters' width and depth, in metres.
#[derive(Clone, Copy, Debug)]
struct Gust {
    window: u64,
    sentence: u32,
    origin: glam::Vec2,
    along: glam::Vec2,
    width: f32,
    depth: f32,
    start: f32,
}

/// Seconds a gust takes to write its sentence, hold it, and lift it away.
const GUST_SECONDS: f32 = 3.2;
/// Each stretch of this many seconds has at most one gust, and most have none.
const GUST_WINDOW: f32 = 150.0;
/// A sentence is written where the eye's gaze meets the grass, no nearer than this, and
/// sized to be read there: its letters subtend about the same angle at any distance, and
/// run long away from the eye as much as the eye looks across rather than down on them.
const GUST_NEAREST: f32 = 14.0;

/// A sentence laid across the grass where a view is looking, to run left to right as the
/// view sees it, its letters' tops away from the eye. None if the view looks at no grass.
fn gust_ahead(view: &crate::View, grass: &crate::grass::GrassField, sentence: u32, chars: usize, window: u64, start: f32) -> Option<Gust> {
    let eye = view.camera.eye;
    let forward = view.camera.rotation * glam::Vec3::NEG_Z;
    let f = glam::Vec2::new(forward.x, forward.z).try_normalize()?;
    let level = glam::Vec3::new(f.x, 0.0, f.y);
    // Where the gaze meets the grass, a little below the middle of the view, where the
    // eye rests on the ground ahead; if it meets none, the grass straight ahead.
    let gaze = (view.camera.rotation * glam::Vec3::new(0.0, -(view.camera.fov_y * 0.18).tan(), -1.0)).normalize();
    let hit = grass
        .ground_hit(eye, gaze)
        .or_else(|| grass.ground_hit(eye + level * GUST_NEAREST + glam::Vec3::Y * 50.0, -glam::Vec3::Y))?;
    let mut centre = hit;
    let mut distance = glam::Vec2::new(centre.x - eye.x, centre.z - eye.z).length();
    if distance < GUST_NEAREST {
        centre = grass.ground_hit(eye + level * GUST_NEAREST + glam::Vec3::Y * 50.0, -glam::Vec3::Y)?;
        distance = GUST_NEAREST;
    }
    let along = glam::Vec2::new(-f.y, f.x);
    let half = distance * (view.camera.fov_y * 0.5).tan() * 16.0 / 9.0;
    let width = (distance * 0.09).min(half * 2.0 * 0.85 / chars.max(1) as f32).max(0.8);
    let down = ((eye.y - centre.y) / distance.max(1.0)).max(0.07);
    let depth = (width * 0.8 / down).min(40.0);
    let length = chars as f32 * width;
    let origin = glam::Vec2::new(centre.x, centre.z) - along * length * 0.5 - f * depth * 0.5;
    Some(Gust { window, sentence, origin, along, width, depth, start })
}

/// A far cascade as it was last drawn.
#[derive(Clone, Copy, Debug)]
struct FarShadow {
    matrix: Mat4,
    light: Vec3,
    cell: glam::IVec3,
    statics: u64,
    frame: u64,
}

fn aligned(size: u64, align: u64) -> u64 {
    size.div_ceil(align) * align
}

fn uniform_entry(binding: u32, visibility: wgpu::ShaderStages, dynamic: bool, size: u64) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry {
        binding,
        visibility,
        ty: wgpu::BindingType::Buffer {
            ty: wgpu::BufferBindingType::Uniform,
            has_dynamic_offset: dynamic,
            min_binding_size: wgpu::BufferSize::new(size),
        },
        count: None,
    }
}

impl ScenePass {
    pub fn new(
        gpu: &Gpu,
        registry: &mut PipelineRegistry,
        shaders: &ShaderLibrary,
        surface: &SurfaceResources,
        target_format: wgpu::TextureFormat,
        grade: vellmark_content::Grade,
        wind_sentences: Vec<String>,
    ) -> Result<Self, String> {
        let device = &gpu.device;
        let align = device.limits().min_uniform_buffer_offset_alignment as u64;
        let view_size = std::mem::size_of::<ViewUniform>() as u64;
        let view_stride = aligned(view_size, align);
        let object_stride = aligned(16, align);
        let cascade_stride = aligned(CASCADE_UNIFORM, align);

        let frame_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("frame"),
            entries: &[
                uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT, true, view_size),
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                uniform_entry(3, wgpu::ShaderStages::FRAGMENT, false, 0),
                wgpu::BindGroupLayoutEntry {
                    binding: 4,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Uint,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 5,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::CubeArray,
                        multisampled: false,
                    },
                    count: None,
                },
                // The fog volume, summed along each line of sight, and its sampler.
                wgpu::BindGroupLayoutEntry {
                    binding: 6,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 7,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                // Contact occlusion, at half resolution.
                wgpu::BindGroupLayoutEntry {
                    binding: 8,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });
        let lights = Lights::new(gpu);
        let bloom_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("bloom"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        // The instances: every placed thing, the lists of which to draw, and where in a
        // list one batch starts.
        let storage = |binding, visibility| wgpu::BindGroupLayoutEntry {
            binding,
            visibility,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only: true }, has_dynamic_offset: false, min_binding_size: None },
            count: None,
        };
        let object_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("instances"),
            entries: &[
                storage(0, wgpu::ShaderStages::VERTEX_FRAGMENT),
                storage(1, wgpu::ShaderStages::VERTEX),
                // The batch: the trees' fragments read their level and leaves from it.
                uniform_entry(2, wgpu::ShaderStages::VERTEX_FRAGMENT, true, 16),
            ],
        });
        let cascade_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cascade"),
            entries: &[uniform_entry(0, wgpu::ShaderStages::VERTEX, true, CASCADE_UNIFORM)],
        });
        let post_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("post"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                uniform_entry(2, wgpu::ShaderStages::FRAGMENT, false, 64),
                wgpu::BindGroupLayoutEntry {
                    binding: 3,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
            ],
        });

        let buffer = |label, size| {
            device.create_buffer(&wgpu::BufferDescriptor {
                label: Some(label),
                size,
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            })
        };
        let view_buffer = buffer("views", view_stride * MAX_VIEWS as u64);
        let instance_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("instances"),
            size: (MAX_INSTANCES * std::mem::size_of::<InstanceGpu>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // One list of every instance for the shadows, then one for each view of what it sees.
        let visible_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("visible instances"),
            size: (MAX_INSTANCES * (1 + MAX_VIEWS) * 4) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let draw_buffer = buffer("draw batches", object_stride * ((1 + MAX_VIEWS) * MAX_GROUPS) as u64);
        let cascade_buffer = buffer("cascades", cascade_stride * (CASCADES * SHADOWED_VIEWS + CUBE_FACES) as u64);
        let post_buffer = buffer("post", 64);

        let one = |label, layout: &wgpu::BindGroupLayout, buffer: &wgpu::Buffer, size| {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some(label),
                layout,
                entries: &[wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer, offset: 0, size: wgpu::BufferSize::new(size) }),
                }],
            })
        };
        let object_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("instances"),
            layout: &object_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: instance_buffer.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: visible_buffer.as_entire_binding() },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: &draw_buffer, offset: 0, size: wgpu::BufferSize::new(16) }),
                },
            ],
        });
        let cascade_bind_group = one("cascade", &cascade_layout, &cascade_buffer, CASCADE_UNIFORM);

        // Shadow maps: three cascades for each shadowed view, in one array. Each view's
        // three layers are bound as their own array.
        let shadow_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("sun shadows"),
            size: wgpu::Extent3d {
                width: SHADOW_SIZE,
                height: SHADOW_SIZE,
                depth_or_array_layers: (CASCADES * SHADOWED_VIEWS) as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: DEPTH_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let shadow_layers = (0..(CASCADES * SHADOWED_VIEWS) as u32)
            .map(|layer| {
                shadow_texture.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_array_layer: layer,
                    array_layer_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();
        let shadow_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("shadow"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            compare: Some(wgpu::CompareFunction::LessEqual),
            ..Default::default()
        });
        let shadow_arrays: Vec<wgpu::TextureView> = (0..SHADOWED_VIEWS as u32)
            .map(|v| {
                shadow_texture.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    base_array_layer: v * CASCADES as u32,
                    array_layer_count: Some(CASCADES as u32),
                    ..Default::default()
                })
            })
            .collect();
        let cull_module = shaders.module(gpu, "cull")?;
        let pyramid_module = shaders.module(gpu, "pyramid")?;
        let culling = crate::cull::Culling::new(gpu, registry, &cull_module, &pyramid_module);
        let contact_module = shaders.module(gpu, "contact")?;
        let contact = crate::contact::ContactOcclusion::new(gpu, registry, &contact_module, view_size);
        let fog_module = shaders.module(gpu, "fog")?;
        let fog = crate::fog::FogVolume::new(gpu, registry, &fog_module, SHADOWED_VIEWS, &view_buffer, view_size, &shadow_arrays, &shadow_sampler);
        let frame_bind_groups = (0..SHADOWED_VIEWS)
            .map(|v| {
                let array = &shadow_arrays[v];
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("frame"),
                    layout: &frame_layout,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                                buffer: &view_buffer,
                                offset: 0,
                                size: wgpu::BufferSize::new(view_size),
                            }),
                        },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(array) },
                        wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&shadow_sampler) },
                        wgpu::BindGroupEntry { binding: 3, resource: lights.uniform.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&lights.cluster_view) },
                        wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&lights.cube_view) },
                        wgpu::BindGroupEntry { binding: 6, resource: wgpu::BindingResource::TextureView(&fog.integrated_views[v]) },
                        wgpu::BindGroupEntry { binding: 7, resource: wgpu::BindingResource::Sampler(&fog.sampler) },
                        wgpu::BindGroupEntry { binding: 8, resource: wgpu::BindingResource::TextureView(&contact.view) },
                    ],
                })
            })
            .collect();
        let post_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("post"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let terrain_layout = crate::terrain::layout(&gpu.device);
        let water_layout = crate::terrain::water_layout(&gpu.device);
        let water_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("water"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        let pipelines = Self::pipelines(
            gpu, registry, shaders, surface, target_format, &frame_layout, &object_layout, &cascade_layout, &post_layout, &bloom_layout, &terrain_layout, &water_layout,
        )?;
        Ok(Self {
            pipelines,
            frame_layout,
            object_layout,
            cascade_layout,
            post_layout,
            bloom_layout,
            terrain_layout,
            water_layout,
            water_sampler,
            water_group: None,
            lights,
            view_buffer,
            view_stride,
            instance_buffer,
            visible_buffer,
            draw_buffer,
            object_stride,
            object_bind_group,
            cascade_buffer,
            cascade_stride,
            cascade_bind_group,
            post_buffer,
            grade,
            wind_sentences,
            leaves: surface.leaves,
            gust: None,
            post_sampler,
            shadow_sampler,
            shadow_layers,
            frame_bind_groups,
            fog,
            contact,
            culling,
            view_size,
            targets: None,
            target_format,
            letter_buffer: gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("loose letters"),
                size: (vellmark_sim::debris::MAX_LETTERS * std::mem::size_of::<LetterInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            letters_sent: None,
            grass_sent: None,
            far_shadows: [None; SHADOWED_VIEWS],
            shadows_drawn: (0, 0),
            frame_count: 0,
            quality: crate::Quality::default(),
        })
    }

    #[allow(clippy::too_many_arguments)]
    fn pipelines(
        gpu: &Gpu,
        registry: &mut PipelineRegistry,
        shaders: &ShaderLibrary,
        surface: &SurfaceResources,
        target_format: wgpu::TextureFormat,
        frame_layout: &wgpu::BindGroupLayout,
        object_layout: &wgpu::BindGroupLayout,
        cascade_layout: &wgpu::BindGroupLayout,
        post_layout: &wgpu::BindGroupLayout,
        bloom_layout: &wgpu::BindGroupLayout,
        terrain_layout: &wgpu::BindGroupLayout,
        water_layout: &wgpu::BindGroupLayout,
    ) -> Result<Pipelines, String> {
        let device = &gpu.device;
        let opaque_module = shaders.module(gpu, "opaque")?;
        let shadow_module = shaders.module(gpu, "shadow")?;
        let post_module = shaders.module(gpu, "post")?;
        let bloom_module = shaders.module(gpu, "bloom")?;

        let layout = |label, groups: &[Option<&wgpu::BindGroupLayout>]| {
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some(label),
                bind_group_layouts: groups,
                immediate_size: 0,
            })
        };
        let world_layout = layout("world", &[Some(frame_layout), Some(&surface.layout), Some(object_layout)]);
        let shadow_layout = layout("shadow", &[Some(cascade_layout), Some(object_layout)]);
        let post_pipeline_layout = layout("post", &[Some(post_layout)]);

        let multisample = wgpu::MultisampleState { count: SAMPLES, mask: !0, alpha_to_coverage_enabled: false };
        let solid = wgpu::PrimitiveState {
            front_face: wgpu::FrontFace::Ccw,
            cull_mode: Some(wgpu::Face::Back),
            ..Default::default()
        };
        let depth_state = |write: bool, compare| wgpu::DepthStencilState {
            format: DEPTH_FORMAT,
            depth_write_enabled: Some(write),
            depth_compare: Some(compare),
            stencil: Default::default(),
            bias: Default::default(),
        };

        let shadow = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("sun shadow"),
                layout: Some(&shadow_layout),
                vertex: wgpu::VertexState {
                    module: &shadow_module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[Some(vertex_layout())],
                },
                fragment: None,
                // Thin things like a hanging cloth must cast from both sides.
                primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: Default::default(),
                    bias: wgpu::DepthBiasState { constant: 2, slope_scale: 2.5, clamp: 0.0 },
                }),
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            },
        );
        let depth = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("depth pre-pass"),
                layout: Some(&world_layout),
                vertex: wgpu::VertexState {
                    module: &opaque_module,
                    entry_point: Some("vs_depth"),
                    compilation_options: Default::default(),
                    buffers: &[Some(vertex_layout())],
                },
                fragment: None,
                primitive: solid,
                // Reversed depth: nearer is greater.
                depth_stencil: Some(depth_state(true, wgpu::CompareFunction::Greater)),
                multisample,
                multiview_mask: None,
                cache: None,
            },
        );
        let opaque = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("opaque"),
                layout: Some(&world_layout),
                vertex: wgpu::VertexState {
                    module: &opaque_module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[Some(vertex_layout())],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &opaque_module,
                    entry_point: Some("fs"),
                    compilation_options: Default::default(),
                    targets: &[Some(HDR_FORMAT.into())],
                }),
                primitive: solid,
                // The pre-pass has already decided what is in front; only that is shaded.
                depth_stencil: Some(depth_state(false, wgpu::CompareFunction::Equal)),
                multisample,
                multiview_mask: None,
                cache: None,
            },
        );
        let sky = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("sky"),
                layout: Some(&world_layout),
                vertex: wgpu::VertexState {
                    module: &opaque_module,
                    entry_point: Some("vs_sky"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &opaque_module,
                    entry_point: Some("fs_sky"),
                    compilation_options: Default::default(),
                    targets: &[Some(HDR_FORMAT.into())],
                }),
                primitive: Default::default(),
                // Only where nothing was drawn: depth still at its cleared value.
                depth_stencil: Some(depth_state(false, wgpu::CompareFunction::Equal)),
                multisample,
                multiview_mask: None,
                cache: None,
            },
        );
        // Trees: drawn from both sides, for their leaf cards, and cut in the fragment stage
        // to their leaves, their crossfades, and their far form's picture.
        let both_sides = wgpu::PrimitiveState { cull_mode: None, ..Default::default() };
        let tree_shadow = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("tree shadow"),
                layout: Some(&shadow_layout),
                vertex: wgpu::VertexState {
                    module: &shadow_module,
                    entry_point: Some("vs_tree"),
                    compilation_options: Default::default(),
                    buffers: &[Some(vertex_layout())],
                },
                fragment: Some(wgpu::FragmentState { module: &shadow_module, entry_point: Some("fs_tree"), compilation_options: Default::default(), targets: &[] }),
                primitive: both_sides,
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: Default::default(),
                    bias: wgpu::DepthBiasState { constant: 2, slope_scale: 2.5, clamp: 0.0 },
                }),
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            },
        );
        let tree_depth = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("tree depth"),
                layout: Some(&world_layout),
                vertex: wgpu::VertexState {
                    module: &opaque_module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[Some(vertex_layout())],
                },
                fragment: Some(wgpu::FragmentState { module: &opaque_module, entry_point: Some("fs_tree_depth"), compilation_options: Default::default(), targets: &[] }),
                primitive: both_sides,
                depth_stencil: Some(depth_state(true, wgpu::CompareFunction::Greater)),
                multisample,
                multiview_mask: None,
                cache: None,
            },
        );
        let tree = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("tree"),
                layout: Some(&world_layout),
                vertex: wgpu::VertexState {
                    module: &opaque_module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[Some(vertex_layout())],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &opaque_module,
                    entry_point: Some("fs_tree"),
                    compilation_options: Default::default(),
                    targets: &[Some(HDR_FORMAT.into())],
                }),
                primitive: both_sides,
                depth_stencil: Some(depth_state(false, wgpu::CompareFunction::Equal)),
                multisample,
                multiview_mask: None,
                cache: None,
            },
        );
        let letter_shadow_layout = layout("letter shadow", &[Some(cascade_layout)]);
        let letter_shadow = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("letter shadow"),
                layout: Some(&letter_shadow_layout),
                vertex: wgpu::VertexState {
                    module: &shadow_module,
                    entry_point: Some("vs_letter"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<LetterInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &LETTER_ATTRIBUTES,
                    })],
                },
                fragment: None,
                primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: Default::default(),
                    bias: wgpu::DepthBiasState { constant: 2, slope_scale: 2.5, clamp: 0.0 },
                }),
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            },
        );
        // The terrain: its own vertex stage, which reads the heights, and its own layouts,
        // with the terrain's bind group after the object's.
        let terrain_world_layout = layout("terrain", &[Some(frame_layout), Some(&surface.layout), Some(object_layout), Some(terrain_layout)]);
        let terrain_shadow_layout = layout("terrain shadow", &[Some(cascade_layout), Some(object_layout), Some(terrain_layout)]);
        let terrain_shadow = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("terrain shadow"),
                layout: Some(&terrain_shadow_layout),
                vertex: wgpu::VertexState { module: &shadow_module, entry_point: Some("vs_terrain"), compilation_options: Default::default(), buffers: &[] },
                fragment: Some(wgpu::FragmentState { module: &shadow_module, entry_point: Some("fs_terrain"), compilation_options: Default::default(), targets: &[] }),
                primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
                depth_stencil: Some(wgpu::DepthStencilState {
                    format: DEPTH_FORMAT,
                    depth_write_enabled: Some(true),
                    depth_compare: Some(wgpu::CompareFunction::LessEqual),
                    stencil: Default::default(),
                    bias: wgpu::DepthBiasState { constant: 2, slope_scale: 2.5, clamp: 0.0 },
                }),
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            },
        );
        let terrain_depth = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("terrain depth"),
                layout: Some(&terrain_world_layout),
                vertex: wgpu::VertexState { module: &opaque_module, entry_point: Some("vs_terrain"), compilation_options: Default::default(), buffers: &[] },
                fragment: Some(wgpu::FragmentState { module: &opaque_module, entry_point: Some("fs_terrain_depth"), compilation_options: Default::default(), targets: &[] }),
                primitive: solid,
                depth_stencil: Some(depth_state(true, wgpu::CompareFunction::Greater)),
                multisample,
                multiview_mask: None,
                cache: None,
            },
        );
        let terrain = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("terrain"),
                layout: Some(&terrain_world_layout),
                vertex: wgpu::VertexState { module: &opaque_module, entry_point: Some("vs_terrain"), compilation_options: Default::default(), buffers: &[] },
                fragment: Some(wgpu::FragmentState {
                    module: &opaque_module,
                    entry_point: Some("fs_terrain"),
                    compilation_options: Default::default(),
                    targets: &[Some(HDR_FORMAT.into())],
                }),
                primitive: solid,
                depth_stencil: Some(depth_state(false, wgpu::CompareFunction::Equal)),
                multisample,
                multiview_mask: None,
                cache: None,
            },
        );
        // Grass: blades made in the vertex stage, seen from both sides.
        let grass_depth = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("grass depth"),
                layout: Some(&terrain_world_layout),
                vertex: wgpu::VertexState { module: &opaque_module, entry_point: Some("vs_grass"), compilation_options: Default::default(), buffers: &[] },
                fragment: Some(wgpu::FragmentState { module: &opaque_module, entry_point: Some("fs_grass_depth"), compilation_options: Default::default(), targets: &[] }),
                primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
                depth_stencil: Some(depth_state(true, wgpu::CompareFunction::Greater)),
                multisample,
                multiview_mask: None,
                cache: None,
            },
        );
        let grass = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("grass"),
                layout: Some(&terrain_world_layout),
                vertex: wgpu::VertexState { module: &opaque_module, entry_point: Some("vs_grass"), compilation_options: Default::default(), buffers: &[] },
                fragment: Some(wgpu::FragmentState {
                    module: &opaque_module,
                    entry_point: Some("fs_grass"),
                    compilation_options: Default::default(),
                    targets: &[Some(HDR_FORMAT.into())],
                }),
                primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
                depth_stencil: Some(depth_state(false, wgpu::CompareFunction::Equal)),
                multisample,
                multiview_mask: None,
                cache: None,
            },
        );
        // Water: after the opaque pass, into the resolved scene, reading a copy of it and
        // its depth. It tests depth itself, so it needs no depth target.
        let water_world_layout = layout("water", &[Some(frame_layout), Some(&surface.layout), Some(object_layout), Some(water_layout)]);
        let water = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("water"),
                layout: Some(&water_world_layout),
                vertex: wgpu::VertexState { module: &opaque_module, entry_point: Some("vs_water"), compilation_options: Default::default(), buffers: &[] },
                fragment: Some(wgpu::FragmentState {
                    module: &opaque_module,
                    entry_point: Some("fs_water"),
                    compilation_options: Default::default(),
                    targets: &[Some(HDR_FORMAT.into())],
                }),
                primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            },
        );
        // Rain: streaks and splashes over everything, after the water, blended.
        let rain = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("rain"),
                layout: Some(&water_world_layout),
                vertex: wgpu::VertexState { module: &opaque_module, entry_point: Some("vs_rain"), compilation_options: Default::default(), buffers: &[] },
                fragment: Some(wgpu::FragmentState {
                    module: &opaque_module,
                    entry_point: Some("fs_rain"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: HDR_FORMAT,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            },
        );
        let letters = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("loose letters"),
                layout: Some(&world_layout),
                vertex: wgpu::VertexState {
                    module: &opaque_module,
                    entry_point: Some("vs_letter"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<LetterInstance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &LETTER_ATTRIBUTES,
                    })],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &opaque_module,
                    entry_point: Some("fs_letter"),
                    compilation_options: Default::default(),
                    targets: &[Some(HDR_FORMAT.into())],
                }),
                // A letter is seen from both sides as it tumbles.
                primitive: wgpu::PrimitiveState { cull_mode: None, ..Default::default() },
                // Not in the pre-pass: tested against it and written as they are drawn.
                depth_stencil: Some(depth_state(true, wgpu::CompareFunction::Greater)),
                // The shape of the character is cut out by coverage, so its edge is smooth.
                multisample: wgpu::MultisampleState { count: SAMPLES, mask: !0, alpha_to_coverage_enabled: true },
                multiview_mask: None,
                cache: None,
            },
        );
        let post = registry.render_pipeline(
            device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("post"),
                layout: Some(&post_pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &post_module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &post_module,
                    entry_point: Some("fs"),
                    compilation_options: Default::default(),
                    targets: &[Some(target_format.into())],
                }),
                primitive: Default::default(),
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            },
        );
        let bloom_pipeline_layout = layout("bloom", &[Some(bloom_layout)]);
        let mut bloom = |label, entry, blend| {
            registry.render_pipeline(
                device,
                &wgpu::RenderPipelineDescriptor {
                    label: Some(label),
                    layout: Some(&bloom_pipeline_layout),
                    vertex: wgpu::VertexState {
                        module: &bloom_module,
                        entry_point: Some("vs"),
                        compilation_options: Default::default(),
                        buffers: &[],
                    },
                    fragment: Some(wgpu::FragmentState {
                        module: &bloom_module,
                        entry_point: Some(entry),
                        compilation_options: Default::default(),
                        targets: &[Some(wgpu::ColorTargetState { format: HDR_FORMAT, blend, write_mask: wgpu::ColorWrites::ALL })],
                    }),
                    primitive: Default::default(),
                    depth_stencil: None,
                    multisample: Default::default(),
                    multiview_mask: None,
                    cache: None,
                },
            )
        };
        let add = wgpu::BlendState {
            color: wgpu::BlendComponent { src_factor: wgpu::BlendFactor::One, dst_factor: wgpu::BlendFactor::One, operation: wgpu::BlendOperation::Add },
            alpha: wgpu::BlendComponent::REPLACE,
        };
        let bloom_first = bloom("bloom first", "fs_first", None);
        let bloom_down = bloom("bloom down", "fs_down", None);
        let bloom_up = bloom("bloom up", "fs_up", Some(add));
        Ok(Pipelines { shadow, depth, opaque, sky, letters, letter_shadow, terrain_shadow, terrain_depth, terrain, grass_depth, grass, tree_shadow, tree_depth, tree, water, rain, post, bloom_first, bloom_down, bloom_up })
    }

    /// Remakes the pipelines from the shaders as they now are. On an error nothing
    /// changes.
    pub fn reload(
        &mut self,
        gpu: &Gpu,
        registry: &mut PipelineRegistry,
        shaders: &ShaderLibrary,
        surface: &SurfaceResources,
    ) -> Result<(), String> {
        self.pipelines = Self::pipelines(
            gpu,
            registry,
            shaders,
            surface,
            self.target_format,
            &self.frame_layout,
            &self.object_layout,
            &self.cascade_layout,
            &self.post_layout,
            &self.bloom_layout,
            &self.terrain_layout,
            &self.water_layout,
        )?;
        Ok(())
    }

    /// The scene's render targets, remade when the output changes size. Textures are
    /// not pipelines: making them during play costs no stutter.
    fn targets(&mut self, gpu: &Gpu, size: (u32, u32)) -> &Targets {
        if self.targets.as_ref().is_none_or(|t| t.size != size) {
            let texture = |label, format, samples, usage| {
                gpu.device
                    .create_texture(&wgpu::TextureDescriptor {
                        label: Some(label),
                        size: wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
                        mip_level_count: 1,
                        sample_count: samples,
                        dimension: wgpu::TextureDimension::D2,
                        format,
                        usage,
                        view_formats: &[],
                    })
                    .create_view(&Default::default())
            };
            let attachment = wgpu::TextureUsages::RENDER_ATTACHMENT;
            let colour_msaa = texture("scene colour", HDR_FORMAT, SAMPLES, attachment);
            // Read by contact occlusion as well as drawn into.
            let depth_msaa = texture("scene depth", DEPTH_FORMAT, SAMPLES, attachment | wgpu::TextureUsages::TEXTURE_BINDING);
            let plain = |label| {
                gpu.device.create_texture(&wgpu::TextureDescriptor {
                    label: Some(label),
                    size: wgpu::Extent3d { width: size.0, height: size.1, depth_or_array_layers: 1 },
                    mip_level_count: 1,
                    sample_count: 1,
                    dimension: wgpu::TextureDimension::D2,
                    format: HDR_FORMAT,
                    usage: attachment | wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_SRC | wgpu::TextureUsages::COPY_DST,
                    view_formats: &[],
                })
            };
            let resolved_texture = plain("scene resolved");
            let resolved = resolved_texture.create_view(&Default::default());
            let behind_texture = plain("scene behind the water");
            let behind = behind_texture.create_view(&Default::default());
            self.water_group = None;
            let bloom_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
                label: Some("bloom"),
                size: wgpu::Extent3d { width: (size.0 / 2).max(16), height: (size.1 / 2).max(16), depth_or_array_layers: 1 },
                mip_level_count: BLOOM_LEVELS as u32,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: HDR_FORMAT,
                usage: attachment | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            });
            let bloom_levels: Vec<wgpu::TextureView> = (0..BLOOM_LEVELS as u32)
                .map(|level| {
                    bloom_texture.create_view(&wgpu::TextureViewDescriptor {
                        base_mip_level: level,
                        mip_level_count: Some(1),
                        ..Default::default()
                    })
                })
                .collect();
            let source = |view: &wgpu::TextureView| {
                gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("bloom source"),
                    layout: &self.bloom_layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(view) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.post_sampler) },
                    ],
                })
            };
            let bloom_sources = bloom_levels.iter().map(&source).collect();
            let scene_source = source(&resolved);
            let post_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("post"),
                layout: &self.post_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&resolved) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.post_sampler) },
                    wgpu::BindGroupEntry { binding: 2, resource: self.post_buffer.as_entire_binding() },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&bloom_levels[0]) },
                ],
            });
            let contact_group = self.contact.bind(gpu, &self.view_buffer, self.view_size, &depth_msaa);
            let pyramid = self.culling.targets(gpu, size, &depth_msaa, &self.instance_buffer, &self.visible_buffer);
            self.targets = Some(Targets { size, colour_msaa, depth_msaa, resolved, resolved_texture, behind_texture, behind, post_bind_group, bloom_levels, bloom_sources, scene_source, contact_group, pyramid });
        }
        self.targets.as_ref().unwrap()
    }

    fn write_cascade(&self, gpu: &Gpu, slot: usize, matrix: &Mat4, glowing: u32) {
        let mut bytes = [0u8; CASCADE_UNIFORM as usize];
        bytes[..64].copy_from_slice(bytemuck::cast_slice(&matrix.to_cols_array()));
        bytes[64..68].copy_from_slice(&glowing.to_le_bytes());
        gpu.queue.write_buffer(&self.cascade_buffer, slot as u64 * self.cascade_stride, &bytes);
    }

    /// Where batch `group` of list `list` starts, as a dynamic offset into the batches.
    /// List 0 is every instance; list 1 + v is what view v sees.
    fn batch_offset(&self, list: usize, group: usize) -> u32 {
        ((list * MAX_GROUPS + group) as u64 * self.object_stride) as u32
    }

    /// For tests: how many instances of each batch view 0 drew, read back from the draws.
    #[cfg(feature = "dev")]
    pub fn drawn_last_frame(&self, gpu: &Gpu, scene: &SceneGpu) -> Vec<(vellmark_content::AssetId, u32)> {
        let words = self.read_draws(gpu, scene.groups.len());
        scene
            .groups
            .iter()
            .enumerate()
            .map(|(g, group)| (scene.placed[group.first_instance as usize].key, words[g * 5 + 1]))
            .collect()
    }

    /// For tests: how many grass patches view 0 drew last frame at each level of detail.
    #[cfg(feature = "dev")]
    pub fn grass_drawn_last_frame(&self, gpu: &Gpu, scene: &SceneGpu) -> [u32; crate::grass::LEVELS] {
        let first = scene.groups.len();
        let words = self.read_draws(gpu, first + crate::grass::LEVELS);
        std::array::from_fn(|l| words[(first + l) * 5 + 1])
    }

    #[cfg(feature = "dev")]
    fn read_draws(&self, gpu: &Gpu, groups: usize) -> Vec<u32> {
        let size = (groups * 20) as u64;
        if size == 0 {
            return Vec::new();
        }
        let readback = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("draws, read back"),
            size,
            usage: wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::MAP_READ,
            mapped_at_creation: false,
        });
        let mut encoder = gpu.device.create_command_encoder(&Default::default());
        encoder.copy_buffer_to_buffer(&self.culling.draws, 0, &readback, 0, size);
        gpu.queue.submit([encoder.finish()]);
        let slice = readback.slice(..);
        slice.map_async(wgpu::MapMode::Read, |_| {});
        let _ = gpu.device.poll(wgpu::PollType::wait_indefinitely());
        let words: Vec<u32> = bytemuck::cast_slice(&slice.get_mapped_range().expect("mapped")).to_vec();
        words
    }

    /// Every mesh of the scene that view `v` can see, one indirect batch each, as the
    /// culling pass left them.
    /// A mesh with levels of detail is a tree's, drawn with `tree`; every other with `plain`.
    fn draw_scene_culled(&self, pass: &mut wgpu::RenderPass<'_>, scene: &SceneGpu, v: usize, object_group: u32, plain: &wgpu::RenderPipeline, tree: &wgpu::RenderPipeline) {
        let (Some(vertices), Some(indices)) = (&scene.vertices, &scene.indices) else { return };
        pass.set_vertex_buffer(0, vertices.slice(..));
        pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
        let mut current: Option<bool> = None;
        for (g, group) in scene.groups.iter().enumerate() {
            if group.count == 0 || scene.meshes[group.mesh].index_count == 0 {
                continue;
            }
            let leveled = group.levels != 1;
            if current != Some(leveled) {
                pass.set_pipeline(if leveled { tree } else { plain });
                current = Some(leveled);
            }
            pass.set_bind_group(object_group, &self.object_bind_group, &[self.batch_offset(1 + v, g)]);
            pass.draw_indexed_indirect(&self.culling.draws, crate::cull::Culling::draw_offset(v, g));
        }
    }

    /// Every mesh of the scene, one batch each, from list `list`, for the shadows. A mesh
    /// with levels of detail casts from one coarse level, with `tree`; every other is
    /// drawn with `plain`.
    fn draw_scene(&self, pass: &mut wgpu::RenderPass<'_>, scene: &SceneGpu, list: usize, object_group: u32, plain: &wgpu::RenderPipeline, tree: &wgpu::RenderPipeline) {
        let (Some(vertices), Some(indices)) = (&scene.vertices, &scene.indices) else { return };
        pass.set_vertex_buffer(0, vertices.slice(..));
        pass.set_index_buffer(indices.slice(..), wgpu::IndexFormat::Uint32);
        let mut current: Option<bool> = None;
        for (g, group) in scene.groups.iter().enumerate() {
            if group.is_coarser_level() {
                continue;
            }
            let leveled = group.levels > 1;
            // The third level's branches and cards, which is close enough for a shadow map.
            let mesh = &scene.meshes[scene.groups[g + if leveled { SHADOW_LEVEL.min(group.levels as usize - 1) } else { 0 }].mesh];
            if group.count == 0 || mesh.index_count == 0 {
                continue;
            }
            if current != Some(leveled) {
                pass.set_pipeline(if leveled { tree } else { plain });
                current = Some(leveled);
            }
            pass.set_bind_group(object_group, &self.object_bind_group, &[self.batch_offset(list, g)]);
            pass.draw_indexed(mesh.first_index..mesh.first_index + mesh.index_count, mesh.base_vertex, 0..group.count);
        }
    }

    /// The grass's density has changed: choose and send its patches afresh.
    pub fn forget_grass_patches(&mut self) {
        self.grass_sent = None;
    }

    /// The terrain has been replaced: send its grass afresh and bind its water anew.
    pub fn forget_terrain(&mut self) {
        self.grass_sent = None;
        self.water_group = None;
        self.far_shadows = [None; SHADOWED_VIEWS];
    }

    /// The far cascade of shadowed view `s`, and whether it must be drawn this frame.
    ///
    /// It covers everything within its reach of the eye, whichever way the eye looks, so
    /// turning never changes it. It is drawn once and kept until the light has moved by a
    /// quarter of a degree, the eye has crossed into another cell of a grid an eighth of
    /// its reach, or a still thing has changed.
    fn far_cascade(&mut self, s: usize, eye: Vec3, light: Vec3, statics: u64, relief: &crate::Relief) -> (Mat4, bool) {
        let reach = CASCADE_FAR[CASCADES - 1];
        let step = reach / 8.0;
        let cell = (eye / step).floor().as_ivec3();
        self.frame_count += 1;
        if let Some(far) = self.far_shadows[s] {
            // The governor may let the light move further, and less often, before a redraw.
            let light_held = far.light.dot(light) > relief.far_shadow_degrees().to_radians().cos()
                || self.frame_count - far.frame < relief.far_shadow_frames() as u64;
            if far.cell == cell && far.statics == statics && light_held {
                return (far.matrix, false);
            }
        }
        let centre = (cell.as_vec3() + 0.5) * step;
        // Reach from the cell's middle to any eye within it, and the whole cascade beyond.
        let radius = reach + step * 0.9;
        let matrix = crate::frame::sphere_cascade_matrix(centre, radius, light);
        self.far_shadows[s] = Some(FarShadow { matrix, light, cell, statics, frame: self.frame_count });
        (matrix, true)
    }

    pub fn terrain_layout(&self) -> &wgpu::BindGroupLayout {
        &self.terrain_layout
    }

    /// The terrain, with the pipeline already set and the frame or cascade bound.
    fn draw_terrain(&self, pass: &mut wgpu::RenderPass<'_>, terrain: Option<&crate::terrain::TerrainGpu>, object_group: u32) {
        if let Some(t) = terrain {
            pass.set_bind_group(object_group, &self.object_bind_group, &[0]);
            pass.set_bind_group(object_group + 1, &t.bind_group, &[]);
            pass.set_index_buffer(t.indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..t.index_count, 0, 0..t.tile_count);
        }
    }

    /// The grass view `v` can see, each level of detail one indirect batch, with the
    /// pipeline already set and the frame bound.
    fn draw_grass(&self, pass: &mut wgpu::RenderPass<'_>, terrain: Option<&crate::terrain::TerrainGpu>, v: usize, first_group: usize, counts: &[u32; crate::grass::LEVELS]) {
        let Some(t) = terrain else { return };
        pass.set_bind_group(3, &t.bind_group, &[]);
        pass.set_index_buffer(t.grass.indices.slice(..), wgpu::IndexFormat::Uint32);
        for (l, count) in counts.iter().enumerate() {
            if *count == 0 {
                continue;
            }
            pass.set_bind_group(2, &self.object_bind_group, &[self.batch_offset(1 + v, first_group + l)]);
            pass.draw_indexed_indirect(&self.culling.draws, crate::cull::Culling::draw_offset(v, first_group + l));
        }
    }

    /// Loose letters into a shadow map whose cascade or face is already bound.
    fn draw_letter_shadows(&self, pass: &mut wgpu::RenderPass<'_>, count: usize) {
        if count > 0 {
            pass.set_pipeline(&self.pipelines.letter_shadow);
            pass.set_vertex_buffer(0, self.letter_buffer.slice(..));
            pass.draw(0..6, 0..count as u32);
        }
    }

    #[allow(clippy::too_many_arguments)]
    pub fn render(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        target_size: (u32, u32),
        frame: &FrameInput<'_>,
        scene: &SceneGpu,
        surface: &SurfaceResources,
        terrain: Option<&crate::terrain::TerrainGpu>,
        timers: &mut PassTimers,
    ) {
        // The scene is drawn at the governor's share of the target, in its top left
        // corner; the post pass stretches it to fill the screen.
        let scale = frame.relief.scene_scale();
        let scene_size = (
            ((target_size.0 as f32 * scale).round() as u32).clamp(1, target_size.0),
            ((target_size.1 as f32 * scale).round() as u32).clamp(1, target_size.1),
        );
        let _ = &self.shadow_sampler;
        let mut atmosphere = Atmosphere::at(frame.time_of_day);
        let air = self.grade.at(frame.time_of_day);
        atmosphere.fog = [air.fog_density, air.fog_floor, air.fog_falloff];
        let atmosphere = atmosphere.with_rain(frame.weather.rain).with_fog(frame.weather.fog);
        let views = &frame.views[..frame.views.len().min(MAX_VIEWS)];
        let letter_count = frame.letters.len().min(vellmark_sim::debris::MAX_LETTERS);
        if self.letters_sent != Some((frame.letters_step, letter_count)) {
            let instances: Vec<LetterInstance> = frame.letters[..letter_count]
                .iter()
                .map(|l| LetterInstance {
                    position: l.position.to_array(),
                    rotation: l.rotation.to_array(),
                    material: l.material,
                    seed: l.seed,
                })
                .collect();
            if !instances.is_empty() {
                gpu.queue.write_buffer(&self.letter_buffer, 0, bytemuck::cast_slice(&instances));
            }
            self.letters_sent = Some((frame.letters_step, letter_count));
        }

        // The instances, with this frame's heat and breakage; the list of every one; and
        // where each batch starts in each list.
        let instances = scene.instances(frame.heat, frame.broken);
        if !instances.is_empty() {
            gpu.queue.write_buffer(&self.instance_buffer, 0, bytemuck::cast_slice(&instances));
            // The shadows draw every placed thing; each view's own list is written by culling.
            let every: Vec<u32> = (0..instances.len() as u32).collect();
            gpu.queue.write_buffer(&self.visible_buffer, 0, bytemuck::cast_slice(&every));
            for list in 0..=views.len().min(MAX_VIEWS) {
                // List 0 holds every instance in order, so a batch starts where its group's
                // instances do; a view's list has a place for every level of detail.
                let mut finest = 0;
                for (g, group) in scene.groups.iter().enumerate() {
                    if !group.is_coarser_level() {
                        finest = g;
                    }
                    let start = if list == 0 { group.first_instance } else { group.list_base };
                    // x: where the batch starts. y: its level of detail. z: the leaf cards'
                    // material, from 1. w: how many levels its mesh has.
                    let base = [(list * MAX_INSTANCES) as u32 + start, (g - finest) as u32, self.leaves, scene.groups[finest].levels];
                    gpu.queue.write_buffer(&self.draw_buffer, self.batch_offset(list, g) as u64, bytemuck::cast_slice(&base));
                }
            }
        }
        // The grass patches near any view, after the placed things, one batch for each
        // level of detail. They are sent again only when the patches change.
        let mut headers: Vec<[u32; 3]> = scene.groups.iter().map(|g| {
            let mesh = &scene.meshes[g.mesh];
            [mesh.index_count, mesh.first_index, mesh.base_vertex as u32]
        }).collect();
        let grass_group = scene.groups.len();
        let mut grass_counts = [0u32; crate::grass::LEVELS];
        let mut instance_count = instances.len();
        if let Some(t) = terrain.filter(|_| grass_group + crate::grass::LEVELS <= MAX_GROUPS) {
            let eyes: Vec<glam::Vec3> = views.iter().map(|v| v.camera.eye).collect();
            // Grass follows every place the scene's batches take in a list, the trees'
            // coarser levels included.
            let list_end = scene.list_end.max(instances.len() as u32);
            let chosen = t.grass.choose(&eyes, MAX_INSTANCES.saturating_sub(list_end as usize));
            let (patches, counts) = t.grass.instances(&chosen, list_end, grass_group as u32);
            grass_counts = counts;
            instance_count += patches.len();
            headers.extend_from_slice(&t.grass.draw_headers());
            let key = (instances.len(), chosen);
            if self.grass_sent.as_ref() != Some(&key) {
                if !patches.is_empty() {
                    let offset = (instances.len() * std::mem::size_of::<crate::world_gpu::InstanceGpu>()) as u64;
                    gpu.queue.write_buffer(&self.instance_buffer, offset, bytemuck::cast_slice(&patches));
                }
                self.grass_sent = Some(key);
            }
            let mut start = list_end;
            for (l, count) in counts.iter().enumerate() {
                for list in 1..=views.len().min(MAX_VIEWS) {
                    let base = [(list * MAX_INSTANCES) as u32 + start, 0, 0, 0];
                    gpu.queue.write_buffer(&self.draw_buffer, self.batch_offset(list, grass_group + l) as u64, bytemuck::cast_slice(&base));
                }
                start += count;
            }
            gpu.queue.write_buffer(&t.grass_uniform, 0, bytemuck::bytes_of(&t.grass.uniform(frame.seconds, &eyes, &frame.weather, &self.wind_sentences)));
        }
        // What casts shadows, as a number that changes whenever any of it does: the
        // still things (the scene, the terrain, what is broken) and, for the lights' cubes,
        // the loose letters too.
        let statics = {
            let mut h = vellmark_sim::Fnv64::new();
            h.u64(scene.placed.len() as u64);
            h.u64(terrain.map_or(0, |t| t.tile_count as u64));
            for b in frame.broken {
                h.u64(*b);
            }
            h.finish()
        };
        let generation = statics ^ frame.letters_step.wrapping_mul(0x9e37_79b9_7f4a_7c15);
        let mut redraw_layers = [false; SHADOWED_VIEWS * CASCADES];
        self.shadows_drawn = (0, 0);
        // The wind that reads. Now and then, in a strong enough wind over grass, a gust
        // writes one of its sentences across the field ahead of the first view. Each
        // stretch of time has at most one, at a time fixed by the clock, so a capture or
        // a replay sees the same gusts. A tool may ask for one, written whole.
        let mut reading: Option<(Gust, f32)> = None;
        if let (Some(t), Some(first), false) = (terrain, views.first(), self.wind_sentences.is_empty()) {
            let count = self.wind_sentences.len() as u32;
            if let Some((n, progress)) = frame.wind_gust {
                // Asked for by a tool: placed when it begins, then held where it is.
                let n = n % count;
                let stale = self.gust.is_none_or(|g| g.window != u64::MAX || g.sentence != n || progress < g.start);
                if stale {
                    self.gust = gust_ahead(first, &t.grass, n, self.wind_sentences[n as usize].len(), u64::MAX, progress);
                }
                if let Some(g) = self.gust.as_mut() {
                    g.start = progress;
                    reading = Some((*g, progress));
                }
            } else {
                let window = (frame.seconds / GUST_WINDOW).floor().max(0.0) as u64;
                let h = (window.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ 0x51ed_270b).wrapping_mul(0xff51_afd7_ed55_8ccd);
                let start = window as f32 * GUST_WINDOW + 20.0 + ((h >> 20) % 100) as f32;
                let windy = frame.weather.wind_strength > 0.3;
                if h % 10 < 6 && windy && frame.seconds >= start && frame.seconds < start + GUST_SECONDS {
                    if self.gust.is_none_or(|g| g.window != window) {
                        let n = ((h >> 40) % count as u64) as u32;
                        self.gust = gust_ahead(first, &t.grass, n, self.wind_sentences[n as usize].len(), window, start);
                    }
                    if let Some(g) = self.gust.filter(|g| g.window == window) {
                        reading = Some((g, (frame.seconds - g.start) / GUST_SECONDS));
                    }
                }
            }
        }
        for (i, view) in views.iter().enumerate() {
            let (_, _, w, h) = view.viewport.pixels(scene_size);
            let source = if i < SHADOWED_VIEWS { view } else { &views[0] };
            let (_, _, sw, sh) = source.viewport.pixels(scene_size);
            let mut cascades = [Mat4::IDENTITY; CASCADES];
            let mut near = source.camera.near;
            for (c, slot) in cascades.iter_mut().enumerate() {
                if c == CASCADES - 1 {
                    // The far cascade is kept from frame to frame: see `far_cascade`.
                    let s = i.min(SHADOWED_VIEWS - 1);
                    let (matrix, redraw) = self.far_cascade(s, source.camera.eye, atmosphere.light_dir, statics, &frame.relief);
                    *slot = matrix;
                    if i < SHADOWED_VIEWS && redraw {
                        self.write_cascade(gpu, i * CASCADES + c, slot, surface.glowing);
                        redraw_layers[i * CASCADES + c] = true;
                    }
                    continue;
                }
                *slot = cascade_matrix(source, sw / sh, near, CASCADE_FAR[c], atmosphere.light_dir);
                near = CASCADE_FAR[c] * 0.9;
                if i < SHADOWED_VIEWS {
                    self.write_cascade(gpu, i * CASCADES + c, slot, surface.glowing);
                    redraw_layers[i * CASCADES + c] = true;
                }
            }
            let uniform = view_uniform(
                view,
                (w, h),
                &atmosphere,
                frame.time_of_day,
                frame.seconds,
                frame.glyph_layer,
                frame.close_reading,
                &frame.tuning,
                frame.reveal_mask,
                i as u32,
                {
                    let (x, y, _, _) = view.viewport.pixels(scene_size);
                    (x, y)
                },
                frame.dive,
                frame.lamps,
                &cascades,
            );
            let mut uniform = uniform;
            let w = &frame.weather;
            uniform.weather = [w.rain, w.wetness, w.wind_strength, 0.0];
            let r = &frame.relief;
            uniform.quality = [r.far_grass(), r.contact(), r.fog_grid(), r.particles() * self.quality.particles];
            uniform.wind = [w.wind[0], w.wind[1], 0.0, 0.0];
            if let Some((g, progress)) = reading {
                uniform.sentence = [g.origin.x, g.origin.y, g.along.x, g.along.y];
                uniform.gust = [progress, g.sentence as f32, g.width, g.depth];
            }
            if let Some(t) = terrain {
                uniform.ground_tiles = t.tiles_around(view.camera.eye.x, view.camera.eye.z);
            }
            gpu.queue.write_buffer(&self.view_buffer, i as u64 * self.view_stride, bytemuck::bytes_of(&uniform));
        }
        // In the reveal mask nothing may bleed from one pixel to the next.
        let (bloom, edge) = if frame.reveal_mask { (0.0, 0.0) } else { (0.045, 1.0) };
        let g = self.grade.at(frame.time_of_day);
        let post: [f32; 16] = [
            atmosphere.exposure, bloom, edge, if frame.reveal_mask { 1.0 } else { 0.0 },
            g.lift[0], g.lift[1], g.lift[2], g.saturation,
            g.gamma[0], g.gamma[1], g.gamma[2], scene_size.0 as f32 / target_size.0 as f32,
            g.gain[0], g.gain[1], g.gain[2], scene_size.1 as f32 / target_size.1 as f32,
        ];
        gpu.queue.write_buffer(&self.post_buffer, 0, bytemuck::cast_slice(&post));

        let aspects: Vec<f32> = views
            .iter()
            .map(|v| {
                let (_, _, w, h) = v.viewport.pixels(scene_size);
                w / h
            })
            .collect();
        self.lights.update(gpu, &scene.lights, views, &aspects, generation, self.quality.shadowed_lights);
        if let Some((_, position, radius)) = self.lights.draw {
            self.shadows_drawn.1 += 1;
            for face in 0..CUBE_FACES {
                let matrix = cube_face_matrix(position, radius, face);
                self.write_cascade(gpu, CASCADES * SHADOWED_VIEWS + face, &matrix, surface.glowing);
            }
        }

        // Sun shadows, one pass for each cascade of each shadowed view. Only the first
        // is timed; the timer's name covers them all in the report.
        for layer in 0..views.len().min(SHADOWED_VIEWS) * CASCADES {
            if !redraw_layers[layer] {
                continue;
            }
            self.shadows_drawn.0 += 1;
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("shadows"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.shadow_layers[layer],
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: timers.render_pass(["shadows", "shadows 2", "shadows 3", "shadows 4", "shadows 5", "shadows 6"][layer]),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(0, &self.cascade_bind_group, &[(layer as u64 * self.cascade_stride) as u32]);
            self.draw_scene(&mut pass, scene, 0, 1, &self.pipelines.shadow, &self.pipelines.tree_shadow);
            self.draw_letter_shadows(&mut pass, letter_count);
            if terrain.is_some() {
                pass.set_pipeline(&self.pipelines.terrain_shadow);
                self.draw_terrain(&mut pass, terrain, 1);
            }
        }

        // A shadowed lantern or fire whose cube is missing or out of date: six faces of
        // a cube around it. The other cubes are kept as they were.
        for face in 0..CUBE_FACES * usize::from(self.lights.draw.is_some()) {
            let cube = self.lights.draw.map_or(0, |d| d.0);
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("light shadow"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.lights.cube_faces[cube * CUBE_FACES + face],
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: timers.render_pass(["light shadow", "light shadow 2", "light shadow 3", "light shadow 4", "light shadow 5", "light shadow 6"][face]),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            {
                pass.set_bind_group(0, &self.cascade_bind_group, &[((CASCADES * SHADOWED_VIEWS + face) as u64 * self.cascade_stride) as u32]);
                self.draw_scene(&mut pass, scene, 0, 1, &self.pipelines.shadow, &self.pipelines.tree_shadow);
                self.draw_letter_shadows(&mut pass, letter_count);
            }
        }

        self.targets(gpu, target_size);
        if let (Some(t), None) = (terrain, &self.water_group) {
            let tg = self.targets.as_ref().unwrap();
            self.water_group = Some(t.water_group(&gpu.device, &self.water_layout, &tg.behind, &tg.depth_msaa, &self.water_sampler));
        }
        let cull_views: Vec<crate::cull::CullView> = views
            .iter()
            .map(|v| {
                let (x, y, w, h) = v.viewport.pixels(scene_size);
                crate::cull::CullView {
                    view_proj: v.camera.projection(w / h) * v.camera.view_matrix(),
                    eye: v.camera.eye,
                    forward: v.camera.rotation * glam::Vec3::NEG_Z,
                    viewport: (x, y, w, h),
                }
            })
            .collect();
        self.culling.prepare(gpu, &headers, &cull_views, instance_count, &self.targets.as_ref().unwrap().pyramid);
        let targets = self.targets.as_ref().unwrap();
        let viewports: Vec<(f32, f32, f32, f32)> = views.iter().map(|v| v.viewport.pixels(scene_size)).collect();

        // Culling: each view's list of what it can see, against last frame's pyramid.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("culling"), timestamp_writes: timers.compute_pass("culling") });
            self.culling.cull(&mut pass, &targets.pyramid, views.len(), instance_count);
        }

        // The fog, now that the shadow maps it reads are drawn.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("fog"), timestamp_writes: timers.compute_pass("fog") });
            for v in 0..views.len().min(SHADOWED_VIEWS) {
                self.fog.dispatch(&mut pass, v, (v as u64 * self.view_stride) as u32, frame.relief.fog_grid());
            }
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("depth"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &targets.depth_msaa,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(0.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: timers.render_pass("depth"),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(1, &surface.bind_group, &[]);
            for (i, &(x, y, w, h)) in viewports.iter().enumerate() {
                pass.set_viewport(x, y, w, h, 0.0, 1.0);
                pass.set_bind_group(0, &self.frame_bind_groups[i.min(SHADOWED_VIEWS - 1)], &[(i as u64 * self.view_stride) as u32]);
                self.draw_scene_culled(&mut pass, scene, i, 2, &self.pipelines.depth, &self.pipelines.tree_depth);
                if terrain.is_some() {
                    pass.set_pipeline(&self.pipelines.terrain_depth);
                    self.draw_terrain(&mut pass, terrain, 2);
                    pass.set_pipeline(&self.pipelines.grass_depth);
                    self.draw_grass(&mut pass, terrain, i, grass_group, &grass_counts);
                    pass.set_pipeline(&self.pipelines.depth);
                }
            }
        }
        // The depth pyramid, from the depth just drawn, for the next frame's culling.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor { label: Some("depth pyramid"), timestamp_writes: timers.compute_pass("depth pyramid") });
            self.culling.build_pyramid(&mut pass, &targets.pyramid);
        }
        // Contact occlusion, from the depth just drawn.
        {
            let mut pass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: Some("contact occlusion"),
                timestamp_writes: timers.compute_pass("contact occlusion"),
            });
            for (i, &(_, _, w, h)) in viewports.iter().enumerate().filter(|_| frame.relief.contact() > 0.0) {
                self.contact.dispatch(&mut pass, &targets.contact_group, (i as u64 * self.view_stride) as u32, (w as u32, h as u32));
            }
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("opaque"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &targets.colour_msaa,
                    depth_slice: None,
                    resolve_target: Some(&targets.resolved),
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Discard },
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &targets.depth_msaa,
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: timers.render_pass("opaque"),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_bind_group(1, &surface.bind_group, &[]);
            for (i, &(x, y, w, h)) in viewports.iter().enumerate() {
                pass.set_viewport(x, y, w, h, 0.0, 1.0);
                pass.set_bind_group(0, &self.frame_bind_groups[i.min(SHADOWED_VIEWS - 1)], &[(i as u64 * self.view_stride) as u32]);
                self.draw_scene_culled(&mut pass, scene, i, 2, &self.pipelines.opaque, &self.pipelines.tree);
                if terrain.is_some() {
                    pass.set_pipeline(&self.pipelines.terrain);
                    self.draw_terrain(&mut pass, terrain, 2);
                    pass.set_pipeline(&self.pipelines.grass);
                    self.draw_grass(&mut pass, terrain, i, grass_group, &grass_counts);
                }
                if letter_count > 0 {
                    pass.set_pipeline(&self.pipelines.letters);
                    pass.set_bind_group(2, &self.object_bind_group, &[0]);
                    pass.set_vertex_buffer(0, self.letter_buffer.slice(..));
                    pass.draw(0..6, 0..letter_count as u32);
                }
                pass.set_pipeline(&self.pipelines.sky);
                pass.set_bind_group(2, &self.object_bind_group, &[0]);
                pass.draw(0..3, 0..1);
            }
        }
        // The water, over the scene it lies on, which it reads from a copy.
        if let (Some(t), Some(group)) = (terrain.filter(|t| t.water_tile_count > 0), &self.water_group) {
            encoder.copy_texture_to_texture(
                targets.resolved_texture.as_image_copy(),
                targets.behind_texture.as_image_copy(),
                wgpu::Extent3d { width: targets.size.0, height: targets.size.1, depth_or_array_layers: 1 },
            );
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("water"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &targets.resolved,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: timers.render_pass("water"),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipelines.water);
            pass.set_bind_group(1, &surface.bind_group, &[]);
            pass.set_bind_group(2, &self.object_bind_group, &[0]);
            pass.set_bind_group(3, group, &[]);
            pass.set_index_buffer(t.indices.slice(..), wgpu::IndexFormat::Uint32);
            for (i, &(x, y, w, h)) in viewports.iter().enumerate() {
                pass.set_viewport(x, y, w, h, 0.0, 1.0);
                pass.set_bind_group(0, &self.frame_bind_groups[i.min(SHADOWED_VIEWS - 1)], &[(i as u64 * self.view_stride) as u32]);
                pass.draw_indexed(0..t.index_count, 0, 0..t.water_tile_count);
            }
        }
        // The rain, over everything, around each view.
        if let (Some(t), Some(group)) = (terrain.filter(|_| frame.weather.rain > 0.0), &self.water_group) {
            let _ = t;
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("rain"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &targets.resolved,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: timers.render_pass("rain"),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipelines.rain);
            pass.set_bind_group(1, &surface.bind_group, &[]);
            pass.set_bind_group(2, &self.object_bind_group, &[0]);
            pass.set_bind_group(3, group, &[]);
            for (i, &(x, y, w, h)) in viewports.iter().enumerate() {
                pass.set_viewport(x, y, w, h, 0.0, 1.0);
                pass.set_bind_group(0, &self.frame_bind_groups[i.min(SHADOWED_VIEWS - 1)], &[(i as u64 * self.view_stride) as u32]);
                pass.draw(0..6, 0..RAIN_DROPS);
            }
        }
        // Bloom: down the chain, then back up it, each level added to the one above.
        let fullscreen = |encoder: &mut wgpu::CommandEncoder, name, target: &wgpu::TextureView, pipeline: &wgpu::RenderPipeline, source: &wgpu::BindGroup, load, timers: &mut PassTimers| {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("bloom"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: timers.render_pass(name),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(pipeline);
            pass.set_bind_group(0, source, &[]);
            pass.draw(0..3, 0..1);
        };
        let clear = wgpu::LoadOp::Clear(wgpu::Color::BLACK);
        const DOWN: [&str; BLOOM_LEVELS] = ["bloom", "bloom down 2", "bloom down 3", "bloom down 4", "bloom down 5"];
        const UP: [&str; BLOOM_LEVELS] = ["bloom up 1", "bloom up 2", "bloom up 3", "bloom up 4", "bloom up 5"];
        fullscreen(encoder, DOWN[0], &targets.bloom_levels[0], &self.pipelines.bloom_first, &targets.scene_source, clear, timers);
        for level in 1..BLOOM_LEVELS {
            fullscreen(encoder, DOWN[level], &targets.bloom_levels[level], &self.pipelines.bloom_down, &targets.bloom_sources[level - 1], clear, timers);
        }
        for level in (0..BLOOM_LEVELS - 1).rev() {
            fullscreen(encoder, UP[level], &targets.bloom_levels[level], &self.pipelines.bloom_up, &targets.bloom_sources[level + 1], wgpu::LoadOp::Load, timers);
        }
        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("post"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: timers.render_pass("post"),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            pass.set_pipeline(&self.pipelines.post);
            pass.set_bind_group(0, &targets.post_bind_group, &[]);
            pass.draw(0..3, 0..1);
        }
        // The pyramid now holds this frame's depth, for the next frame to cull against.
        self.culling.pyramid_ready = true;
    }
}

