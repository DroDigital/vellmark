//! The atmosphere, on the processor: the same single-scattering model the sky shader
//! draws, evaluated in a few directions a frame to give the sun's and the moon's colour
//! where they reach the ground, and the sky's light on surfaces. So the light on a wall
//! at dusk is the light of the sky above it, not a separate guess.
//!
//! Air scatters blue far more than red (Rayleigh); haze scatters all colours alike and
//! mostly forwards (Mie); ozone takes a little orange and is what keeps the sky blue
//! after sunset. Lengths are in metres. Radiance is per unit of light arriving at the
//! top of the atmosphere; `SUN` turns that into the renderer's units.

use glam::Vec3;

pub const EARTH: f32 = 6_360_000.0;
pub const TOP: f32 = 6_460_000.0;
pub const RAYLEIGH: Vec3 = Vec3::new(5.802e-6, 13.558e-6, 33.1e-6);
pub const RAYLEIGH_HEIGHT: f32 = 8_000.0;
pub const MIE_SCATTER: f32 = 3.996e-6;
pub const MIE_EXTINCTION: f32 = 4.40e-6;
pub const MIE_HEIGHT: f32 = 1_200.0;
pub const MIE_G: f32 = 0.8;
pub const OZONE: Vec3 = Vec3::new(0.650e-6, 1.881e-6, 0.085e-6);
/// The sun at the top of the atmosphere, in the renderer's units of light.
pub const SUN: f32 = 6.4;
/// The full moon's light as a share of the sun's. The real share is about one in four
/// hundred thousand; the eye adapts to night, and the game must still be played in it.
pub const MOON: f32 = 0.05;
/// Starlight and airglow: the night sky's own faint light, in the renderer's units.
pub const NIGHT_GLOW: Vec3 = Vec3::new(0.0016, 0.0022, 0.0042);
/// The eye's height above the sea for the sky, wherever it is in the valley.
const EYE: f32 = 120.0;
/// Single scattering leaves out the light the air scatters more than once, which is
/// most of the light in shade. The sky's light on surfaces is raised by this to stand
/// for it; the sky as seen is not.
pub const MULTIPLE: f32 = 1.5;

/// How much air, haze, and ozone there is at a height, as shares of sea level's.
fn density(height: f32) -> (f32, f32, f32) {
    let ozone = (1.0 - (height - 25_000.0).abs() / 15_000.0).max(0.0);
    ((-height / RAYLEIGH_HEIGHT).exp(), (-height / MIE_HEIGHT).exp(), ozone)
}

fn extinction(height: f32) -> Vec3 {
    let (r, m, o) = density(height.max(0.0));
    RAYLEIGH * r + Vec3::splat(MIE_EXTINCTION * m) + OZONE * o
}

/// Distance along a ray from a point at `radius` going at `cos` to the vertical, to a
/// sphere of radius `to`; negative if it is not reached going forward.
fn to_sphere(radius: f32, cos: f32, to: f32) -> f32 {
    let b = radius * cos;
    let c = radius * radius - to * to;
    let d = b * b - c;
    if d < 0.0 {
        return -1.0;
    }
    let s = d.sqrt();
    if -b - s > 0.0 {
        -b - s
    } else {
        -b + s
    }
}

/// How much light gets through from the top of the atmosphere to a point at `height`,
/// arriving from a direction `cos` from the vertical. Nothing, if the earth is between.
pub fn transmittance(height: f32, cos: f32) -> Vec3 {
    let radius = EARTH + height.max(0.0);
    let ground = to_sphere(radius, cos, EARTH);
    if ground > 0.0 && cos < 0.0 {
        return Vec3::ZERO;
    }
    let length = to_sphere(radius, cos, TOP).max(0.0);
    let steps = 12;
    let ds = length / steps as f32;
    let mut depth = Vec3::ZERO;
    for i in 0..steps {
        let t = (i as f32 + 0.5) * ds;
        let r = (radius * radius + t * t + 2.0 * radius * t * cos).sqrt();
        depth += extinction(r - EARTH) * ds;
    }
    (-depth).exp()
}

fn rayleigh_phase(mu: f32) -> f32 {
    3.0 / (16.0 * std::f32::consts::PI) * (1.0 + mu * mu)
}

fn mie_phase(mu: f32) -> f32 {
    let g = MIE_G;
    let k = 3.0 / (8.0 * std::f32::consts::PI) * (1.0 - g * g) / (2.0 + g * g);
    k * (1.0 + mu * mu) / (1.0 + g * g - 2.0 * g * mu).powf(1.5)
}

/// The light of the sky seen in direction `view`, lit from direction `light`, per unit
/// of light at the top of the atmosphere.
pub fn radiance(view: Vec3, light: Vec3) -> Vec3 {
    let radius = EARTH + EYE;
    let cos = view.y;
    let ground = to_sphere(radius, cos, EARTH);
    let length = if ground > 0.0 && cos < 0.0 { ground } else { to_sphere(radius, cos, TOP).max(0.0) };
    let mu = view.dot(light);
    let (pr, pm) = (rayleigh_phase(mu), mie_phase(mu));
    let steps = 24;
    let ds = length / steps as f32;
    let mut through = Vec3::ONE;
    let mut gathered = Vec3::ZERO;
    for i in 0..steps {
        let t = (i as f32 + 0.5) * ds;
        let point = Vec3::new(0.0, radius, 0.0) + view * t;
        let r = point.length();
        let up = point / r;
        let height = r - EARTH;
        let (dr, dm, _) = density(height);
        let sun = transmittance(height, up.dot(light));
        let scatter = (RAYLEIGH * dr * pr + Vec3::splat(MIE_SCATTER * dm * pm)) * sun;
        let step = (-extinction(height) * ds).exp();
        // The light scattered over this step, as it reaches the eye.
        gathered += through * scatter * ds;
        through *= step;
    }
    gathered
}

/// The light of the sky and its sources at an hour, in the renderer's units.
#[derive(Clone, Copy, Debug)]
pub struct SkyLight {
    pub sun: Vec3,
    pub moon: Vec3,
    /// How much of the moon's face is lit, 0 to 1.
    pub moon_lit: f32,
    pub zenith: Vec3,
    pub horizon: Vec3,
}

/// The sun and the moon as they reach the valley, and the sky's light straight up and at
/// the horizon.
pub fn sky_light(sun_dir: Vec3, moon_dir: Vec3) -> SkyLight {
    let moon_lit = 0.5 * (1.0 - sun_dir.dot(moon_dir));
    let sun = transmittance(EYE, sun_dir.y) * SUN;
    let moon_strength = SUN * MOON * moon_lit;
    let moon = transmittance(EYE, moon_dir.y) * moon_strength;
    let sky = |dir: Vec3| radiance(dir, sun_dir) * SUN + radiance(dir, moon_dir) * moon_strength + NIGHT_GLOW;
    let zenith = sky(Vec3::Y) * MULTIPLE;
    let horizon = [Vec3::X, Vec3::NEG_X, Vec3::Z, Vec3::NEG_Z]
        .iter()
        .map(|d| sky((*d + Vec3::Y * 0.09).normalize()))
        .sum::<Vec3>()
        * 0.25
        * MULTIPLE;
    SkyLight { sun, moon, moon_lit, zenith, horizon }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_noon_sky_is_blue_and_the_sun_white() {
        let sun = Vec3::new(0.2, 0.9, 0.3).normalize();
        let s = sky_light(sun, -sun);
        assert!(s.zenith.z > s.zenith.x * 1.8, "the zenith is blue: {}", s.zenith);
        assert!(s.sun.x > 0.8 * s.sun.z && s.sun.z > 0.7 * s.sun.x, "the noon sun is near white: {}", s.sun);
        assert!(s.horizon.length() > s.zenith.length() * 0.6, "the horizon is pale and bright: {}", s.horizon);
    }

    #[test]
    fn the_setting_sun_is_red_and_night_is_dark() {
        let low = Vec3::new(-0.999, 0.03, 0.0).normalize();
        let s = sky_light(low, -low);
        assert!(s.sun.x > s.sun.z * 2.5, "the setting sun is red: {}", s.sun);
        let night = Vec3::new(0.3, -0.6, 0.2).normalize();
        let n = sky_light(night, Vec3::new(0.0, 0.5, 0.8).normalize());
        let noon = sky_light(Vec3::Y, -Vec3::Y);
        assert!(n.zenith.length() < noon.zenith.length() * 0.05, "night is dark: {}", n.zenith);
        assert_eq!(n.sun, Vec3::ZERO, "the sun below the earth sends no light");
    }

    #[test]
    fn the_moon_waxes_as_it_draws_away_from_the_sun() {
        let sun = Vec3::new(1.0, -0.3, 0.0).normalize();
        let near = sky_light(sun, Vec3::new(0.9, 0.3, 0.0).normalize());
        let opposite = sky_light(sun, Vec3::new(-0.9, 0.4, 0.0).normalize());
        assert!(opposite.moon_lit > 0.9 && near.moon_lit < 0.3);
    }
}

#[cfg(test)]
mod print {
    #[test]
    #[ignore]
    fn print_the_day() {
        for hour in [0.0f32, 3.0, 5.5, 6.0, 6.4, 7.0, 9.0, 13.0, 17.0, 18.0, 18.4, 19.0, 21.0] {
            let a = crate::frame::Atmosphere::at(hour);
            let l = |v: glam::Vec3| v.dot(glam::Vec3::new(0.2126, 0.7152, 0.0722));
            eprintln!("{hour:5.1} sun_y {:6.3} light {:7.4} zenith {:7.5} horizon {:7.5} exposure {:5.2}", a.sun_dir.y, l(a.light_colour), l(a.sky_zenith), l(a.sky_horizon), a.exposure);
        }
    }
}
