//! The weather: how hard it rains, how wet the world is, and the wind. Kept by the
//! simulation, so every view and every player sees the same sky, and sent to the
//! renderer in the snapshot.

use crate::clock::STEP_SECONDS;

/// The weather of one step.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Weather {
    /// How hard it is raining: 0 dry to 1 a downpour.
    pub rain: f32,
    /// How wet the world is: 0 dry to 1 soaked. It follows the rain, slowly, and dries
    /// more slowly still.
    pub wetness: f32,
    /// The wind's direction over the ground, x and z, of length 1.
    pub wind: [f32; 2],
    /// Its strength: 0 still to 1 a gale.
    pub wind_strength: f32,
}

impl Default for Weather {
    /// Dry, with the wind as `WeatherState::step` has it at the clock's start.
    fn default() -> Self {
        let angle = WIND_ANGLE;
        Self { rain: 0.0, wetness: 0.0, wind: [angle.cos(), angle.sin()], wind_strength: 0.45 }
    }
}

/// The wind's direction at the clock's start, in radians from +x towards +z.
const WIND_ANGLE: f32 = 0.7;

/// How fast the rain comes on or eases, per second.
const RAIN_RATE: f32 = 0.08;
/// Seconds for a downpour to soak dry ground, and for soaked ground to dry.
const SOAK_SECONDS: f32 = 60.0;
const DRY_SECONDS: f32 = 480.0;

/// What the weather is heading for, and the weather now.
#[derive(Clone, Debug, Default)]
pub struct WeatherState {
    pub now: Weather,
    pub rain_wanted: f32,
}

impl WeatherState {
    /// One step: the rain moves towards what is wanted, the ground wets and dries, and
    /// the wind wanders in direction and strength, harder while it rains. `seconds` is
    /// the world's clock, which keeps the wind the same on every run.
    pub fn step(&mut self, seconds: f64) {
        let w = &mut self.now;
        let to = self.rain_wanted.clamp(0.0, 1.0);
        let most = RAIN_RATE * STEP_SECONDS;
        w.rain += (to - w.rain).clamp(-most, most);
        if w.rain > 0.0 {
            w.wetness = (w.wetness + w.rain * STEP_SECONDS / SOAK_SECONDS).min(1.0);
        } else {
            w.wetness = (w.wetness - STEP_SECONDS / DRY_SECONDS).max(0.0);
        }
        let t = seconds as f32;
        let angle = WIND_ANGLE + 0.35 * (t / 47.0).sin() + 0.1 * (t / 13.0).sin();
        w.wind = [angle.cos(), angle.sin()];
        w.wind_strength = (0.45 + 0.2 * (t / 23.0).sin() + 0.25 * w.rain).clamp(0.0, 1.0);
    }

    /// Sets the weather at once, wet as long rain would leave it: for tools and tests.
    pub fn set_now(&mut self, rain: f32) {
        self.rain_wanted = rain.clamp(0.0, 1.0);
        self.now.rain = self.rain_wanted;
        self.now.wetness = if self.rain_wanted > 0.0 { 1.0 } else { 0.0 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rain_soaks_the_ground_then_it_dries() {
        let mut s = WeatherState { rain_wanted: 1.0, ..Default::default() };
        let mut t = 0.0;
        for _ in 0..(90.0 / STEP_SECONDS) as usize {
            s.step(t);
            t += STEP_SECONDS as f64;
        }
        assert_eq!(s.now.rain, 1.0);
        assert!(s.now.wetness > 0.95, "ninety seconds of rain soak the ground: {}", s.now.wetness);
        s.rain_wanted = 0.0;
        for _ in 0..(60.0 / STEP_SECONDS) as usize {
            s.step(t);
            t += STEP_SECONDS as f64;
        }
        assert_eq!(s.now.rain, 0.0);
        assert!(s.now.wetness > 0.5 && s.now.wetness < 1.0, "the ground dries slowly: {}", s.now.wetness);
    }
}
