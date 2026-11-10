use crate::camera::sun_direction;
use crate::{FrameInput, ShaderLibrary};
use bytemuck::{Pod, Zeroable};
use vellmark_gpu::{wgpu, Gpu, PipelineRegistry};

/// Matches `ViewUniform` in `shaders/frame.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ViewUniform {
    pub view_proj: [[f32; 4]; 4],
    pub inv_view_proj: [[f32; 4]; 4],
    pub eye_time: [f32; 4],
    pub sun_dir_hour: [f32; 4],
    pub switches: [f32; 4],
}

/// The most views drawn in one frame. Two is all the full game will ever ask for; four
/// leaves room for tools.
pub const MAX_VIEWS: usize = 4;

pub struct FieldPass {
    pipeline: wgpu::RenderPipeline,
    uniforms: wgpu::Buffer,
    bind_group: wgpu::BindGroup,
    stride: u64,
}

impl FieldPass {
    pub fn new(
        gpu: &Gpu,
        registry: &mut PipelineRegistry,
        shaders: &ShaderLibrary,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self, String> {
        let module = shaders.module(gpu, "field")?;
        let size = std::mem::size_of::<ViewUniform>() as u64;
        let align = gpu.device.limits().min_uniform_buffer_offset_alignment as u64;
        let stride = size.div_ceil(align) * align;

        let layout = gpu.device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("view"),
            entries: &[wgpu::BindGroupLayoutEntry {
                binding: 0,
                visibility: wgpu::ShaderStages::VERTEX_FRAGMENT,
                ty: wgpu::BindingType::Buffer {
                    ty: wgpu::BufferBindingType::Uniform,
                    has_dynamic_offset: true,
                    min_binding_size: wgpu::BufferSize::new(size),
                },
                count: None,
            }],
        });
        let uniforms = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("view uniforms"),
            size: stride * MAX_VIEWS as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        let bind_group = gpu.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("view"),
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: wgpu::BindingResource::Buffer(wgpu::BufferBinding {
                    buffer: &uniforms,
                    offset: 0,
                    size: wgpu::BufferSize::new(size),
                }),
            }],
        });
        let pipeline_layout = gpu.device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("field"),
            bind_group_layouts: &[Some(&layout)],
            immediate_size: 0,
        });
        let pipeline = registry.render_pipeline(
            &gpu.device,
            &wgpu::RenderPipelineDescriptor {
                label: Some("field"),
                layout: Some(&pipeline_layout),
                vertex: wgpu::VertexState {
                    module: &module,
                    entry_point: Some("vs"),
                    compilation_options: Default::default(),
                    buffers: &[],
                },
                fragment: Some(wgpu::FragmentState {
                    module: &module,
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
        Ok(Self { pipeline, uniforms, bind_group, stride })
    }

    pub fn prepare(&self, gpu: &Gpu, target_size: (u32, u32), frame: &FrameInput<'_>) {
        let sun = sun_direction(frame.time_of_day);
        for (i, view) in frame.views.iter().take(MAX_VIEWS).enumerate() {
            let (_, _, w, h) = view.viewport.pixels(target_size);
            let view_proj = view.camera.view_projection(w / h);
            let uniform = ViewUniform {
                view_proj: view_proj.to_cols_array_2d(),
                inv_view_proj: view_proj.inverse().to_cols_array_2d(),
                eye_time: view.camera.eye.extend(frame.seconds).to_array(),
                sun_dir_hour: sun.extend(frame.time_of_day).to_array(),
                switches: [if frame.glyph_layer { 1.0 } else { 0.0 }, 0.0, 0.0, 0.0],
            };
            gpu.queue.write_buffer(&self.uniforms, i as u64 * self.stride, bytemuck::bytes_of(&uniform));
        }
    }

    pub fn draw(&self, pass: &mut wgpu::RenderPass<'_>, target_size: (u32, u32), frame: &FrameInput<'_>) {
        pass.set_pipeline(&self.pipeline);
        for (i, view) in frame.views.iter().take(MAX_VIEWS).enumerate() {
            let (x, y, w, h) = view.viewport.pixels(target_size);
            pass.set_viewport(x, y, w, h, 0.0, 1.0);
            pass.set_bind_group(0, &self.bind_group, &[(i as u64 * self.stride) as u32]);
            pass.draw(0..3, 0..1);
        }
    }
}
