//! Contact occlusion at half resolution: `shaders/contact.wgsl` says how. Its result is
//! held in one texture sized for the largest target, half of 4K, and each view writes its
//! own part.

use vellmark_gpu::{Gpu, PipelineRegistry, wgpu};

/// The largest half-resolution target: half of 3840 × 2160.
pub const MAX: (u32, u32) = (1920, 1080);

pub struct ContactOcclusion {
    pub layout: wgpu::BindGroupLayout,
    pub view: wgpu::TextureView,
    output: wgpu::TextureView,
    pipeline: wgpu::ComputePipeline,
}

impl ContactOcclusion {
    pub fn new(
        gpu: &Gpu,
        registry: &mut PipelineRegistry,
        module: &wgpu::ShaderModule,
        view_size: u64,
    ) -> Self {
        let device = &gpu.device;
        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("contact occlusion"),
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
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: true,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Rgba8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
            ],
        });
        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("contact occlusion"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = registry.compute_pipeline(
            device,
            &wgpu::ComputePipelineDescriptor {
                label: Some("contact occlusion"),
                layout: Some(&pipeline_layout),
                module,
                entry_point: Some("main"),
                compilation_options: Default::default(),
                cache: None,
            },
        );
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("contact occlusion"),
            size: wgpu::Extent3d {
                width: MAX.0,
                height: MAX.1,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&Default::default());
        let output = texture.create_view(&Default::default());
        Self {
            layout,
            view,
            output,
            pipeline,
        }
    }

    /// The bind group for a depth target: made again whenever the targets are.
    pub fn bind(
        &self,
        gpu: &Gpu,
        view_buffer: &wgpu::Buffer,
        view_size: u64,
        depth: &wgpu::TextureView,
    ) -> wgpu::BindGroup {
        gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("contact occlusion"),
            layout: &self.layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                        buffer: view_buffer,
                        offset: 0,
                        size: wgpu::BufferSize::new(view_size),
                    }),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(depth),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(&self.output),
                },
            ],
        })
    }

    /// Finds the occlusion of one view, of `size` pixels, whose uniform is at `offset`.
    pub fn dispatch(
        &self,
        pass: &mut wgpu::ComputePass<'_>,
        group: &wgpu::BindGroup,
        offset: u32,
        size: (u32, u32),
    ) {
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, group, &[offset]);
        let half = ((size.0 / 2).min(MAX.0), (size.1 / 2).min(MAX.1));
        pass.dispatch_workgroups(half.0.div_ceil(8), half.1.div_ceil(8), 1);
    }
}
