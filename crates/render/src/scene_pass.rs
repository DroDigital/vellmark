//! The passes that draw the world: sun shadows, the depth pre-pass, lit surfaces with
//! the sky behind them, and the step from the lit scene to the screen.

use crate::frame::{
    cascade_matrix, view_uniform, Atmosphere, ViewUniform, CASCADES, CASCADE_FAR, MAX_VIEWS, SHADOW_SIZE,
};
use crate::lights::{cube_face_matrix, Lights};
use crate::world_gpu::{vertex_layout, SceneGpu, SurfaceResources, MAX_OBJECTS};
use crate::{FrameInput, ShaderLibrary};
use glam::Mat4;
use vellmark_gpu::{wgpu, Gpu, PassTimers, PipelineRegistry};

pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const SAMPLES: u32 = 4;

/// Views whose shadows are drawn. Further views borrow the first view's.
const SHADOWED_VIEWS: usize = 2;

const BLOOM_LEVELS: usize = 5;
const CUBE_FACES: usize = 6;
/// The size of one cascade or cube-face uniform: a matrix and a row of flags.
const CASCADE_UNIFORM: u64 = 80;
/// The size of one object's uniform: its transform, and its state (x: heat, -1 to 1).
const OBJECT_UNIFORM: u64 = 80;

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
    post_bind_group: wgpu::BindGroup,
    /// Each level of the bloom chain, largest first, as a target and as a source.
    bloom_levels: Vec<wgpu::TextureView>,
    bloom_sources: Vec<wgpu::BindGroup>,
    scene_source: wgpu::BindGroup,
}

struct Pipelines {
    shadow: wgpu::RenderPipeline,
    depth: wgpu::RenderPipeline,
    opaque: wgpu::RenderPipeline,
    sky: wgpu::RenderPipeline,
    letters: wgpu::RenderPipeline,
    letter_shadow: wgpu::RenderPipeline,
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
    lights: Lights,
    view_buffer: wgpu::Buffer,
    view_stride: u64,
    object_buffer: wgpu::Buffer,
    object_stride: u64,
    object_bind_group: wgpu::BindGroup,
    cascade_buffer: wgpu::Buffer,
    cascade_stride: u64,
    cascade_bind_group: wgpu::BindGroup,
    post_buffer: wgpu::Buffer,
    post_sampler: wgpu::Sampler,
    shadow_sampler: wgpu::Sampler,
    shadow_layers: Vec<wgpu::TextureView>,
    frame_bind_groups: Vec<wgpu::BindGroup>,
    targets: Option<Targets>,
    target_format: wgpu::TextureFormat,
    /// One instance for each loose letter, and the step they were last sent at.
    letter_buffer: wgpu::Buffer,
    letters_sent: Option<(u64, usize)>,
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
    ) -> Result<Self, String> {
        let device = &gpu.device;
        let align = device.limits().min_uniform_buffer_offset_alignment as u64;
        let view_size = std::mem::size_of::<ViewUniform>() as u64;
        let view_stride = aligned(view_size, align);
        let object_stride = aligned(OBJECT_UNIFORM, align);
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
                        view_dimension: wgpu::TextureViewDimension::Cube,
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
        let object_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("object"),
            entries: &[uniform_entry(0, wgpu::ShaderStages::VERTEX_FRAGMENT, true, OBJECT_UNIFORM)],
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
                uniform_entry(2, wgpu::ShaderStages::FRAGMENT, false, 16),
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
        let object_buffer = buffer("objects", object_stride * MAX_OBJECTS as u64);
        let cascade_buffer = buffer("cascades", cascade_stride * (CASCADES * SHADOWED_VIEWS + CUBE_FACES) as u64);
        let post_buffer = buffer("post", 16);

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
        let object_bind_group = one("object", &object_layout, &object_buffer, OBJECT_UNIFORM);
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
        let frame_bind_groups = (0..SHADOWED_VIEWS as u32)
            .map(|v| {
                let array = shadow_texture.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2Array),
                    base_array_layer: v * CASCADES as u32,
                    array_layer_count: Some(CASCADES as u32),
                    ..Default::default()
                });
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
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&array) },
                        wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&shadow_sampler) },
                        wgpu::BindGroupEntry { binding: 3, resource: lights.uniform.as_entire_binding() },
                        wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&lights.cluster_view) },
                        wgpu::BindGroupEntry { binding: 5, resource: wgpu::BindingResource::TextureView(&lights.cube_view) },
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

        let pipelines = Self::pipelines(
            gpu, registry, shaders, surface, target_format, &frame_layout, &object_layout, &cascade_layout, &post_layout, &bloom_layout,
        )?;
        Ok(Self {
            pipelines,
            frame_layout,
            object_layout,
            cascade_layout,
            post_layout,
            bloom_layout,
            lights,
            view_buffer,
            view_stride,
            object_buffer,
            object_stride,
            object_bind_group,
            cascade_buffer,
            cascade_stride,
            cascade_bind_group,
            post_buffer,
            post_sampler,
            shadow_sampler,
            shadow_layers,
            frame_bind_groups,
            targets: None,
            target_format,
            letter_buffer: gpu.device.create_buffer(&wgpu::BufferDescriptor {
                label: Some("loose letters"),
                size: (vellmark_sim::debris::MAX_LETTERS * std::mem::size_of::<LetterInstance>()) as u64,
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
                mapped_at_creation: false,
            }),
            letters_sent: None,
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
        Ok(Pipelines { shadow, depth, opaque, sky, letters, letter_shadow, post, bloom_first, bloom_down, bloom_up })
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
            let depth_msaa = texture("scene depth", DEPTH_FORMAT, SAMPLES, attachment);
            let resolved = texture("scene resolved", HDR_FORMAT, 1, attachment | wgpu::TextureUsages::TEXTURE_BINDING);
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
            self.targets = Some(Targets { size, colour_msaa, depth_msaa, resolved, post_bind_group, bloom_levels, bloom_sources, scene_source });
        }
        self.targets.as_ref().unwrap()
    }

    fn write_cascade(&self, gpu: &Gpu, slot: usize, matrix: &Mat4, glowing: u32) {
        let mut bytes = [0u8; CASCADE_UNIFORM as usize];
        bytes[..64].copy_from_slice(bytemuck::cast_slice(&matrix.to_cols_array()));
        bytes[64..68].copy_from_slice(&glowing.to_le_bytes());
        gpu.queue.write_buffer(&self.cascade_buffer, slot as u64 * self.cascade_stride, &bytes);
    }

    fn draw_scene(&self, pass: &mut wgpu::RenderPass<'_>, scene: &SceneGpu, broken: &[u64], object_group: u32) {
        for (i, drawable) in scene.drawables.iter().enumerate() {
            // A broken thing is its letters now.
            if broken.contains(&drawable.key.0) {
                continue;
            }
            pass.set_bind_group(object_group, &self.object_bind_group, &[(i as u64 * self.object_stride) as u32]);
            pass.set_vertex_buffer(0, drawable.mesh.vertices.slice(..));
            pass.set_index_buffer(drawable.mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..drawable.mesh.index_count, 0, 0..1);
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
        timers: &mut PassTimers,
    ) {
        let _ = &self.shadow_sampler;
        let atmosphere = Atmosphere::at(frame.time_of_day);
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

        // Uniforms: every object's transform, every view, every cascade.
        for (i, drawable) in scene.drawables.iter().enumerate() {
            let heat = frame.heat.iter().find(|(id, _)| *id == drawable.key.0).map_or(0.0, |(_, h)| *h);
            let mut uniform = [0.0f32; 20];
            uniform[..16].copy_from_slice(&drawable.transform.to_cols_array());
            uniform[16] = heat;
            gpu.queue.write_buffer(&self.object_buffer, i as u64 * self.object_stride, bytemuck::cast_slice(&uniform));
        }
        for (i, view) in views.iter().enumerate() {
            let (_, _, w, h) = view.viewport.pixels(target_size);
            let source = if i < SHADOWED_VIEWS { view } else { &views[0] };
            let (_, _, sw, sh) = source.viewport.pixels(target_size);
            let mut cascades = [Mat4::IDENTITY; CASCADES];
            let mut near = source.camera.near;
            for (c, slot) in cascades.iter_mut().enumerate() {
                *slot = cascade_matrix(source, sw / sh, near, CASCADE_FAR[c], atmosphere.light_dir);
                near = CASCADE_FAR[c] * 0.9;
                if i < SHADOWED_VIEWS {
                    self.write_cascade(gpu, i * CASCADES + c, slot, surface.glowing);
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
                    let (x, y, _, _) = view.viewport.pixels(target_size);
                    (x, y)
                },
                frame.dive,
                frame.lamps,
                &cascades,
            );
            gpu.queue.write_buffer(&self.view_buffer, i as u64 * self.view_stride, bytemuck::bytes_of(&uniform));
        }
        // In the reveal mask nothing may bleed from one pixel to the next.
        let (bloom, edge) = if frame.reveal_mask { (0.0, 0.0) } else { (0.045, 1.0) };
        gpu.queue.write_buffer(&self.post_buffer, 0, bytemuck::cast_slice(&[atmosphere.exposure, bloom, edge, 0.0]));

        let aspects: Vec<f32> = views
            .iter()
            .map(|v| {
                let (_, _, w, h) = v.viewport.pixels(target_size);
                w / h
            })
            .collect();
        self.lights.update(gpu, &scene.lights, views, &aspects);
        if let Some((position, radius)) = self.lights.shadowed {
            for face in 0..CUBE_FACES {
                let matrix = cube_face_matrix(position, radius, face);
                self.write_cascade(gpu, CASCADES * SHADOWED_VIEWS + face, &matrix, surface.glowing);
            }
        }

        // Sun shadows, one pass for each cascade of each shadowed view. Only the first
        // is timed; the timer's name covers them all in the report.
        for layer in 0..views.len().min(SHADOWED_VIEWS) * CASCADES {
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
            pass.set_pipeline(&self.pipelines.shadow);
            pass.set_bind_group(0, &self.cascade_bind_group, &[(layer as u64 * self.cascade_stride) as u32]);
            self.draw_scene(&mut pass, scene, frame.broken, 1);
            self.draw_letter_shadows(&mut pass, letter_count);
        }

        // The one shadowed lantern or fire: six faces of a cube around it.
        for face in 0..CUBE_FACES {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("light shadow"),
                color_attachments: &[],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.lights.cube_faces[face],
                    depth_ops: Some(wgpu::Operations { load: wgpu::LoadOp::Clear(1.0), store: wgpu::StoreOp::Store }),
                    stencil_ops: None,
                }),
                timestamp_writes: timers.render_pass(["light shadow", "light shadow 2", "light shadow 3", "light shadow 4", "light shadow 5", "light shadow 6"][face]),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            if self.lights.shadowed.is_some() {
                pass.set_pipeline(&self.pipelines.shadow);
                pass.set_bind_group(0, &self.cascade_bind_group, &[((CASCADES * SHADOWED_VIEWS + face) as u64 * self.cascade_stride) as u32]);
                self.draw_scene(&mut pass, scene, frame.broken, 1);
                self.draw_letter_shadows(&mut pass, letter_count);
            }
        }

        self.targets(gpu, target_size);
        let targets = self.targets.as_ref().unwrap();
        let viewports: Vec<(f32, f32, f32, f32)> = views.iter().map(|v| v.viewport.pixels(target_size)).collect();

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
            pass.set_pipeline(&self.pipelines.depth);
            pass.set_bind_group(1, &surface.bind_group, &[]);
            for (i, &(x, y, w, h)) in viewports.iter().enumerate() {
                pass.set_viewport(x, y, w, h, 0.0, 1.0);
                pass.set_bind_group(0, &self.frame_bind_groups[i.min(SHADOWED_VIEWS - 1)], &[(i as u64 * self.view_stride) as u32]);
                self.draw_scene(&mut pass, scene, frame.broken, 2);
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
                pass.set_pipeline(&self.pipelines.opaque);
                self.draw_scene(&mut pass, scene, frame.broken, 2);
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
    }
}
