//! The renderer. It draws a list of views from an immutable snapshot. One view is in
//! use in the Showcase; the list is there from the first day so that split-screen is an
//! addition and not a rewrite.

mod atlas;
mod camera;
mod field;
mod shaders;
mod ui_pass;

pub use camera::{sun_direction, Camera, View, Viewport};
pub use shaders::ShaderLibrary;

use atlas::AtlasGpu;
use field::FieldPass;
use ui_pass::UiPass;
use vellmark_content::GlyphAtlas;
use vellmark_ui::Canvas;
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
    /// The interface, drawn over the finished frame.
    pub ui: Option<&'a Canvas>,
}

pub struct Renderer {
    field: FieldPass,
    ui: UiPass,
    atlas: AtlasGpu,
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
        glyphs: &GlyphAtlas,
    ) -> Result<Self, String> {
        let atlas = AtlasGpu::new(gpu, glyphs);
        Ok(Self {
            field: FieldPass::new(gpu, registry, shaders, target_format)?,
            ui: UiPass::new(gpu, registry, shaders, &atlas, target_format)?,
            atlas,
            target_format,
        })
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
        let field = FieldPass::new(gpu, registry, shaders, self.target_format);
        let ui = UiPass::new(gpu, registry, shaders, &self.atlas, self.target_format);
        if was == vellmark_gpu::PipelinePhase::Play {
            registry.seal();
        }
        // Nothing is replaced unless everything compiled.
        let (field, ui) = (field?, ui?);
        self.field = field;
        self.ui = ui;
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
        drop(pass);

        if let Some(canvas) = frame.ui.filter(|c| !c.quads.is_empty()) {
            self.ui.prepare(gpu, target_size, canvas);
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("interface"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: target,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations { load: wgpu::LoadOp::Load, store: wgpu::StoreOp::Store },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: timers.render_pass("interface"),
                occlusion_query_set: None,
                multiview_mask: None,
            });
            self.ui.draw(&mut pass);
        }
    }
}
