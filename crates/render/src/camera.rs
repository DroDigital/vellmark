use glam::{Mat4, Quat, Vec3};
use vellmark_sim::ViewSnap;

/// Part of the target a view is drawn into, as fractions of its width and height.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Viewport {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Viewport {
    pub const FULL: Self = Self {
        x: 0.0,
        y: 0.0,
        width: 1.0,
        height: 1.0,
    };

    /// The rectangle in pixels: x, y, width, height.
    pub fn pixels(&self, target: (u32, u32)) -> (f32, f32, f32, f32) {
        let (w, h) = (target.0 as f32, target.1 as f32);
        (
            (self.x * w).round(),
            (self.y * h).round(),
            (self.width * w).round().max(1.0),
            (self.height * h).round().max(1.0),
        )
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Camera {
    pub eye: Vec3,
    pub rotation: Quat,
    /// Vertical field of view in radians.
    pub fov_y: f32,
    pub near: f32,
}

impl Camera {
    pub const DEFAULT_FOV_Y: f32 = 1.1345; // 65 degrees
    pub const DEFAULT_NEAR: f32 = 0.05;

    pub fn from_view(view: &ViewSnap) -> Self {
        Self {
            eye: view.eye,
            rotation: view.rotation(),
            fov_y: Self::DEFAULT_FOV_Y,
            near: Self::DEFAULT_NEAR,
        }
    }

    /// Yaw turns right, pitch looks up, both in radians, as in the simulation.
    pub fn looking(eye: Vec3, yaw: f32, pitch: f32) -> Self {
        Self {
            eye,
            rotation: Quat::from_rotation_y(-yaw) * Quat::from_rotation_x(pitch),
            fov_y: Self::DEFAULT_FOV_Y,
            near: Self::DEFAULT_NEAR,
        }
    }

    pub fn view_matrix(&self) -> Mat4 {
        Mat4::from_rotation_translation(self.rotation, self.eye).inverse()
    }

    /// Reversed depth with an infinite far plane: 1 at the near plane, 0 at infinity.
    /// This keeps depth precise at every distance the valley can show.
    pub fn projection(&self, aspect: f32) -> Mat4 {
        let f = 1.0 / (self.fov_y * 0.5).tan();
        Mat4::from_cols_array(&[
            f / aspect,
            0.0,
            0.0,
            0.0,
            0.0,
            f,
            0.0,
            0.0,
            0.0,
            0.0,
            0.0,
            -1.0,
            0.0,
            0.0,
            self.near,
            0.0,
        ])
    }

    pub fn view_projection(&self, aspect: f32) -> Mat4 {
        self.projection(aspect) * self.view_matrix()
    }
}

/// A right-handed view matrix looking from `eye` towards `target`. -Z is forward.
pub fn look_at(eye: Vec3, target: Vec3, up: Vec3) -> Mat4 {
    let f = (target - eye).normalize();
    let s = f.cross(up).normalize();
    let u = s.cross(f);
    Mat4::from_cols_array(&[
        s.x,
        u.x,
        -f.x,
        0.0,
        s.y,
        u.y,
        -f.y,
        0.0,
        s.z,
        u.z,
        -f.z,
        0.0,
        -s.dot(eye),
        -u.dot(eye),
        f.dot(eye),
        1.0,
    ])
}

/// A right-handed orthographic projection with depth from 0 at `near` to 1 at `far`.
pub fn orthographic(left: f32, right: f32, bottom: f32, top: f32, near: f32, far: f32) -> Mat4 {
    let (w, h, d) = (right - left, top - bottom, far - near);
    Mat4::from_cols_array(&[
        2.0 / w,
        0.0,
        0.0,
        0.0,
        0.0,
        2.0 / h,
        0.0,
        0.0,
        0.0,
        0.0,
        -1.0 / d,
        0.0,
        -(right + left) / w,
        -(top + bottom) / h,
        -near / d,
        1.0,
    ])
}

/// One camera and where on the target it is drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct View {
    pub camera: Camera,
    pub viewport: Viewport,
}

impl View {
    pub fn full(camera: Camera) -> Self {
        Self {
            camera,
            viewport: Viewport::FULL,
        }
    }
}

/// Direction towards the sun for an hour of the day. The sun rises in the east (+X) at
/// six, stands in the south (+Z) at noon, and sets in the west at eighteen.
pub fn sun_direction(hour: f32) -> Vec3 {
    let angle = (hour - 6.0) / 24.0 * std::f32::consts::TAU;
    let (sin, cos) = angle.sin_cos();
    // Tilted towards the south, so that noon shadows fall north and are never straight down.
    Vec3::new(cos, sin * 0.82, sin * 0.57 + 0.12).normalize()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sun_rises_east_and_sets_west() {
        assert!(sun_direction(6.5).x > 0.9);
        assert!(sun_direction(12.0).y > 0.7);
        assert!(sun_direction(17.5).x < -0.9);
        assert!(sun_direction(0.0).y < -0.7);
    }

    #[test]
    fn the_point_ahead_projects_to_the_centre() {
        let camera = Camera::looking(Vec3::new(1.0, 2.0, 3.0), 0.7, 0.2);
        let ahead = camera.eye + camera.rotation * Vec3::NEG_Z * 10.0;
        let clip = camera.view_projection(16.0 / 9.0) * ahead.extend(1.0);
        let ndc = clip.truncate() / clip.w;
        assert!(ndc.x.abs() < 1e-4 && ndc.y.abs() < 1e-4);
        assert!(ndc.z > 0.0 && ndc.z < 1.0);
    }

    #[test]
    fn viewport_pixels() {
        let left = Viewport {
            x: 0.0,
            y: 0.0,
            width: 0.5,
            height: 1.0,
        };
        assert_eq!(left.pixels((1920, 1080)), (0.0, 0.0, 960.0, 1080.0));
    }
}
