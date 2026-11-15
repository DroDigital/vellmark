//! The fog volume: for each view, a grid of cells in front of the eye that the mist's
//! light is gathered into and added up along each line of sight. `shaders/fog.wgsl`
//! says how. Surfaces and the sky read the summed grid.

use vellmark_gpu::{wgpu, Gpu, PipelineRegistry};

pub const SIZE: (u32, u32, u32) = (160, 90, 64);
const FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba16Float;

pub struct FogVolume {
    pub inject_groups: Vec<wgpu::BindGroup>,
    pub integrate_groups: Vec<wgpu::BindGroup>,
    /// The summed grid of each view, for the frame's bind group.
    pub integrated_views: Vec<wgpu::TextureView>,
    pub sampler: wgpu::Sampler,
    inject: wgpu::ComputePipeline,
    integrate: wgpu::ComputePipeline,
    empty: wgpu::BindGroup,
}

impl FogVolume {
    /// `views` is how many views have their own fog; `view_buffer` holds every view's
    /// uniform at `view_stride`; `shadow_arrays` is each such view's cascades.
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        gpu: &Gpu,
        registry: &mut PipelineRegistry,
        module: &wgpu::ShaderModule,
        views: usize,
        view_buffer: &wgpu::Buffer,
        view_size: u64,
        shadow_arrays: &[wgpu::TextureView],
        shadow_sampler: &wgpu::Sampler,
    ) -> Self {
        let device = &gpu.device;
        let storage = |binding| wgpu::BindGroupLayoutEntry {
            binding,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::StorageTexture {
                access: wgpu::StorageTextureAccess::WriteOnly,
                format: FORMAT,
                view_dimension: wgpu::TextureViewDimension::D3,
            },
            count: None,
        };
        let inject_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fog inject"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: true,
                        min_binding_size: wgpu::BufferSize::new(view_size),
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Depth,
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Comparison),
                    count: None,
                },
                storage(3),
            ],
        });
        let integrate_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("fog integrate"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: false },
                        view_dimension: wgpu::TextureViewDimension::D3,
                        multisampled: false,
                    },
                    count: None,
                },
                storage(1),
            ],
        });
        // Each pass has its own layout: the gathered grid is written by the first and read
        // by the second, and may not be bound both ways at once. The second needs only the
        // view at index 0, then its grids.
        let view_entry = wgpu::BindGroupLayoutEntry {
            binding: 0,
            visibility: wgpu::ShaderStages::COMPUTE,
            ty: wgpu::BindingType::Buffer { ty: wgpu::BufferBindingType::Uniform, has_dynamic_offset: true, min_binding_size: wgpu::BufferSize::new(view_size) },
            count: None,
        };
        let empty_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor { label: Some("fog view"), entries: &[view_entry] });
        let inject_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fog inject"),
            bind_group_layouts: &[Some(&inject_layout)],
            immediate_size: 0,
        });
        let integrate_pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("fog integrate"),
            bind_group_layouts: &[Some(&empty_layout), Some(&integrate_layout)],
            immediate_size: 0,
        });
        let mut pipeline = |entry: &'static str, layout: &wgpu::PipelineLayout| {
            registry.compute_pipeline(
                device,
                &wgpu::ComputePipelineDescriptor {
                    label: Some(entry),
                    layout: Some(layout),
                    module,
                    entry_point: Some(entry),
                    compilation_options: Default::default(),
                    cache: None,
                },
            )
        };
        let inject = pipeline("inject", &inject_pipeline_layout);
        let integrate = pipeline("integrate", &integrate_pipeline_layout);
        let empty = device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("fog view"),
            layout: &empty_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: view_buffer, offset: 0, size: wgpu::BufferSize::new(view_size) }),
            }],
        });

        let volume = |label| {
            device.create_texture(&wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d { width: SIZE.0, height: SIZE.1, depth_or_array_layers: SIZE.2 },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D3,
                format: FORMAT,
                usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
                view_formats: &[],
            })
        };
        let mut inject_groups = Vec::new();
        let mut integrate_groups = Vec::new();
        let mut integrated_views = Vec::new();
        for v in 0..views {
            let injected = volume("fog injected").create_view(&Default::default());
            let integrated = volume("fog integrated").create_view(&Default::default());
            inject_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fog inject"),
                layout: &inject_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding { buffer: view_buffer, offset: 0, size: wgpu::BufferSize::new(view_size) }),
                    },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&shadow_arrays[v]) },
                    wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(shadow_sampler) },
                    wgpu::BindGroupEntry { binding: 3, resource: wgpu::BindingResource::TextureView(&injected) },
                ],
            }));
            integrate_groups.push(device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("fog integrate"),
                layout: &integrate_layout,
                entries: &[
                    wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&injected) },
                    wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&integrated) },
                ],
            }));
            integrated_views.push(integrated);
        }
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("fog"),
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            ..Default::default()
        });
        Self { inject_groups, integrate_groups, integrated_views, sampler, inject, integrate, empty }
    }

    /// Gathers and sums the fog of view `v`, whose uniform is at `offset`, over the share
    /// `grid` of the full grid along each side that the governor allows.
    pub fn dispatch(&self, pass: &mut wgpu::ComputePass<'_>, v: usize, offset: u32, grid: f32) {
        // A little more than the shader's own rounding: it skips what is beyond its grid.
        let used = |n: u32| ((n as f32 * grid).ceil() as u32).clamp(1, n);
        let (x, y, z) = (used(SIZE.0), used(SIZE.1), used(SIZE.2));
        pass.set_pipeline(&self.inject);
        pass.set_bind_group(0, &self.inject_groups[v], &[offset]);
        pass.dispatch_workgroups(x.div_ceil(8), y.div_ceil(8), z);
        pass.set_pipeline(&self.integrate);
        pass.set_bind_group(0, &self.empty, &[offset]);
        pass.set_bind_group(1, &self.integrate_groups[v], &[]);
        pass.dispatch_workgroups(x.div_ceil(8), y.div_ceil(8), 1);
    }
}
