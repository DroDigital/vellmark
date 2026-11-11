//! What every pass of a view shares: the camera, the light of the hour, and the
//! shadow cascades.

use crate::camera::{sun_direction, View};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3, Vec4};

/// Matches `ViewUniform` in `shaders/frame.wgsl`.
#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
pub struct ViewUniform {
    pub view_proj: [[f32; 4]; 4],
    pub inv_view_proj: [[f32; 4]; 4],
    pub view: [[f32; 4]; 4],
    pub eye_time: [f32; 4],
    pub light_dir_hour: [f32; 4],
    pub light_colour: [f32; 4],
    pub sun_dir: [f32; 4],
    pub sky_zenith: [f32; 4],
    pub sky_horizon: [f32; 4],
    pub switches: [f32; 4],
    pub viewport: [f32; 4],
    pub cascade_far: [f32; 4],
    pub shadow: [[[f32; 4]; 4]; 3],
    pub indices: [u32; 4],
    pub viewport_origin: [f32; 4],
    pub effects: [f32; 4],
}

/// The most views drawn in one frame. Two is all the full game will ever ask for; four
/// leaves room for tools.
pub const MAX_VIEWS: usize = 4;
pub const CASCADES: usize = 3;
pub const SHADOW_SIZE: u32 = 2048;

fn smoothstep(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// The light of an hour of the day.
#[derive(Clone, Copy, Debug)]
pub struct Atmosphere {
    pub sun_dir: Vec3,
    /// Towards whichever of the sun and the moon is casting shadows.
    pub light_dir: Vec3,
    pub light_colour: Vec3,
    pub sky_zenith: Vec3,
    pub sky_horizon: Vec3,
    pub exposure: f32,
}

impl Atmosphere {
    pub fn at(hour: f32) -> Self {
        let sun_dir = sun_direction(hour);
        let height = sun_dir.y;
        let day = smoothstep(-0.12, 0.18, height);

        // The sun reddens and weakens as it nears the horizon.
        let low = smoothstep(0.0, 0.38, height);
        let sun_colour = Vec3::new(1.0, 0.50, 0.22).lerp(Vec3::new(1.0, 0.95, 0.88), low)
            * 5.6
            * smoothstep(-0.02, 0.14, height);
        // The moon stands opposite the sun and is far weaker and bluer.
        let moon_dir = Vec3::new(-sun_dir.x, (-sun_dir.y).max(0.25), -sun_dir.z).normalize();
        let moon_colour = Vec3::new(0.26, 0.34, 0.52) * 0.34 * smoothstep(0.02, -0.10, height);
        let (light_dir, light_colour) =
            if height > 0.0 { (sun_dir, sun_colour) } else { (moon_dir, moon_colour) };

        let dusk = smoothstep(0.30, 0.02, height) * day;
        let zenith = Vec3::new(0.012, 0.020, 0.046).lerp(Vec3::new(0.24, 0.38, 0.72), day)
            .lerp(Vec3::new(0.16, 0.22, 0.50), dusk * 0.6);
        let horizon = Vec3::new(0.016, 0.022, 0.040).lerp(Vec3::new(0.58, 0.68, 0.82), day)
            .lerp(Vec3::new(0.95, 0.52, 0.30), dusk * 0.75);

        Self {
            sun_dir,
            light_dir,
            light_colour,
            sky_zenith: zenith,
            sky_horizon: horizon,
            exposure: 2.6 + (0.62 - 2.6) * day,
        }
    }
}

/// How the fine print is tuned. The defaults are the brief's; the Look Gate tunes them.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GlyphTuning {
    /// Scales the distance at which letters begin to show. 1 puts the first trace at
    /// about six metres at 1080p.
    pub reveal: f32,
    /// Scales every material's letter contrast.
    pub contrast: f32,
    /// How letters sit in a surface up close: 0 flat on it, 1 atoms through it,
    /// 2 a lattice in front of deeper layers, 3 a lattice over the glowing source.
    pub look: u32,
}

impl Default for GlyphTuning {
    fn default() -> Self {
        Self { reveal: 1.0, contrast: 1.0, look: 0 }
    }
}

/// The far edge of each cascade, in metres in front of the eye.
pub const CASCADE_FAR: [f32; CASCADES] = [7.0, 26.0, 90.0];

/// The view-projection of one shadow cascade: an orthographic box around the part of
/// the view between `near` and `far`, seen from the light, moved in whole texels so
/// that shadow edges do not crawl as the camera moves.
pub fn cascade_matrix(view: &View, aspect: f32, near: f32, far: f32, light_dir: Vec3) -> Mat4 {
    let tan_y = (view.camera.fov_y * 0.5).tan();
    let tan_x = tan_y * aspect;
    let mut corners = [Vec3::ZERO; 8];
    for (i, corner) in corners.iter_mut().enumerate() {
        let d = if i < 4 { near } else { far };
        let sx = if i & 1 == 0 { -1.0 } else { 1.0 };
        let sy = if i & 2 == 0 { -1.0 } else { 1.0 };
        let in_view = Vec3::new(sx * tan_x * d, sy * tan_y * d, -d);
        *corner = view.camera.eye + view.camera.rotation * in_view;
    }
    let centre = corners.iter().copied().sum::<Vec3>() / 8.0;
    // A sphere, so the box's size does not change as the camera turns.
    let radius = corners.iter().map(|c| c.distance(centre)).fold(0.0f32, f32::max);
    let radius = (radius * 16.0).ceil() / 16.0;

    let up = if light_dir.y.abs() > 0.99 { Vec3::Z } else { Vec3::Y };
    let light_view = crate::camera::look_at(Vec3::ZERO, -light_dir, up);
    let texel = radius * 2.0 / SHADOW_SIZE as f32;
    let mut c = light_view.transform_point3(centre);
    c.x = (c.x / texel).floor() * texel;
    c.y = (c.y / texel).floor() * texel;
    // Casters between the light and the box still cast into it.
    let back = radius + 80.0;
    let projection = crate::camera::orthographic(c.x - radius, c.x + radius, c.y - radius, c.y + radius, -c.z - back, -c.z + radius);
    projection * light_view
}

#[allow(clippy::too_many_arguments)]
pub fn view_uniform(
    view: &View,
    size: (f32, f32),
    atmosphere: &Atmosphere,
    hour: f32,
    seconds: f32,
    glyph_layer: bool,
    close_reading: f32,
    tuning: &GlyphTuning,
    reveal_mask: bool,
    index: u32,
    origin: (f32, f32),
    dive: f32,
    lamps: f32,
    cascades: &[Mat4; CASCADES],
) -> ViewUniform {
    let view_matrix = view.camera.view_matrix();
    let view_proj = view.camera.projection(size.0 / size.1) * view_matrix;
    // The brief's thresholds are for a view 1080 pixels tall; a smaller view shows its
    // letters at the same distances, not nearer.
    let reveal = tuning.reveal * 1080.0 / size.1;
    // World size of a texel of the first cascade, for stepping off surfaces.
    let texel = {
        let inverse = cascades[0].inverse();
        let a = inverse.project_point3(Vec3::ZERO);
        let b = inverse.project_point3(Vec3::new(2.0 / SHADOW_SIZE as f32, 0.0, 0.0));
        a.distance(b)
    };
    ViewUniform {
        view_proj: view_proj.to_cols_array_2d(),
        inv_view_proj: view_proj.inverse().to_cols_array_2d(),
        view: view_matrix.to_cols_array_2d(),
        eye_time: view.camera.eye.extend(seconds).to_array(),
        light_dir_hour: atmosphere.light_dir.extend(hour).to_array(),
        light_colour: atmosphere.light_colour.extend(0.0).to_array(),
        sun_dir: atmosphere.sun_dir.extend(0.0).to_array(),
        sky_zenith: atmosphere.sky_zenith.extend(0.0).to_array(),
        sky_horizon: atmosphere.sky_horizon.extend(0.0).to_array(),
        switches: [if glyph_layer { 1.0 } else { 0.0 }, close_reading, tuning.contrast, reveal],
        viewport: [size.0, size.1, atmosphere.exposure, if reveal_mask { 1.0 } else { 0.0 }],
        cascade_far: Vec4::new(CASCADE_FAR[0], CASCADE_FAR[1], CASCADE_FAR[2], texel).to_array(),
        shadow: [
            cascades[0].to_cols_array_2d(),
            cascades[1].to_cols_array_2d(),
            cascades[2].to_cols_array_2d(),
        ],
        indices: [index, 0, 0, 0],
        viewport_origin: [origin.0, origin.1, 0.0, 0.0],
        effects: [dive, lamps, tuning.look as f32, 0.0],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::camera::Camera;

    #[test]
    fn noon_is_bright_and_midnight_is_moonlit() {
        let noon = Atmosphere::at(12.0);
        let night = Atmosphere::at(0.5);
        assert!(noon.light_colour.max_element() > 5.0);
        assert!(night.light_colour.max_element() < 0.3 && night.light_colour.max_element() > 0.05);
        assert!(night.light_dir.y > 0.0, "the moon is above the horizon");
        assert!(night.exposure > noon.exposure);
        let dusk = Atmosphere::at(17.6);
        assert!(dusk.sky_horizon.x > dusk.sky_horizon.z, "the horizon is warm at dusk");
    }

    #[test]
    fn a_cascade_holds_its_part_of_the_view() {
        let view = View::full(Camera::looking(Vec3::new(3.0, 1.7, 2.0), 0.4, -0.1));
        let light = Atmosphere::at(10.0).light_dir;
        let m = cascade_matrix(&view, 16.0 / 9.0, 0.05, 7.0, light);
        for d in [0.5, 3.0, 6.9] {
            let p = view.camera.eye + view.camera.rotation * Vec3::new(0.0, 0.0, -d);
            let clip = m.project_point3(p);
            assert!(clip.x.abs() <= 1.0 && clip.y.abs() <= 1.0 && (0.0..=1.0).contains(&clip.z), "{clip:?}");
        }
    }
}
