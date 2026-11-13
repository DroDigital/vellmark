//! The renderer. It draws a list of views from an immutable snapshot. One view is in
//! use in the Showcase; the list is there from the first day so that split-screen is an
//! addition and not a rewrite.

mod atlas;
pub mod bark;
mod camera;
mod contact;
mod cull;
mod fog;
mod frame;
mod lights;
mod scene_pass;
mod shaders;
pub mod sky;
mod governor;
mod quality;
mod grass;
mod terrain;
mod ui_pass;
mod world_gpu;

pub use camera::{sun_direction, Camera, View, Viewport};
pub use frame::{Atmosphere, GlyphTuning};
pub use governor::{Governor, Knob, Relief};
pub use quality::{Preset, Quality};
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
    /// The rain, how wet the world is, and the wind.
    pub weather: vellmark_sim::Weather,
    /// For tools: a gust writing a wind sentence, by number, and how far it has come, 0
    /// to 1; at 0.4 the sentence is whole.
    pub wind_gust: Option<(u32, f32)>,
    /// What the governor has given up this frame.
    pub relief: Relief,
    /// The interface, drawn over the finished frame.
    pub ui: Option<&'a Canvas>,
}

/// The baked assets the renderer is built from.
pub struct RenderAssets<'a> {
    pub glyphs: &'a GlyphAtlas,
    pub materials: &'a MaterialSet,
    /// The place's colour grade through the day. Neutral if the pack has none.
    pub grade: &'a vellmark_content::Grade,
    /// The sentences the wind writes in the grass, from the string table.
    pub wind: &'a [String],
    /// The words a tree's bark records itself in, from the string table.
    pub bark: &'a bark::BarkWords,
}

/// The renderer's assets as read from a pack.
pub struct LoadedAssets {
    pub glyphs: GlyphAtlas,
    pub materials: MaterialSet,
    pub grade: vellmark_content::Grade,
    pub wind: Vec<String>,
    pub bark: bark::BarkWords,
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
            wind: wind_sentences(pack),
            bark: bark_words(pack),
        })
    }

    pub fn borrow(&self) -> RenderAssets<'_> {
        RenderAssets { glyphs: &self.glyphs, materials: &self.materials, grade: &self.grade, wind: &self.wind, bark: &self.bark }
    }
}

/// The Vale's wind sentences, `wind.vale.0` on, from the base language's string table.
/// The Showcase is in one language; the full game reads the player's.
fn wind_sentences(pack: &vellmark_content::Pack) -> Vec<String> {
    use vellmark_content::{AssetId, AssetKind, StringId, StringTable};
    let Some(table) = pack.get(AssetKind::StringTable, AssetId::from_key("strings.en")).and_then(|b| StringTable::from_bytes(b).ok()) else {
        return Vec::new();
    };
    (0..crate::grass::MAX_SENTENCES)
        .map_while(|i| table.get(StringId::from_key(&format!("wind.vale.{i}"))).map(str::to_owned))
        .collect()
}

/// The words a tree's bark is written in, from the base language's string table: the
/// record, `tree.bark.record`, and each kind's name, `tree.kind.<kind>`.
fn bark_words(pack: &vellmark_content::Pack) -> bark::BarkWords {
    use vellmark_content::{AssetId, AssetKind, StringId, StringTable};
    let Some(table) = pack.get(AssetKind::StringTable, AssetId::from_key("strings.en")).and_then(|b| StringTable::from_bytes(b).ok()) else {
        return bark::BarkWords::default();
    };
    let kinds = ["oak", "ash", "apple", "hawthorn"]
        .iter()
        .filter_map(|k| table.get(StringId::from_key(&format!("tree.kind.{k}"))).map(|n| (k.to_string(), n.to_owned())))
        .collect();
    bark::BarkWords { record: table.get(StringId::from_key("tree.bark.record")).unwrap_or_default().to_owned(), kinds }
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
    let levels = match pack.get(AssetKind::Table, AssetId::from_key(vellmark_content::MESH_LEVELS_KEY)) {
        Some(bytes) => vellmark_content::MeshLevels::from_bytes(bytes)?,
        None => vellmark_content::MeshLevels::default(),
    };
    let mut on_gpu = SceneGpu::new(gpu, &scene, pages, &levels, &|id| pack.get(AssetKind::Mesh, id).and_then(|b| Mesh::from_bytes(b).ok()))?;
    if let Some(bytes) = pack.get(AssetKind::Table, AssetId::from_key(vellmark_content::TREE_RECORDS_KEY)) {
        on_gpu.records = vellmark_content::TreeRecords::from_bytes(bytes)?;
    }
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

/// The trunks of a scene's trees, which bodies cannot walk through: where each stands,
/// from its object, and how thick, from its record.
pub fn tree_trunks(pack: &vellmark_content::Pack, scene: &vellmark_content::Scene) -> Vec<vellmark_sim::Trunk> {
    use vellmark_content::{AssetId, AssetKind, TreeRecords, TREE_RECORDS_KEY};
    let Some(records) = pack.get(AssetKind::Table, AssetId::from_key(TREE_RECORDS_KEY)).and_then(|b| TreeRecords::from_bytes(b).ok()) else {
        return Vec::new();
    };
    let girth: std::collections::HashMap<u64, f32> = records.trees.iter().map(|r| (r.tree.0, r.girth)).collect();
    scene
        .objects
        .iter()
        .filter_map(|o| {
            let girth = *girth.get(&o.key.0)?;
            let t = &o.transform;
            let scale = (t[0] * t[0] + t[8] * t[8]).sqrt();
            Some(vellmark_sim::Trunk {
                foot: glam::Vec3::new(t[3], t[7], t[11]),
                radius: (girth / std::f32::consts::TAU * scale).max(0.08),
                height: 3.2 * scale,
            })
        })
        .collect()
}

/// The words, as the rules know them, from the word table.
pub fn word_rules(pack: &vellmark_content::Pack) -> vellmark_sim::emendation::WordRules {
    use vellmark_content::{AssetId, AssetKind, Outcome, WordTable, WORDS_KEY};
    use vellmark_sim::emendation::{CompoundRule, RuleOutcome, WordRule, WordRules};
    let Some(t) = pack.get(AssetKind::Table, AssetId::from_key(WORDS_KEY)).and_then(|b| WordTable::from_bytes(b).ok()) else {
        return WordRules::default();
    };
    let rule = |o: &Outcome| match o {
        Outcome::Effect { effect, strength, preview } => RuleOutcome::Effect { effect: effect.clone(), strength: *strength, preview: preview.0 },
        Outcome::Refusal(line) => RuleOutcome::Refusal(line.0),
    };
    WordRules {
        classes: t.classes.clone(),
        words: t
            .words
            .iter()
            .map(|w| WordRule {
                spelling: w.word.clone(),
                class: w.class.clone(),
                ink: w.ink,
                flight_speed: w.look.flight_speed,
                outcomes: w.outcomes.iter().map(rule).collect(),
                weapon: rule(&w.weapon),
                typo: w.typo.word.clone(),
                typo_outcomes: w.typo.outcomes.iter().map(rule).collect(),
            })
            .collect(),
        compounds: t
            .compounds
            .iter()
            .filter_map(|c| {
                let a = t.words.iter().position(|w| w.word == c.words[0])?;
                let b = t.words.iter().position(|w| w.word == c.words[1])?;
                Some(CompoundRule { name: c.name.clone(), words: [a, b], outcomes: c.outcomes.iter().map(rule).collect() })
            })
            .collect(),
    }
}

/// A scene's things that carry Inscriptions, for the rules.
pub fn scene_targets(pack: &vellmark_content::Pack, scene_key: &str, rules: &vellmark_sim::emendation::WordRules) -> Vec<vellmark_sim::emendation::Target> {
    use vellmark_content::{targets_key, AssetId, AssetKind, InscribedThings};
    use vellmark_sim::emendation::{Resistance, Target};
    let Some(things) = pack.get(AssetKind::Table, AssetId::from_key(&targets_key(scene_key))).and_then(|b| InscribedThings::from_bytes(b).ok()) else {
        return Vec::new();
    };
    things
        .items
        .iter()
        .filter_map(|t| Some(Target::new(t.id.0, rules.class(&t.class)?, t.centre.into(), t.radius, Resistance::from_index(t.resistance), &t.words)))
        .collect()
}

/// A scene's texts that can be read, for the rules.
pub fn scene_readables(pack: &vellmark_content::Pack, scene_key: &str) -> Vec<vellmark_sim::reading::ReadableText> {
    use vellmark_content::{readables_key, AssetId, AssetKind, Readables};
    let Some(r) = pack.get(AssetKind::Table, AssetId::from_key(&readables_key(scene_key))).and_then(|b| Readables::from_bytes(b).ok()) else {
        return Vec::new();
    };
    r.items
        .iter()
        .map(|t| vellmark_sim::reading::ReadableText {
            id: t.id.0,
            text: t.text.0,
            min: t.min.into(),
            max: t.max.into(),
            hidden: t.hidden.iter().map(|(s, p)| (s.0, *p)).collect(),
        })
        .collect()
}

pub struct Renderer {
    world: ScenePass,
    surface: SurfaceResources,
    ui: UiPass,
    atlas: AtlasGpu,
    scene: SceneGpu,
    terrain: Option<terrain::TerrainGpu>,
    target_format: wgpu::TextureFormat,
    bark_words: bark::BarkWords,
    bark: bark::Bark,
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
            world: ScenePass::new(gpu, registry, shaders, &surface, target_format, assets.grade.clone(), assets.wind.to_vec())?,
            ui: UiPass::new(gpu, registry, shaders, &atlas, target_format)?,
            surface,
            atlas,
            scene: SceneGpu::default(),
            terrain: None,
            target_format,
            bark_words: assets.bark.clone(),
            bark: bark::Bark::default(),
        })
    }

    pub fn target_format(&self) -> wgpu::TextureFormat {
        self.target_format
    }

    /// Replaces the scene being drawn. Buffers, not pipelines, so it is safe in play.
    pub fn set_scene(&mut self, gpu: &Gpu, scene: SceneGpu) {
        self.surface.set_pages(gpu, &scene.pages);
        // Setting the pages cleared the generated ones too: the bark starts afresh.
        self.bark = bark::Bark::new(&scene, &scene.records, &self.bark_words);
        self.scene = scene;
    }

    /// Chooses a preset. Safe in play: nothing is rebuilt that would stall.
    pub fn set_quality(&mut self, quality: Quality) {
        self.world.quality = quality;
        if let Some(t) = self.terrain.as_mut() {
            t.grass.set_tuning(grass::GrassTuning { density: quality.grass_density, near: quality.grass_near });
        }
        self.world.forget_grass_patches();
    }

    /// Replaces the terrain being drawn, or removes it. Buffers and textures only, so it
    /// is safe in play.
    pub fn set_terrain(&mut self, gpu: &Gpu, tiles: &[&vellmark_content::TerrainTile], materials: &MaterialSet) -> Result<(), String> {
        self.world.forget_terrain();
        self.terrain = if tiles.is_empty() { None } else { Some(terrain::TerrainGpu::new(gpu, self.world.terrain_layout(), tiles, materials)?) };
        let q = self.world.quality;
        if let Some(t) = self.terrain.as_mut() {
            t.grass.set_tuning(grass::GrassTuning { density: q.grass_density, near: q.grass_near });
        }
        Ok(())
    }

    /// For tests: how many instances of each batch the first view drew in the last frame,
    /// by the permanent ID of the batch's first placed thing.
    #[cfg(feature = "dev")]
    pub fn drawn_last_frame(&self, gpu: &Gpu) -> Vec<(vellmark_content::AssetId, u32)> {
        self.world.drawn_last_frame(gpu, &self.scene)
    }

    /// For tests: how many of the sun's shadow maps and the lights' shadow cubes were
    /// drawn last frame. The rest were kept from earlier frames.
    pub fn shadows_drawn_last_frame(&self) -> (usize, usize) {
        self.world.shadows_drawn
    }

    /// For tests: how many grass patches view 0 drew last frame at each level of detail.
    #[cfg(feature = "dev")]
    pub fn grass_drawn_last_frame(&self, gpu: &Gpu) -> [u32; 4] {
        self.world.grass_drawn_last_frame(gpu, &self.scene)
    }

    /// For tests: how many trees view 0 drew last frame at each level of detail. A tree
    /// in a crossfade is counted at both its levels.
    #[cfg(feature = "dev")]
    pub fn tree_levels_drawn_last_frame(&self, gpu: &Gpu) -> [u32; 4] {
        self.world.tree_levels_drawn_last_frame(gpu, &self.scene)
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
        // The bark of the trees nearest the eyes reads their records.
        let eyes: Vec<glam::Vec3> = frame.views.iter().map(|v| v.camera.eye).collect();
        for written in self.bark.update(&eyes) {
            self.surface.set_generated_page(gpu, &written);
        }
        self.scene.set_generated(self.bark.pages());
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
