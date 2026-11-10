//! The passes that draw the world: sun shadows, the depth pre-pass, lit surfaces with
//! the sky behind them, and the step from the lit scene to the screen.

use crate::frame::{
    cascade_matrix, view_uniform, Atmosphere, ViewUniform, CASCADES, CASCADE_FAR, MAX_VIEWS, SHADOW_SIZE,
};
use crate::world_gpu::{vertex_layout, SceneGpu, SurfaceResources, MAX_OBJECTS};
use crate::{FrameInput, ShaderLibrary};
use glam::Mat4;
use vellmark_gpu::{wgpu, Gpu, PassTimers, PipelineRegistry};

pub const HDR_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;
pub const DEPTH_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Depth32Float;
pub const SAMPLES: u32 = 4;

/// Views whose shadows are drawn. Further views borrow the first view's.
const SHADOWED_VIEWS: usize = 2;

struct Targets {
    size: (u32, u32),
    colour_msaa: wgpu::TextureView,
    depth_msaa: wgpu::TextureView,
    resolved: wgpu::TextureView,
    post_bind_group: wgpu::BindGroup,
}

struct Pipelines {
    shadow: wgpu::RenderPipeline,
    depth: wgpu::RenderPipeline,
    opaque: wgpu::RenderPipeline,
    sky: wgpu::RenderPipeline,
    post: wgpu::RenderPipeline,
}

pub struct ScenePass {
    pipelines: Pipelines,
    frame_layout: wgpu::BindGroupLayout,
    object_layout: wgpu::BindGroupLayout,
    cascade_layout: wgpu::BindGroupLayout,
    post_layout: wgpu::BindGroupLayout,
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
        let object_stride = aligned(64, align);
        let cascade_stride = aligned(64, align);

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
            ],
        });
        let object_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("object"),
            entries: &[uniform_entry(0, wgpu::ShaderStages::VERTEX, true, 64)],
        });
        let cascade_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("cascade"),
            entries: &[uniform_entry(0, wgpu::ShaderStages::VERTEX, true, 64)],
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
        let cascade_buffer = buffer("cascades", cascade_stride * (CASCADES * SHADOWED_VIEWS) as u64);
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
        let object_bind_group = one("object", &object_layout, &object_buffer, 64);
        let cascade_bind_group = one("cascade", &cascade_layout, &cascade_buffer, 64);

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
            gpu, registry, shaders, surface, target_format, &frame_layout, &object_layout, &cascade_layout, &post_layout,
        )?;
        Ok(Self {
            pipelines,
            frame_layout,
            object_layout,
            cascade_layout,
            post_layout,
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
    ) -> Result<Pipelines, String> {
        let device = &gpu.device;
        let opaque_module = shaders.module(gpu, "opaque")?;
        let shadow_module = shaders.module(gpu, "shadow")?;
        let post_module = shaders.module(gpu, "post")?;

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
        Ok(Pipelines { shadow, depth, opaque, sky, post })
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
            let post_bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("post"),
                layout: &self.post_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&resolved) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&self.post_sampler) },
                    wgpu::BindGroupEntry { binding: 2, resource: self.post_buffer.as_entire_binding() },
                ],
            });
            self.targets = Some(Targets { size, colour_msaa, depth_msaa, resolved, post_bind_group });
        }
        self.targets.as_ref().unwrap()
    }

    fn draw_scene(&self, pass: &mut wgpu::RenderPass<'_>, scene: &SceneGpu, object_group: u32) {
        for (i, drawable) in scene.drawables.iter().enumerate() {
            pass.set_bind_group(object_group, &self.object_bind_group, &[(i as u64 * self.object_stride) as u32]);
            pass.set_vertex_buffer(0, drawable.mesh.vertices.slice(..));
            pass.set_index_buffer(drawable.mesh.indices.slice(..), wgpu::IndexFormat::Uint32);
            pass.draw_indexed(0..drawable.mesh.index_count, 0, 0..1);
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

        // Uniforms: every object's transform, every view, every cascade.
        for (i, drawable) in scene.drawables.iter().enumerate() {
            gpu.queue.write_buffer(
                &self.object_buffer,
                i as u64 * self.object_stride,
                bytemuck::cast_slice(&drawable.transform.to_cols_array()),
            );
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
                    gpu.queue.write_buffer(
                        &self.cascade_buffer,
                        (i * CASCADES + c) as u64 * self.cascade_stride,
                        bytemuck::cast_slice(&slot.to_cols_array()),
                    );
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
                &cascades,
            );
            gpu.queue.write_buffer(&self.view_buffer, i as u64 * self.view_stride, bytemuck::bytes_of(&uniform));
        }
        gpu.queue.write_buffer(&self.post_buffer, 0, bytemuck::cast_slice(&[atmosphere.exposure, 0.0, 0.0, 0.0]));

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
            self.draw_scene(&mut pass, scene, 1);
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
                self.draw_scene(&mut pass, scene, 2);
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
                self.draw_scene(&mut pass, scene, 2);
                pass.set_pipeline(&self.pipelines.sky);
                pass.set_bind_group(2, &self.object_bind_group, &[0]);
                pass.draw(0..3, 0..1);
            }
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
