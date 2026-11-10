//! The renderer. It draws a list of views from an immutable snapshot. One view is in
//! use in the Showcase; the list is there from the first day so that split-screen is an
//! addition and not a rewrite.

mod camera;
mod field;
mod shaders;

pub use camera::{sun_direction, Camera, View, Viewport};
pub use shaders::ShaderLibrary;

use field::FieldPass;
use vellmark_gpu::{wgpu, Gpu, PassTimers, PipelineRegistry};

/// What a frame is drawn from, beyond the views themselves.
#[derive(Clone, Copy, Debug)]
pub struct FrameInput<'a> {
    pub views: &'a [View],
    /// Hours since midnight.
    pub time_of_day: f32,
    /// Seconds on the fixed clock. Animation in shaders reads only this.
    pub seconds: f32,
    /// The glyph layer. Off for the comparison captures.
    pub glyph_layer: bool,
}

pub struct Renderer {
    field: FieldPass,
    target_format: wgpu::TextureFormat,
}

impl Renderer {
    /// Makes every pipeline. Call during startup or loading, before the registry is
    /// sealed.
    pub fn new(
        gpu: &Gpu,
        registry: &mut PipelineRegistry,
        shaders: &ShaderLibrary,
        target_format: wgpu::TextureFormat,
    ) -> Result<Self, String> {
        Ok(Self { field: FieldPass::new(gpu, registry, shaders, target_format)?, target_format })
    }

    pub fn target_format(&self) -> wgpu::TextureFormat {
        self.target_format
    }

    /// Remakes the pipelines after a shader changed on disk. Development builds only;
    /// on failure the old pipelines stay in use and the error is returned.
    pub fn reload_shaders(
        &mut self,
        gpu: &Gpu,
        registry: &mut PipelineRegistry,
        shaders: &ShaderLibrary,
    ) -> Result<(), String> {
        let was = registry.phase();
        registry.unseal();
        let result = FieldPass::new(gpu, registry, shaders, self.target_format);
        if was == vellmark_gpu::PipelinePhase::Play {
            registry.seal();
        }
        self.field = result?;
        Ok(())
    }

    /// Draws every view into `target`.
    pub fn render(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        target_size: (u32, u32),
        frame: &FrameInput<'_>,
        timers: &mut PassTimers,
    ) {
        self.field.prepare(gpu, target_size, frame);
        let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("field"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                depth_slice: None,
                resolve_target: None,
                ops: wgpu::Operations { load: wgpu::LoadOp::Clear(wgpu::Color::BLACK), store: wgpu::StoreOp::Store },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: timers.render_pass("field"),
            occlusion_query_set: None,
            multiview_mask: None,
        });
        self.field.draw(&mut pass, target_size, frame);
    }
}
