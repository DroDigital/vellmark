//! Culling on the GPU: for each view, a list of the instances it can see and an indirect
//! draw for each batch, from `shaders/cull.wgsl`; and the depth pyramid that the next
//! frame culls against, from `shaders/pyramid.wgsl`.

use glam::{Mat4, Vec3};
use vellmark_gpu::{wgpu, Gpu, PipelineRegistry};

use crate::frame::MAX_VIEWS;
use crate::world_gpu::{MAX_GROUPS, MAX_INSTANCES};

/// Matches `Cull` in `shaders/cull.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, bytemuck::Pod, bytemuck::Zeroable)]
struct CullUniform {
    view_proj: [f32; 16],
    previous: [f32; 16],
    viewport: [f32; 4],
    pyramid: [f32; 4],
    counts: [u32; 4],
    /// The eye, for choosing levels of detail.
    eye: [f32; 4],
}

/// The size of one indirect indexed draw, in bytes.
const DRAW: u64 = 20;

/// What a view was last frame, to cull against the pyramid it drew.
#[derive(Clone, Copy)]
struct Seen {
    view_proj: Mat4,
    eye: Vec3,
    forward: Vec3,
}

/// One view of a frame, as culling needs it.
pub struct CullView {
    pub view_proj: Mat4,
    pub eye: Vec3,
    pub forward: Vec3,
    /// Top left and size on the target, in pixels.
    pub viewport: (f32, f32, f32, f32),
}

/// The pyramid and the bind groups that read the depth target: made with the targets.
pub struct PyramidTargets {
    size: (u32, u32),
    levels: u32,
    first_group: wgpu::BindGroup,
    down_groups: Vec<wgpu::BindGroup>,
    cull_group: wgpu::BindGroup,
}

pub struct Culling {
    pub draws: wgpu::Buffer,
    uniforms: wgpu::Buffer,
    stride: u64,
    cull_layout: wgpu::BindGroupLayout,
    first_layout: wgpu::BindGroupLayout,
    down_layout: wgpu::BindGroupLayout,
    empty_group: wgpu::BindGroup,
    cull: wgpu::ComputePipeline,
    first: wgpu::ComputePipeline,
    down: wgpu::ComputePipeline,
    previous: Vec<Option<Seen>>,
    /// Whether the pyramid holds last frame's depth for these targets.
    pub pyramid_ready: bool,
}

fn entry(binding: u32, ty: wgpu::BindingType) -> wgpu::BindGroupLayoutEntry {
    wgpu::BindGroupLayoutEntry { binding, visibility: wgpu::ShaderStages::COMPUTE, ty, count: None }
}

fn storage_buffer(read_only: bool) -> wgpu::BindingType {
    wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Storage { read_only }, has_dynamic_offset: false, min_binding_size: None }
}

fn level_texture(storage: bool) -> wgpu::BindingType {
    if storage {
        wgpu::BindingType::StorageTexture {
            access: wgpu::StorageTextureAccess::WriteOnly,
            format: wgpu::TextureFormat::R32Float,
            view_dimension: wgpu::TextureViewDimension::D2,
        }
    } else {
        wgpu::BindingType::Texture {
            sample_type: wgpu::TextureSampleType::Float { filterable: false },
            view_dimension: wgpu::TextureViewDimension::D2,
            multisampled: false,
        }
    }
}

impl Culling {
    pub fn new(gpu: &Gpu, registry: &mut PipelineRegistry, cull_module: &wgpu::ShaderModule, pyramid_module: &wgpu::ShaderModule) -> Self {
        let device = &gpu.device;
        let align = device.limits().min_uniform_buffer_offset_alignment as u64;
        let stride = (std::mem::size_of::<CullUniform>() as u64).div_ceil(align) * align;
        let draws = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("indirect draws"),
            size: MAX_VIEWS as u64 * MAX_GROUPS as u64 * DRAW,
            usage: wgpu::BufferUsages::INDIRECT | wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST | wgpu::BufferUsages::COPY_SRC,
            mapped_at_creation: false,
        });
        let uniforms = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("culling"),
            size: stride * MAX_VIEWS as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let cull_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("culling"),
            entries: &[
                entry(0, storage_buffer(true)),
                entry(1, storage_buffer(false)),
                entry(2, storage_buffer(false)),
                entry(
                    3,
                    wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(std::mem::size_of::<CullUniform>() as u64),
                    },
                ),
                entry(
                    4,
                    wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                ),
            ],
        });
        let first_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("depth pyramid, first level"),
            entries: &[
                entry(
                    0,
                    wgpu::BindingType::Texture { sample_type: wgpu::TextureSampleType::Depth, view_dimension: wgpu::TextureViewDimension::D2, multisampled: true },
                ),
                entry(1, level_texture(true)),
            ],
        });
        let down_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("depth pyramid, next level"),
            entries: &[entry(0, level_texture(false)), entry(1, level_texture(true))],
        });
        let empty_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("empty"), entries: &[] });
        let empty_group = device.create_bind_group(&wgpu::BindGroupDescriptor { label: Some("empty"), layout: &empty_layout, entries: &[] });
        let pipeline = |label, module, entry_point, groups: &[Option<&wgpu::BindGroupLayout>], registry: &mut PipelineRegistry| {
            let layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor { label: Some(label), bind_group_layouts: groups, immediate_size: 0 });
            registry.compute_pipeline(
                device,
                &wgpu::ComputePipelineDescriptor {
                    label: Some(label),
                    layout: Some(&layout),
                    module,
                    entry_point: Some(entry_point),
                    compilation_options: Default::default(),
                    cache: None,
                },
            )
        };
        let cull = pipeline("culling", cull_module, "main", &[Some(&cull_layout)], registry);
        let first = pipeline("depth pyramid, first level", pyramid_module, "first", &[Some(&first_layout)], registry);
        let down = pipeline("depth pyramid, next level", pyramid_module, "down", &[Some(&empty_layout), Some(&down_layout)], registry);
        Self {
            draws,
            uniforms,
            stride,
            cull_layout,
            first_layout,
            down_layout,
            empty_group,
            cull,
            first,
            down,
            previous: vec![None; MAX_VIEWS],
            pyramid_ready: false,
        }
    }

    /// The pyramid for targets of `size`, reading `depth`. The pyramid is new, so the next
    /// frame culls by the frustum alone.
    pub fn targets(&mut self, gpu: &Gpu, size: (u32, u32), depth: &wgpu::TextureView, instances: &wgpu::Buffer, visible: &wgpu::Buffer) -> PyramidTargets {
        let device = &gpu.device;
        let half = (size.0.div_ceil(2).max(1), size.1.div_ceil(2).max(1));
        let levels = 32 - half.0.max(half.1).leading_zeros();
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("depth pyramid"),
            size: wgpu::Extent3d { width: half.0, height: half.1, depth_or_array_layers: 1 },
            mip_level_count: levels,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R32Float,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let level = |l: u32| texture.create_view(&wgpu::TextureViewDescriptor { base_mip_level: l, mip_level_count: Some(1), ..Default::default() });
        let first_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("depth pyramid, first level"),
            layout: &self.first_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(depth) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&level(0)) },
            ],
        });
        let down_groups = (1..levels)
            .map(|l| {
                device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("depth pyramid, next level"),
                    layout: &self.down_layout,
                    entries: &[
                        wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&level(l - 1)) },
                        wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&level(l)) },
                    ],
                })
            })
            .collect();
        let whole = texture.create_view(&Default::default());
        let cull_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("culling"),
            layout: &self.cull_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: instances.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: visible.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 2, resource: self.draws.as_entire_binding() },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: &self.uniforms,
                        offset: 0,
                        size: wgpu::BufferSize::new(std::mem::size_of::<CullUniform>() as u64),
                    }),
                },
                wgpu::BindGroupEntry { binding: 4, resource: wgpu::BindingResource::TextureView(&whole) },
            ],
        });
        self.pyramid_ready = false;
        PyramidTargets { size: half, levels, first_group, down_groups, cull_group }
    }

    /// This frame's draws, all empty, and each view's culling. A view whose camera has cut
    /// since the last frame, or that has none, culls by the frustum alone.
    /// `headers` holds each batch's draw: its index count, first index, and base vertex.
    pub fn prepare(&mut self, gpu: &Gpu, headers: &[[u32; 3]], views: &[CullView], instance_count: usize, pyramid: &PyramidTargets) {
        let mut draws: Vec<u32> = Vec::with_capacity(views.len() * MAX_GROUPS * 5);
        for _ in views {
            for g in 0..MAX_GROUPS {
                match headers.get(g) {
                    Some(h) => draws.extend_from_slice(&[h[0], 0, h[1], h[2], 0]),
                    None => draws.extend_from_slice(&[0; 5]),
                }
            }
        }
        if !draws.is_empty() {
            gpu.queue.write_buffer(&self.draws, 0, bytemuck::cast_slice(&draws));
        }
        for (v, view) in views.iter().enumerate().take(MAX_VIEWS) {
            let seen = self.previous[v];
            let steady = seen.is_some_and(|s| s.eye.distance(view.eye) < 1.0 && s.forward.dot(view.forward) > 0.97);
            let occlusion = self.pyramid_ready && steady;
            let previous = seen.map_or(view.view_proj, |s| s.view_proj);
            let uniform = CullUniform {
                view_proj: view.view_proj.to_cols_array(),
                previous: previous.to_cols_array(),
                viewport: [view.viewport.0, view.viewport.1, view.viewport.2, view.viewport.3],
                pyramid: [pyramid.size.0 as f32, pyramid.size.1 as f32, pyramid.levels as f32, if occlusion { 1.0 } else { 0.0 }],
                counts: [instance_count as u32, ((1 + v) * MAX_INSTANCES) as u32, (v * MAX_GROUPS * 5) as u32, 0],
                eye: [view.eye.x, view.eye.y, view.eye.z, 0.0],
            };
            gpu.queue.write_buffer(&self.uniforms, v as u64 * self.stride, bytemuck::bytes_of(&uniform));
            self.previous[v] = Some(Seen { view_proj: view.view_proj, eye: view.eye, forward: view.forward });
        }
    }

    pub fn cull(&self, pass: &mut wgpu::ComputePass<'_>, pyramid: &PyramidTargets, views: usize, instance_count: usize) {
        if instance_count == 0 {
            return;
        }
        pass.set_pipeline(&self.cull);
        for v in 0..views.min(MAX_VIEWS) {
            pass.set_bind_group(0, &pyramid.cull_group, &[(v as u64 * self.stride) as u32]);
            pass.dispatch_workgroups((instance_count as u32).div_ceil(64), 1, 1);
        }
    }

    pub fn build_pyramid(&self, pass: &mut wgpu::ComputePass<'_>, pyramid: &PyramidTargets) {
        pass.set_pipeline(&self.first);
        pass.set_bind_group(0, &pyramid.first_group, &[]);
        pass.dispatch_workgroups(pyramid.size.0.div_ceil(8), pyramid.size.1.div_ceil(8), 1);
        pass.set_pipeline(&self.down);
        pass.set_bind_group(0, &self.empty_group, &[]);
        for (l, group) in pyramid.down_groups.iter().enumerate() {
            let w = (pyramid.size.0 >> (l + 1)).max(1);
            let h = (pyramid.size.1 >> (l + 1)).max(1);
            pass.set_bind_group(1, group, &[]);
            pass.dispatch_workgroups(w.div_ceil(8), h.div_ceil(8), 1);
        }
    }

    /// Where view `v`'s draw of batch `g` is, in `draws`.
    pub fn draw_offset(v: usize, g: usize) -> u64 {
        (v * MAX_GROUPS + g) as u64 * DRAW
    }
}
