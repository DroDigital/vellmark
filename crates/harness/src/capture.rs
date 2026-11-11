//! The capture tool. A script names shots; each shot is a camera, an hour, and a number
//! of frames on a fixed clock. The same script always produces the same images.

use crate::image::{contact_sheet, difference_masked, Difference, Image};
use crate::refs::{self, RefResult, Tolerance};
use crate::shimmer::{self, Shimmer};
use glam::Vec3;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use vellmark_gpu::{read_texture_rgba8, wgpu, Gpu, GpuOptions, PassTimers, PipelineRegistry};
use vellmark_content::{AssetId, AssetKind, GlyphAtlas, Pack};
use vellmark_render::{load_scene, Camera, FrameInput, GlyphTuning, LoadedAssets, Renderer, ShaderLibrary, View};

fn one() -> u32 {
    1
}

#[derive(Clone, Copy, Debug, Deserialize)]
pub struct Pose {
    pub eye: (f32, f32, f32),
    /// Degrees. 0 looks north (-Z), 90 looks east (+X).
    pub yaw: f32,
    /// Degrees. Positive looks up.
    pub pitch: f32,
}

impl Pose {
    fn lerp(&self, to: &Pose, t: f32) -> Pose {
        let a = Vec3::from(self.eye);
        let e = a.lerp(Vec3::from(to.eye), t);
        Pose { eye: e.into(), yaw: self.yaw + (to.yaw - self.yaw) * t, pitch: self.pitch + (to.pitch - self.pitch) * t }
    }

    pub fn camera(&self) -> Camera {
        Camera::looking(self.eye.into(), self.yaw.to_radians(), self.pitch.to_radians())
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Shot {
    pub name: String,
    pub from: Pose,
    /// Where the camera ends. With more than one frame, it moves there at a steady rate.
    #[serde(default)]
    pub to: Option<Pose>,
    /// Hours since midnight.
    pub hour: f32,
    #[serde(default = "one")]
    pub frames: u32,
    /// Also capture with the glyph layer off, and report how the two differ.
    #[serde(default)]
    pub pair: bool,
    /// Run the shimmer check over this shot's frames. Needs three or more.
    #[serde(default)]
    pub shimmer: bool,
    /// Close reading, from 0, the realistic frame, to 1, the text frame.
    #[serde(default)]
    pub close_reading: f32,
    /// The depth look: 0 flat, 1 atoms, 2 lattice, 3 source, 4 living.
    #[serde(default)]
    pub look: u32,
    /// Put the lamps out for this shot.
    #[serde(default)]
    pub lamps_out: bool,
    /// The dive through a letter, 0 or 1.
    #[serde(default)]
    pub dive: f32,
    /// The nearest distance drawn, in metres. A dive needs less than the usual 5 cm.
    #[serde(default)]
    pub near: Option<f32>,
    /// Also save this part of the first frame at full size: left, top, width, height.
    /// For judging letters pixel for pixel in a frame too large to look at whole.
    #[serde(default)]
    pub crop: Option<(u32, u32, u32, u32)>,
    /// Break this thing of the scene into its letters before the shot, by its name.
    #[serde(default)]
    pub break_thing: Option<String>,
    /// Where the blow that breaks it lands. The camera, if not given.
    #[serde(default)]
    pub blow: Option<(f32, f32, f32)>,
    /// How hard the blow is: 1 throws the letters across the yard, near 0 the thing
    /// falls in on itself.
    #[serde(default)]
    pub strength: Option<f32>,
    /// Seconds of the clock between one frame and the next. One fixed step, if not
    /// given; a video at 24 frames a second wants 1/24.
    #[serde(default)]
    pub frame_seconds: Option<f32>,
    /// Fixed steps of the rules run after the break and before the first frame.
    #[serde(default)]
    pub settle: u32,
    /// Fixed steps of the rules run between one frame and the next.
    #[serde(default)]
    pub steps_per_frame: u32,
    /// Heat a thing of the scene before the shot, by its name: -1 frozen to 1 as hot as
    /// it gets.
    #[serde(default)]
    pub heat: Option<(String, f32)>,
    /// The eye's heights are above the ground rather than above sea level.
    #[serde(default)]
    pub above_ground: bool,
    /// Keep the first frame of this shot as a reference image.
    #[serde(default)]
    pub reference: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(default)]
pub struct Limits {
    /// The largest mean difference allowed between a glyph-on and glyph-off pair,
    /// beyond the reveal distance.
    pub pair_mean: f32,
    /// The largest difference allowed in any one pixel there.
    pub pair_max: f32,
    /// The largest change in mean brightness the letters may cause where they show.
    pub tone_shift: f32,
    /// The largest 99th-percentile flicker allowed in a shimmer shot.
    pub shimmer_p99: f32,
}

impl Default for Limits {
    fn default() -> Self {
        Self { pair_mean: 0.0005, pair_max: 0.02, tone_shift: 0.03, shimmer_p99: 0.02 }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Script {
    pub name: String,
    /// The scene to capture, by its authored key, such as `scene.yard`.
    pub scene: String,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub limits: Limits,
    pub shots: Vec<Shot>,
}

impl Script {
    pub fn load(path: &Path) -> Result<Self, String> {
        let text = std::fs::read_to_string(path).map_err(|e| format!("{}: {e}", path.display()))?;
        ron::from_str(&text).map_err(|e| format!("{}: {e}", path.display()))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RefMode {
    /// Do not look at reference images.
    Ignore,
    /// Fail on a capture that has moved from its reference.
    Check,
    /// Replace the references with this run's captures. Done only after looking at them.
    Accept,
}

pub struct Options {
    /// Where images are written. `None` keeps them in memory only.
    pub out: Option<PathBuf>,
    pub refs_root: PathBuf,
    pub ref_mode: RefMode,
    pub shaders: ShaderLibrary,
    pub gpu: GpuOptions,
}

/// What the glyph-on and glyph-off frames of one shot show.
#[derive(Clone, Copy, Debug)]
pub struct Pair {
    /// How the two differ beyond the reveal distance. This must be next to nothing.
    pub beyond: Difference,
    /// The share of the frame in which letters are drawn.
    pub revealed_share: f32,
    /// The relative change in mean brightness where letters are drawn.
    pub tone_shift: f32,
}

pub struct ShotResult {
    pub name: String,
    pub frames: Vec<Image>,
    pub glyph_off: Option<Image>,
    pub pair: Option<Pair>,
    pub shimmer: Option<Shimmer>,
    pub reference: Option<RefResult>,
}

pub struct Report {
    pub script: String,
    pub adapter: String,
    pub shots: Vec<ShotResult>,
    /// Every limit that was broken, in words. Empty means the run passed.
    pub failures: Vec<String>,
}

impl Report {
    pub fn passed(&self) -> bool {
        self.failures.is_empty()
    }

    pub fn summary(&self) -> String {
        let mut s = format!("capture `{}` on {}\n", self.script, self.adapter);
        for shot in &self.shots {
            s.push_str(&format!("  {}: {} frame(s)", shot.name, shot.frames.len()));
            if let Some(d) = shot.pair {
                s.push_str(&format!(
                    ", beyond reveal: mean {:.5} max {:.3}; lettered {:.0}% of frame, tone {:+.2}%",
                    d.beyond.mean,
                    d.beyond.max,
                    d.revealed_share * 100.0,
                    d.tone_shift * 100.0
                ));
            }
            if let Some(m) = shot.shimmer {
                s.push_str(&format!(", shimmer mean {:.5} p99 {:.4} flickering {:.4}", m.mean, m.p99, m.flickering));
            }
            match &shot.reference {
                Some(RefResult::Matches(d)) => s.push_str(&format!(", reference matches (mean {:.5})", d.mean)),
                Some(RefResult::Moved(d)) => {
                    s.push_str(&format!(", reference MOVED (mean {:.5}, changed {:.4})", d.mean, d.changed))
                }
                Some(RefResult::Missing) => s.push_str(", reference MISSING"),
                Some(RefResult::SizeChanged) => s.push_str(", reference SIZE CHANGED"),
                None => {}
            }
            s.push('\n');
        }
        for f in &self.failures {
            s.push_str(&format!("  FAIL: {f}\n"));
        }
        s.push_str(if self.passed() { "  passed\n" } else { "  FAILED\n" });
        s
    }
}

/// A renderer that draws into a texture and reads it back.
pub struct Offscreen {
    pub gpu: Gpu,
    pub registry: PipelineRegistry,
    pub renderer: Renderer,
    pub timers: PassTimers,
    texture: wgpu::Texture,
    size: (u32, u32),
    /// The scene being drawn, as the rules need it.
    pub scene: Option<vellmark_content::Scene>,
    /// The ground it stands on, if any.
    pub terrain: Option<vellmark_world::Terrain>,
}

pub const CAPTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

/// Bakes the repository's data and returns the base pack, as the game would load it.
pub fn baked_base_pack() -> Result<Pack, String> {
    let root = crate::repo_root();
    let inputs = vellmark_bake::BakeInputs {
        assets: root.join("assets"),
        third_party: root.join("third_party"),
        cache: Some(root.join("build/bake-cache")),
    };
    let output = vellmark_bake::bake(&inputs).map_err(|e| e.to_string())?;
    let (_, bytes) = output.packs.into_iter().next().ok_or("the bake produced no pack")?;
    Pack::from_bytes(bytes).map_err(|e| e.to_string())
}

pub fn glyph_atlas(pack: &Pack) -> Result<GlyphAtlas, String> {
    let bytes = pack
        .get(AssetKind::GlyphAtlas, AssetId::from_key(vellmark_bake::GLYPH_ATLAS_KEY))
        .ok_or("the pack has no glyph atlas")?;
    GlyphAtlas::from_bytes(bytes).map_err(|e| e.to_string())
}

impl Offscreen {
    /// `scene` is the key of the scene to draw, or `None` for an empty world.
    pub fn new(
        gpu_options: &GpuOptions,
        shaders: &ShaderLibrary,
        width: u32,
        height: u32,
        scene: Option<&str>,
    ) -> Result<Self, String> {
        let gpu = Gpu::headless(gpu_options).map_err(|e| e.to_string())?;
        let mut registry = PipelineRegistry::new();
        let pack = baked_base_pack()?;
        let assets = LoadedAssets::from_pack(&pack)?;
        let mut renderer = Renderer::new(&gpu, &mut registry, shaders, CAPTURE_FORMAT, &assets.borrow())?;
        let mut loaded = None;
        let mut terrain = None;
        if let Some(key) = scene {
            let (data, on_gpu) = load_scene(&gpu, &pack, key)?;
            renderer.set_scene(&gpu, on_gpu);
            if data.terrain.0 != 0 {
                let t = vellmark_world::Terrain::from_pack(&pack)?;
                let tiles: Vec<&vellmark_content::TerrainTile> = t.tiles().collect();
                renderer.set_terrain(&gpu, &tiles, &assets.materials)?;
                terrain = Some(t);
            }
            loaded = Some(data);
        }
        registry.seal();
        let timers = PassTimers::new(&gpu, true);
        let texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("capture"),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: CAPTURE_FORMAT,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        Ok(Self { gpu, registry, renderer, timers, texture, size: (width, height), scene: loaded, terrain })
    }

    pub fn frame(&mut self, frame: &FrameInput<'_>) -> Image {
        let view = self.texture.create_view(&Default::default());
        let mut encoder = self.gpu.device.create_command_encoder(&Default::default());
        self.timers.begin_frame();
        self.renderer.render(&self.gpu, &mut encoder, &view, self.size, frame, &mut self.timers);
        self.timers.end_frame(&self.gpu, &mut encoder);
        self.gpu.queue.submit([encoder.finish()]);
        self.timers.after_submit();
        let rgba = read_texture_rgba8(&self.gpu, &self.texture);
        self.timers.collect();
        Image::new(self.size.0, self.size.1, rgba)
    }
}

pub fn run(script: &Script, options: &Options) -> Result<Report, String> {
    let mut offscreen = Offscreen::new(&options.gpu, &options.shaders, script.width, script.height, Some(&script.scene))?;
    let mut report = Report {
        script: script.name.clone(),
        adapter: offscreen.gpu.describe(),
        shots: Vec::new(),
        failures: Vec::new(),
    };
    let out = options.out.as_ref().map(|o| o.join(&script.name));

    for shot in &script.shots {
        let frames_wanted = shot.frames.max(1);
        let mut frames = Vec::new();
        let mut glyph_off = None;
        let mut reveal_mask: Option<Image> = None;
        // A shot that breaks something runs the rules for it, from a fresh world.
        let mut world = None;
        if let Some(name) = &shot.break_thing {
            let scene = offscreen.scene.as_ref().ok_or("a shot breaks something, but the script names no scene")?;
            let mut w = vellmark_sim::World::new(vellmark_sim::WorldConfig::default());
            let (solids, breakables) = vellmark_render::scene_rules(scene);
            w.set_scene(solids, breakables);
            let id = vellmark_content::AssetId::from_key(&format!("{}.{name}", script.scene));
            let blow = shot.blow.map_or(shot.from.camera().eye, |b| b.into());
            w.break_thing_with(id.0, blow, shot.strength.unwrap_or(1.0)).ok_or_else(|| format!("shot `{}`: the scene has nothing called `{name}` to break", shot.name))?;
            for _ in 0..shot.settle {
                w.step(&[]);
            }
            world = Some(w);
        }
        let heat: Vec<(u64, f32)> = match &shot.heat {
            Some((name, h)) => vec![(vellmark_content::AssetId::from_key(&format!("{}.{name}", script.scene)).0, *h)],
            None => Vec::new(),
        };
        for i in 0..frames_wanted {
            if let (Some(w), true) = (world.as_mut(), i > 0) {
                for _ in 0..shot.steps_per_frame {
                    w.step(&[]);
                }
            }
            let snapshot = world.as_ref().map(|w| w.snapshot());
            let (letters, letters_step, broken): (&[vellmark_sim::LetterSnap], u64, &[u64]) = match &snapshot {
                Some(s) => (&s.letters, s.letters_step, &s.broken),
                None => (&[], 0, &[]),
            };
            let t = if frames_wanted > 1 { i as f32 / (frames_wanted - 1) as f32 } else { 0.0 };
            let pose = match &shot.to {
                Some(to) => shot.from.lerp(to, t),
                None => shot.from,
            };
            let mut pose = pose;
            if shot.above_ground {
                if let Some(h) = offscreen.terrain.as_ref().and_then(|t| t.height_at(pose.eye.0, pose.eye.2)) {
                    pose.eye.1 += h;
                }
            }
            let mut camera = pose.camera();
            if let Some(near) = shot.near {
                camera.near = near;
            }
            let views = [View::full(camera)];
            let input = FrameInput {
                views: &views,
                time_of_day: shot.hour,
                seconds: i as f32 * shot.frame_seconds.unwrap_or(vellmark_sim::STEP_SECONDS),
                glyph_layer: true,
                close_reading: shot.close_reading,
                dive: shot.dive,
                lamps: if shot.lamps_out { 0.0 } else { 1.0 },
                tuning: GlyphTuning { look: shot.look, ..GlyphTuning::default() },
                reveal_mask: false,
                letters,
                letters_step,
                broken,
                heat: &heat,
                ui: None,
            };
            frames.push(offscreen.frame(&input));
            if shot.pair && i == 0 {
                glyph_off = Some(offscreen.frame(&FrameInput { glyph_layer: false, ..input }));
                reveal_mask = Some(offscreen.frame(&FrameInput { reveal_mask: true, ..input }));
            }
        }

        // The two promises, measured. Beyond the reveal distance the frames must match.
        // Within it they differ, as they should, but the letters must be invisible in
        // aggregate: the mean brightness of the lettered part may not move.
        let pair = match (&glyph_off, &reveal_mask) {
            (Some(off), Some(mask)) => {
                let revealed: Vec<bool> = mask.rgba.chunks_exact(4).map(|p| p[0] > 4).collect();
                // The edge filter reads a pixel's neighbours, so a pixel beside lettered
                // ones is not held to the comparison. Three pixels is its reach.
                let (w, h) = (mask.width as i32, mask.height as i32);
                let beyond: Vec<bool> = (0..revealed.len())
                    .map(|i| {
                        let (x, y) = (i as i32 % w, i as i32 / w);
                        for dy in -3..=3 {
                            for dx in -3..=3 {
                                let (xx, yy) = (x + dx, y + dy);
                                if xx >= 0 && yy >= 0 && xx < w && yy < h && revealed[(yy * w + xx) as usize] {
                                    return false;
                                }
                            }
                        }
                        true
                    })
                    .collect();
                let far = difference_masked(&frames[0], off, Some(&beyond))?;
                if far.mean > script.limits.pair_mean || far.max > script.limits.pair_max {
                    report.failures.push(format!(
                        "{}: beyond the reveal distance, glyph-on and glyph-off differ by mean {:.5} (limit {:.5}), max {:.3} (limit {:.3})",
                        shot.name, far.mean, script.limits.pair_mean, far.max, script.limits.pair_max
                    ));
                }
                let share = revealed.iter().filter(|r| **r).count() as f32 / revealed.len() as f32;
                let (on_l, off_l) = (frames[0].luminance(), off.luminance());
                let mean = |l: &[f32]| {
                    let (mut sum, mut n) = (0.0f64, 0u32);
                    for (v, r) in l.iter().zip(&revealed) {
                        if *r {
                            sum += *v as f64;
                            n += 1;
                        }
                    }
                    if n == 0 { 0.0 } else { (sum / n as f64) as f32 }
                };
                let (lit_on, lit_off) = (mean(&on_l), mean(&off_l));
                let tone_shift = if lit_off > 0.0 { (lit_on - lit_off) / lit_off } else { 0.0 };
                if tone_shift.abs() > script.limits.tone_shift {
                    report.failures.push(format!(
                        "{}: within the reveal distance the letters change mean brightness by {:+.2}% (limit {:.1}%)",
                        shot.name,
                        tone_shift * 100.0,
                        script.limits.tone_shift * 100.0
                    ));
                }
                Some(Pair { beyond: far, revealed_share: share, tone_shift })
            }
            _ => None,
        };

        let shimmer = if shot.shimmer {
            let m = shimmer::measure(&frames).map_err(|e| format!("{}: {e}", shot.name))?;
            if m.p99 > script.limits.shimmer_p99 {
                report.failures.push(format!(
                    "{}: shimmer p99 {:.4}, limit {:.4}",
                    shot.name, m.p99, script.limits.shimmer_p99
                ));
            }
            Some(m)
        } else {
            None
        };

        let reference = if shot.reference && options.ref_mode != RefMode::Ignore {
            let path = refs::reference_path(&options.refs_root, &script.name, &shot.name);
            if options.ref_mode == RefMode::Accept {
                refs::accept(&path, &frames[0])?;
            }
            let result = refs::check(&path, &frames[0], Tolerance::default());
            if !result.passed() {
                report.failures.push(format!(
                    "{}: differs from its reference image {} ({result:?}); look at the capture, and accept it if it is right",
                    shot.name,
                    path.display()
                ));
            }
            Some(result)
        } else {
            None
        };

        if let Some(dir) = &out {
            if frames.len() == 1 {
                frames[0].save(&dir.join(format!("{}.png", shot.name)))?;
            } else {
                for (i, frame) in frames.iter().enumerate() {
                    frame.save(&dir.join(format!("{}_{i:04}.png", shot.name)))?;
                }
            }
            if let Some(off) = &glyph_off {
                off.save(&dir.join(format!("{}_glyph_off.png", shot.name)))?;
            }
            if let Some((x, y, w, h)) = shot.crop {
                frames[0].crop(x, y, w, h).save(&dir.join(format!("{}_crop.png", shot.name)))?;
            }
        }

        report.shots.push(ShotResult { name: shot.name.clone(), frames, glyph_off, pair, shimmer, reference });
    }

    if let Some(dir) = &out {
        let firsts: Vec<Image> = report.shots.iter().map(|s| s.frames[0].clone()).collect();
        if !firsts.is_empty() {
            let columns = (firsts.len() as f32).sqrt().ceil() as u32;
            contact_sheet(&firsts, columns).save(&dir.join("contact_sheet.png"))?;
        }
        std::fs::write(dir.join("report.txt"), report.summary()).map_err(|e| e.to_string())?;
    }
    if offscreen.registry.created_during_play() > 0 {
        report.failures.push(format!(
            "pipelines were created during the capture: {:?}",
            offscreen.registry.names_created_during_play()
        ));
    }
    Ok(report)
}
