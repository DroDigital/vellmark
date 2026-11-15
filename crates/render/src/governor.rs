//! The governor: the one controller that holds the frame time. Nothing else trades
//! quality for speed on its own.
//!
//! It reads the frame's GPU time every frame, smoothed over half a second. When the
//! frame runs long it gives things up in a fixed order, a tenth at a time, and takes
//! them back in the reverse order when there is room again:
//!
//! 1. how often the far shadow cascade is redrawn,
//! 2. particle counts,
//! 3. grass density beyond 30 m,
//! 4. the fog grid's resolution,
//! 5. contact occlusion,
//! 6. and last the scene's resolution, down to 85%.
//!
//! It never touches anything inside the reveal distance, and the scene's resolution
//! returns to 100% whenever the player is reading. Every step it takes is logged, so a
//! number met by giving something up can be seen in the performance report.

/// The things the governor may give up, in the order it gives them up.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Knob {
    FarShadows,
    Particles,
    FarGrass,
    FogGrid,
    Contact,
    Resolution,
}

pub const KNOBS: [Knob; 6] = [Knob::FarShadows, Knob::Particles, Knob::FarGrass, Knob::FogGrid, Knob::Contact, Knob::Resolution];

/// How much of each thing has been given up: 0 nothing, 1 all the governor may.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Relief {
    pub levels: [f32; 6],
}

impl Default for Relief {
    fn default() -> Self {
        Self { levels: [0.0; 6] }
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

impl Relief {
    fn level(&self, knob: Knob) -> f32 {
        self.levels[KNOBS.iter().position(|k| *k == knob).unwrap()]
    }

    /// The fewest frames between redraws of the far cascade when only the light has moved.
    pub fn far_shadow_frames(&self) -> u32 {
        lerp(1.0, 8.0, self.level(Knob::FarShadows)).round() as u32
    }

    /// How far the light may move, in degrees, before the far cascade is redrawn.
    pub fn far_shadow_degrees(&self) -> f32 {
        lerp(0.25, 1.0, self.level(Knob::FarShadows))
    }

    /// The share of particles drawn.
    pub fn particles(&self) -> f32 {
        lerp(1.0, 0.5, self.level(Knob::Particles))
    }

    /// The share of grass kept beyond 30 m.
    pub fn far_grass(&self) -> f32 {
        lerp(1.0, 0.35, self.level(Knob::FarGrass))
    }

    /// The fog grid's resolution, as a share of its full size along each side.
    pub fn fog_grid(&self) -> f32 {
        lerp(1.0, 0.6, self.level(Knob::FogGrid))
    }

    /// How strongly contact occlusion darkens; at 0 it is not computed.
    pub fn contact(&self) -> f32 {
        1.0 - self.level(Knob::Contact)
    }

    /// The scene's resolution, as a share of the view's along each side.
    pub fn scene_scale(&self) -> f32 {
        lerp(1.0, SCALE_FLOOR, self.level(Knob::Resolution))
    }
}

/// The scene's resolution is never below this share of the display.
pub const SCALE_FLOOR: f32 = 0.85;
/// How much one step gives up or takes back.
const STEP: f32 = 0.1;
/// Seconds over which the frame time is smoothed.
const SMOOTHING: f32 = 0.5;
/// Seconds to wait after a step before the next: giving up is quicker than taking back.
const GIVE_WAIT: f32 = 0.25;
const TAKE_WAIT: f32 = 0.75;

pub struct Governor {
    budget_ms: f32,
    smoothed: Option<f32>,
    wait: f32,
    relief: Relief,
    /// Every step taken: when, what, which way, and the smoothed frame time then.
    pub log: Vec<String>,
}

impl Governor {
    /// `budget_ms` is the frame's whole budget, 16.6 ms for 60 frames a second.
    pub fn new(budget_ms: f32) -> Self {
        Self { budget_ms, smoothed: None, wait: 0.0, relief: Relief::default(), log: Vec::new() }
    }

    pub fn relief(&self) -> Relief {
        self.relief
    }

    /// The smoothed frame time, in milliseconds.
    pub fn smoothed_ms(&self) -> f32 {
        self.smoothed.unwrap_or(0.0)
    }

    /// One frame: the GPU's time for it in milliseconds (or the frame's wall time where
    /// there are no GPU timers), the seconds since the last frame, the clock for the log,
    /// and whether the player is reading.
    pub fn frame(&mut self, frame_ms: f32, dt: f32, seconds: f32, reading: bool) -> Relief {
        let a = 1.0 - (-dt.max(0.0) / SMOOTHING).exp();
        let s = match self.smoothed {
            Some(s) => s + (frame_ms - s) * a,
            None => frame_ms,
        };
        self.smoothed = Some(s);
        let resolution = KNOBS.iter().position(|k| *k == Knob::Resolution).unwrap();
        // Reading: the scene's resolution comes back at once, a step a frame.
        if reading && self.relief.levels[resolution] > 0.0 {
            self.relief.levels[resolution] = (self.relief.levels[resolution] - STEP).max(0.0);
            self.note(seconds, Knob::Resolution, "taken back for reading", s);
        }
        self.wait -= dt;
        if self.wait > 0.0 {
            return self.relief;
        }
        let high = self.budget_ms - 0.6;
        let low = self.budget_ms - 2.5;
        if s > high {
            let next = (0..KNOBS.len()).find(|&i| self.relief.levels[i] < 1.0 && !(reading && i == resolution));
            if let Some(i) = next {
                self.relief.levels[i] = (self.relief.levels[i] + STEP).min(1.0);
                self.wait = GIVE_WAIT;
                self.note(seconds, KNOBS[i], "given up", s);
            }
        } else if s < low {
            if let Some(i) = (0..KNOBS.len()).rev().find(|&i| self.relief.levels[i] > 0.0) {
                self.relief.levels[i] = (self.relief.levels[i] - STEP).max(0.0);
                self.wait = TAKE_WAIT;
                self.note(seconds, KNOBS[i], "taken back", s);
            }
        }
        self.relief
    }

    fn note(&mut self, seconds: f32, knob: Knob, what: &str, smoothed: f32) {
        let i = KNOBS.iter().position(|k| *k == knob).unwrap();
        self.log.push(format!("{seconds:8.2} s  {knob:?} {what}, now {:.0}%, frame {smoothed:.2} ms", self.relief.levels[i] * 100.0));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(g: &mut Governor, ms: f32, seconds: f32, reading: bool) -> Relief {
        let mut r = g.relief();
        for _ in 0..(seconds * 60.0) as usize {
            r = g.frame(ms, 1.0 / 60.0, 0.0, reading);
        }
        r
    }

    #[test]
    fn it_gives_things_up_in_order_and_takes_them_back_in_reverse() {
        let mut g = Governor::new(16.6);
        // A frame that runs long gives up the far shadows first, and touches nothing else
        // until they are all given up.
        let r = run(&mut g, 18.0, 1.5, false);
        assert!(r.levels[0] > 0.0 && r.levels[1..].iter().all(|l| *l == 0.0), "{r:?}");
        let r = run(&mut g, 18.0, 30.0, false);
        assert!(r.levels.iter().all(|l| *l == 1.0), "a long run gives everything: {r:?}");
        assert!((r.scene_scale() - SCALE_FLOOR).abs() < 1.0e-6, "never below the floor");
        // With room again, the resolution comes back first, the far shadows last.
        let r = run(&mut g, 12.0, 3.0, false);
        assert!(r.levels[5] < 1.0 && r.levels[0] == 1.0, "{r:?}");
        let r = run(&mut g, 12.0, 60.0, false);
        assert_eq!(r, Relief::default(), "everything is taken back");
        assert!(!g.log.is_empty());
    }

    #[test]
    fn it_holds_still_inside_the_band_and_steps_a_little_at_a_time() {
        let mut g = Governor::new(16.6);
        // Between the two thresholds nothing changes.
        let r = run(&mut g, 15.0, 10.0, false);
        assert_eq!(r, Relief::default());
        // One frame that runs long does not move it: the time is smoothed.
        g.frame(40.0, 1.0 / 60.0, 0.0, false);
        let r = run(&mut g, 15.0, 0.2, false);
        assert_eq!(r, Relief::default());
        // Each step is a tenth.
        let r = run(&mut g, 20.0, 0.6, false);
        assert!(r.levels[0] <= 0.31, "{r:?}");
    }

    #[test]
    fn reading_brings_the_resolution_back() {
        let mut g = Governor::new(16.6);
        run(&mut g, 20.0, 30.0, false);
        assert!(g.relief().scene_scale() < 1.0);
        let r = run(&mut g, 20.0, 0.5, true);
        assert_eq!(r.scene_scale(), 1.0, "reading takes the resolution back at once and keeps it");
    }
}
