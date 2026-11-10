//! Development tools. Nothing in this module exists in a shipping build.

pub mod capture;
pub mod reload;

use crate::log;
use glam::{Quat, Vec3};
use reload::HotReload;
use vellmark_gpu::{Gpu, PipelineRegistry};
use vellmark_platform::{Action, ActionState, Bindings};
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
}

impl DevTools {
    pub fn new() -> Self {
        let reload = match HotReload::watch(&crate::paths::repo_root()) {
            Ok(r) => Some(r),
            Err(e) => {
                log::line(&format!("hot reload is off: {e}"));
                None
            }
        };
        Self { reload, free_camera: None, last_reload_ms: None, data_changed: None }
    }

    /// Applies any change on disk. Call once a frame.
    pub fn frame(
        &mut self,
        gpu: &Gpu,
        registry: &mut PipelineRegistry,
        renderer: &mut Renderer,
        shaders: &ShaderLibrary,
        _actions: &ActionState,
    ) {
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
