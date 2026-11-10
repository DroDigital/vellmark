//! The capture tool. A script names shots; each shot is a camera, an hour, and a number
//! of frames on a fixed clock. The same script always produces the same images.

use crate::image::{contact_sheet, difference, Difference, Image};
use crate::refs::{self, RefResult, Tolerance};
use crate::shimmer::{self, Shimmer};
use glam::Vec3;
use serde::Deserialize;
use std::path::{Path, PathBuf};
use vellmark_gpu::{read_texture_rgba8, wgpu, Gpu, GpuOptions, PassTimers, PipelineRegistry};
use vellmark_render::{Camera, FrameInput, Renderer, ShaderLibrary, View};

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
    /// Keep the first frame of this shot as a reference image.
    #[serde(default)]
    pub reference: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct Limits {
    /// The largest mean difference allowed between a glyph-on and glyph-off pair.
    pub pair_mean: f32,
    /// The largest 99th-percentile flicker allowed in a shimmer shot.
    pub shimmer_p99: f32,
}

impl Default for Limits {
    fn default() -> Self {
        Self { pair_mean: 0.004, shimmer_p99: 0.02 }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct Script {
    pub name: String,
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

pub struct ShotResult {
    pub name: String,
    pub frames: Vec<Image>,
    pub glyph_off: Option<Image>,
    pub pair: Option<Difference>,
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
                s.push_str(&format!(", glyph pair mean {:.5} max {:.3}", d.mean, d.max));
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
}

pub const CAPTURE_FORMAT: wgpu::TextureFormat = wgpu::TextureFormat::Rgba8UnormSrgb;

impl Offscreen {
    pub fn new(gpu_options: &GpuOptions, shaders: &ShaderLibrary, width: u32, height: u32) -> Result<Self, String> {
        let gpu = Gpu::headless(gpu_options).map_err(|e| e.to_string())?;
        let mut registry = PipelineRegistry::new();
        let renderer = Renderer::new(&gpu, &mut registry, shaders, CAPTURE_FORMAT)?;
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
        Ok(Self { gpu, registry, renderer, timers, texture, size: (width, height) })
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
    let mut offscreen = Offscreen::new(&options.gpu, &options.shaders, script.width, script.height)?;
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
        for i in 0..frames_wanted {
            let t = if frames_wanted > 1 { i as f32 / (frames_wanted - 1) as f32 } else { 0.0 };
            let pose = match &shot.to {
                Some(to) => shot.from.lerp(to, t),
                None => shot.from,
            };
            let views = [View::full(pose.camera())];
            let input = FrameInput {
                views: &views,
                time_of_day: shot.hour,
                seconds: i as f32 * vellmark_sim::STEP_SECONDS,
                glyph_layer: true,
            };
            frames.push(offscreen.frame(&input));
            if shot.pair && i == 0 {
                glyph_off = Some(offscreen.frame(&FrameInput { glyph_layer: false, ..input }));
            }
        }

        let pair = match &glyph_off {
            Some(off) => {
                let d = difference(&frames[0], off)?;
                if d.mean > script.limits.pair_mean {
                    report.failures.push(format!(
                        "{}: glyph-on and glyph-off differ by mean {:.5}, limit {:.5}",
                        shot.name, d.mean, script.limits.pair_mean
                    ));
                }
                Some(d)
            }
            None => None,
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
