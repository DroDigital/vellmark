//! Development tools. Nothing in this module exists in a shipping build.

pub mod capture;
pub mod overlay;
pub mod reload;

pub use overlay::Command;

use crate::log;
use glam::{Quat, Vec3};
use reload::HotReload;
use vellmark_gpu::{Gpu, PipelineRegistry};
use vellmark_platform::{Action, ActionState, Bindings, RawInput};
use vellmark_render::{Camera, Renderer, ShaderLibrary};

/// A camera that flies free of the player.
struct FreeCamera {
    eye: Vec3,
    yaw: f32,
    pitch: f32,
}

pub struct DevTools {
    reload: Option<HotReload>,
    free_camera: Option<FreeCamera>,
    /// How long the last reload took, from the file changing to the game showing it.
    pub last_reload_ms: Option<f32>,
    /// Text or data changed on disk; the game loop rebakes and reloads it.
    data_changed: Option<std::time::Instant>,
    overlay: overlay::Overlay,
}

impl DevTools {
    pub fn new(profiler: bool) -> Self {
        let reload = match crate::paths::repo_root().map(|root| HotReload::watch(&root)) {
            Some(Ok(r)) => Some(r),
            Some(Err(e)) => {
                log::line(&format!("hot reload is off: {e}"));
                None
            }
            None => None,
        };
        let mut overlay = overlay::Overlay::new();
        overlay.profiler = profiler;
        Self { reload, free_camera: None, last_reload_ms: None, data_changed: None, overlay }
    }

    /// Applies any change on disk. Call once a frame.
    pub fn frame(
        &mut self,
        gpu: &Gpu,
        registry: &mut PipelineRegistry,
        renderer: &mut Renderer,
        shaders: &ShaderLibrary,
        actions: &ActionState,
    ) {
        if actions.pressed(Action::DevProfiler) {
            self.overlay.profiler = !self.overlay.profiler;
        }
        let Some(reload) = self.reload.as_mut() else { return };
        let (changes, seen) = reload.poll();
        if changes.data {
            self.data_changed = seen;
        }
        if changes.shaders {
            match renderer.reload_shaders(gpu, registry, shaders) {
                Ok(()) => {
                    let ms = seen.map(|t| t.elapsed().as_secs_f32() * 1000.0).unwrap_or(0.0);
                    self.last_reload_ms = Some(ms);
                    log::line(&format!("shaders reloaded in {ms:.1} ms"));
                }
                Err(e) => log::line(&format!("shader reload failed, keeping the old one:\n{e}")),
            }
        }
    }

    /// The tuning panel: F4 opens it, and the arrow keys change the reveal distance
    /// and the letter contrast while it is open.
    pub fn tune(&mut self, tuning: &mut vellmark_render::GlyphTuning, actions: &ActionState) {
        if actions.pressed(Action::DevTuning) {
            self.overlay.tuning = !self.overlay.tuning;
        }
        self.overlay.tune(
            tuning,
            actions.pressed(Action::DevNext),
            actions.pressed(Action::DevLess),
            actions.pressed(Action::DevMore),
        );
    }

    /// Lets the console take the keyboard while it is open.
    pub fn console_input(&mut self, raw: &mut Vec<RawInput>) {
        self.overlay.console_input(raw);
    }

    /// Opens or closes the console. Returns whether it is now open.
    pub fn console_open(&self) -> bool {
        self.overlay.console
    }

    pub fn toggle_console(&mut self) -> bool {
        self.overlay.console = !self.overlay.console;
        self.overlay.console
    }

    pub fn take_commands(&mut self) -> Vec<Command> {
        self.overlay.take_commands()
    }

    #[allow(clippy::too_many_arguments)]
    pub fn draw(
        &mut self,
        canvas: &mut vellmark_ui::Canvas,
        atlas: &vellmark_content::GlyphAtlas,
        target: (u32, u32),
        density: f32,
        frame_time: std::time::Duration,
        passes: &[vellmark_gpu::PassTiming],
        registry: &PipelineRegistry,
        tuning: &vellmark_render::GlyphTuning,
    ) {
        self.overlay.draw(canvas, atlas, target, density, frame_time, passes, registry, self.last_reload_ms, tuning);
    }

    /// True once after text or data sheets changed on disk, with when it was seen.
    pub fn take_data_change(&mut self) -> Option<std::time::Instant> {
        self.data_changed.take()
    }

    pub fn note_reload(&mut self, what: &str, seen: std::time::Instant) {
        let ms = seen.elapsed().as_secs_f32() * 1000.0;
        self.last_reload_ms = Some(ms);
        log::line(&format!("{what} reloaded in {ms:.1} ms"));
    }

    /// When the free camera is on, it replaces the player's view and takes the
    /// movement input.
    pub fn override_camera(&mut self, camera: &mut Camera, actions: &ActionState, bindings: &Bindings, dt: f32) {
        if actions.pressed(Action::DevFreeCamera) {
            self.free_camera = match self.free_camera {
                Some(_) => None,
                None => {
                    let (yaw, pitch) = yaw_pitch_of(camera.rotation);
                    Some(FreeCamera { eye: camera.eye, yaw, pitch })
                }
            };
        }
        let Some(free) = self.free_camera.as_mut() else { return };
        let (look_yaw, look_pitch) = actions.look(bindings, dt);
        free.yaw += look_yaw;
        free.pitch = (free.pitch + look_pitch).clamp(-1.55, 1.55);
        let (x, y) = actions.movement();
        let speed = if actions.held(Action::Sprint) { 40.0 } else { 6.0 };
        let rotation = Quat::from_rotation_y(-free.yaw) * Quat::from_rotation_x(free.pitch);
        let rise = actions.held(Action::Jump) as i32 as f32 - actions.held(Action::Sneak) as i32 as f32;
        free.eye += (rotation * Vec3::new(x, 0.0, -y) + Vec3::Y * rise) * speed * dt;
        *camera = Camera::looking(free.eye, free.yaw, free.pitch);
    }
}

fn yaw_pitch_of(rotation: Quat) -> (f32, f32) {
    let forward = rotation * Vec3::NEG_Z;
    (forward.x.atan2(-forward.z), forward.y.clamp(-1.0, 1.0).asin())
}
