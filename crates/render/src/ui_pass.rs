use crate::atlas::AtlasGpu;
use crate::ShaderLibrary;
use bytemuck::{Pod, Zeroable};
use vellmark_gpu::{wgpu, Gpu, PipelineRegistry};
use vellmark_ui::Canvas;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct Instance {
    rect: [f32; 4],
    colour: [f32; 4],
    layer: u32,
    _pad: [u32; 3],
}

/// Draws the interface's canvas over the finished frame, at full resolution.
pub struct UiPass {
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    uniform: wgpu::Buffer,
    instances: wgpu::Buffer,
    capacity: usize,
    count: u32,
    range_cell: (f32, f32),
}

impl UiPass {
    pub fn new(
        gpu: &Gpu,
        registry: &mut PipelineRegistry,
        shaders: &ShaderLibrary,
        atlas: &AtlasGpu,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self, String> {
        let module = shaders.module(gpu, "ui")?;
        let layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("ui"),
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                        view_dimension: wgpu::TextureViewDimension::D2Array,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
        });
        let uniform = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui uniform"),
            size: 16,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("ui"),
            layout: &layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: uniform.as_entire_binding() },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::TextureView(&atlas.view) },
                wgpu::BindGroupEntry { binding: 2, resource: wgpu::BindingResource::Sampler(&atlas.sampler) },
            ],
        });
        let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("ui"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        const ATTRIBUTES: [wgpu::VertexAttribute; 3] =
            wgpu::vertex_attr_array![0 => Float32x4, 1 => Float32x4, 2 => Uint32];
        let pipeline = registry.render_pipeline(
            &gpu.device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("ui"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[Some(wgpu::VertexBufferLayout {
                        array_stride: std::mem::size_of::<Instance>() as u64,
                        step_mode: wgpu::VertexStepMode::Instance,
                        attributes: &ATTRIBUTES,
                    })],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
                    entry_point: Some("fs"),
                    compilation_options: Default::default(),
                    targets: &[Some(wgpu::ColorTargetState {
                        format: target_format,
                        blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                        write_mask: wgpu::ColorWrites::ALL,
                    })],
                }),
                primitive: wgpu::PrimitiveState {
                    topology: wgpu::PrimitiveTopology::TriangleStrip,
                    ..Default::default()
                },
                depth_stencil: None,
                multisample: Default::default(),
                multiview_mask: None,
                cache: None,
            },
        );
        let capacity = 4096;
        let instances = Self::instance_buffer(gpu, capacity);
        Ok(Self {
            pipeline,
            bind_group,
            uniform,
            instances,
            capacity,
            count: 0,
            range_cell: (atlas.range, atlas.cell as f32),
        })
    }

    fn instance_buffer(gpu: &Gpu, capacity: usize) -> wgpu::Buffer {
        gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("ui instances"),
            size: (capacity * std::mem::size_of::<Instance>()) as u64,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        })
    }

    pub fn prepare(&mut self, gpu: &Gpu, target_size: (u32, u32), canvas: &Canvas) {
        self.count = canvas.quads.len() as u32;
        if canvas.quads.is_empty() {
            return;
        }
        if canvas.quads.len() > self.capacity {
            self.capacity = canvas.quads.len().next_power_of_two();
            self.instances = Self::instance_buffer(gpu, self.capacity);
        }
        let data: Vec<Instance> = canvas
            .quads
            .iter()
            .map(|q| Instance { rect: q.rect, colour: q.colour, layer: q.layer, _pad: [0; 3] })
            .collect();
        gpu.queue.write_buffer(&self.instances, 0, bytemuck::cast_slice(&data));
        let uniform = [target_size.0 as f32, target_size.1 as f32, self.range_cell.0, self.range_cell.1];
        gpu.queue.write_buffer(&self.uniform, 0, bytemuck::cast_slice(&uniform));
    }

    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>) {
        if self.count == 0 {
            return;
        }
        pass.set_pipeline(&self.pipeline);
        pass.set_bind_group(0, &self.bind_group, &[]);
        pass.set_vertex_buffer(0, self.instances.slice(..));
        pass.draw(0..4, 0..self.count);
    }
}
