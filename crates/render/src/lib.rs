//! The renderer. It draws a list of views from an immutable snapshot. One view is in
//! use in the Showcase; the list is there from the first day so that split-screen is an
//! addition and not a rewrite.

mod atlas;
mod camera;
mod frame;
mod lights;
mod scene_pass;
mod shaders;
pub mod sky;
mod terrain;
mod ui_pass;
mod world_gpu;

pub use camera::{sun_direction, Camera, View, Viewport};
pub use frame::{Atmosphere, GlyphTuning};
pub use shaders::ShaderLibrary;
pub use world_gpu::SceneGpu;

use atlas::AtlasGpu;
use scene_pass::ScenePass;
use ui_pass::UiPass;
use vellmark_content::{GlyphAtlas, MaterialSet};
use vellmark_gpu::{wgpu, Gpu, PassTimers, PipelineRegistry};
use vellmark_ui::Canvas;
use world_gpu::SurfaceResources;

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
    /// Close reading: 0 is the realistic frame, 1 is the text frame.
    pub close_reading: f32,
    /// The dive through a letter, 0 to 1: levels finer than the finest print are drawn,
    /// so that flying into a character finds more characters inside it.
    pub dive: f32,
    /// How brightly lamps and glowing things burn, 0 to 1. The simulation will drive
    /// this once lanterns are lit by hand and by the hour.
    pub lamps: f32,
    pub tuning: GlyphTuning,
    /// A tool's view: white where the fine print is drawn, black elsewhere.
    pub reveal_mask: bool,
    /// Every loose letter of every broken thing.
    pub letters: &'a [vellmark_sim::LetterSnap],
    /// Changes whenever `letters` does, so that they are sent to the GPU only then.
    pub letters_step: u64,
    /// The permanent IDs of broken things, which are not drawn.
    pub broken: &'a [u64],
    /// How hot or cold things are, by permanent ID, from -1 (frozen) to 1 (as hot as
    /// it gets). Up close their letters say so.
    pub heat: &'a [(u64, f32)],
    /// The interface, drawn over the finished frame.
    pub ui: Option<&'a Canvas>,
}

/// The baked assets the renderer is built from.
pub struct RenderAssets<'a> {
    pub glyphs: &'a GlyphAtlas,
    pub materials: &'a MaterialSet,
    /// The place's colour grade through the day. Neutral if the pack has none.
    pub grade: &'a vellmark_content::Grade,
}

/// The renderer's assets as read from a pack.
pub struct LoadedAssets {
    pub glyphs: GlyphAtlas,
    pub materials: MaterialSet,
    pub grade: vellmark_content::Grade,
}

impl LoadedAssets {
    pub fn from_pack(pack: &vellmark_content::Pack) -> Result<Self, String> {
        use vellmark_content::{AssetId, AssetKind};
        let glyphs = pack
            .get(AssetKind::GlyphAtlas, AssetId::from_key("glyphs.atlas"))
            .ok_or("the pack has no glyph atlas")?;
        let materials = pack
            .get(AssetKind::MaterialSet, AssetId::from_key("materials.set"))
            .ok_or("the pack has no material set")?;
        Ok(Self {
            glyphs: GlyphAtlas::from_bytes(glyphs)?,
            materials: MaterialSet::from_bytes(materials)?,
            // The Vale's grade: the Showcase has one place. The full game blends a grade
            // for each Quire as the player travels.
            grade: match pack.get(AssetKind::Table, AssetId::from_key("grade.vale")) {
                Some(bytes) => vellmark_content::Grade::from_bytes(bytes)?,
                None => vellmark_content::Grade::default(),
            },
        })
    }

    pub fn borrow(&self) -> RenderAssets<'_> {
        RenderAssets { glyphs: &self.glyphs, materials: &self.materials, grade: &self.grade }
    }
}

/// Reads a scene and its meshes from a pack. `key` is the scene's authored key, such as
/// `scene.yard`. Returns the scene's data beside what was put on the GPU.
pub fn load_scene(gpu: &Gpu, pack: &vellmark_content::Pack, key: &str) -> Result<(vellmark_content::Scene, SceneGpu), String> {
    use vellmark_content::{AssetId, AssetKind, Mesh, Scene, TextPages};
    let bytes = pack.get(AssetKind::Scene, AssetId::from_key(key)).ok_or_else(|| format!("the pack has no `{key}`"))?;
    let scene = Scene::from_bytes(bytes)?;
    let pages = match pack.get(AssetKind::TextPage, scene.pages) {
        Some(bytes) => TextPages::from_bytes(bytes)?,
        None => TextPages::default(),
    };
    let on_gpu = SceneGpu::new(gpu, &scene, pages, &|id| pack.get(AssetKind::Mesh, id).and_then(|b| Mesh::from_bytes(b).ok()))?;
    Ok((scene, on_gpu))
}

/// What the rules need of a scene: the boxes loose letters land on and bounce from,
/// and the things in it that can be broken.
pub fn scene_rules(scene: &vellmark_content::Scene) -> (Vec<vellmark_sim::debris::Solid>, Vec<vellmark_sim::BreakableThing>) {
    use glam::{Affine3A, Mat3A, Vec3, Vec3A};
    let solids = scene
        .colliders
        .iter()
        .map(|b| vellmark_sim::debris::Solid { min: b.min.into(), max: b.max.into() })
        .collect();
    let breakables = scene
        .breakables
        .iter()
        .map(|b| {
            let t = &b.transform;
            let to_world = Affine3A::from_mat3_translation(
                Mat3A::from_cols(Vec3A::new(t[0], t[4], t[8]), Vec3A::new(t[1], t[5], t[9]), Vec3A::new(t[2], t[6], t[10])).into(),
                Vec3::new(t[3], t[7], t[11]),
            );
            vellmark_sim::BreakableThing {
                id: b.key.0,
                to_world,
                boxes: b.boxes.iter().map(|s| (Vec3::from(s.min), Vec3::from(s.max), s.material)).collect(),
            }
        })
        .collect();
    (solids, breakables)
}

pub struct Renderer {
    world: ScenePass,
    surface: SurfaceResources,
    ui: UiPass,
    atlas: AtlasGpu,
    scene: SceneGpu,
    terrain: Option<terrain::TerrainGpu>,
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
        assets: &RenderAssets<'_>,
    ) -> Result<Self, String> {
        let atlas = AtlasGpu::new(gpu, assets.glyphs);
        let surface = SurfaceResources::new(gpu, assets.materials, assets.glyphs, &atlas)?;
        Ok(Self {
            world: ScenePass::new(gpu, registry, shaders, &surface, target_format, assets.grade.clone())?,
            ui: UiPass::new(gpu, registry, shaders, &atlas, target_format)?,
            surface,
            atlas,
            scene: SceneGpu::default(),
            terrain: None,
            target_format,
        })
    }

    pub fn target_format(&self) -> wgpu::TextureFormat {
        self.target_format
    }

    /// Replaces the scene being drawn. Buffers, not pipelines, so it is safe in play.
    pub fn set_scene(&mut self, gpu: &Gpu, scene: SceneGpu) {
        self.surface.set_pages(gpu, &scene.pages);
        self.scene = scene;
    }

    /// Replaces the terrain being drawn, or removes it. Buffers and textures only, so it
    /// is safe in play.
    pub fn set_terrain(&mut self, gpu: &Gpu, tiles: &[&vellmark_content::TerrainTile], materials: &MaterialSet) -> Result<(), String> {
        self.terrain = if tiles.is_empty() { None } else { Some(terrain::TerrainGpu::new(gpu, self.world.terrain_layout(), tiles, materials)?) };
        Ok(())
    }

    pub fn scene_mut(&mut self) -> &mut SceneGpu {
        &mut self.scene
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
        let world = self.world.reload(gpu, registry, shaders, &self.surface);
        let ui = UiPass::new(gpu, registry, shaders, &self.atlas, self.target_format);
        if was == vellmark_gpu::PipelinePhase::Play {
            registry.seal();
        }
        world?;
        self.ui = ui?;
        Ok(())
    }

    /// Draws every view into `target`, then the interface over them.
    pub fn render(
        &mut self,
        gpu: &Gpu,
        encoder: &mut wgpu::CommandEncoder,
        target: &wgpu::TextureView,
        target_size: (u32, u32),
        frame: &FrameInput<'_>,
        timers: &mut PassTimers,
    ) {
        self.world.render(gpu, encoder, target, target_size, frame, &self.scene, &self.surface, self.terrain.as_ref(), timers);

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
