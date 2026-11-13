/// Simulation steps per second.
pub const STEP_HZ: u32 = 60;
/// Length of one simulation step in seconds.
pub const STEP_SECONDS: f32 = 1.0 / STEP_HZ as f32;

/// How many steps to run this frame, and how far between the last two snapshots to draw.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct StepPlan {
    pub steps: u32,
    /// 0 draws the previous snapshot, 1 the newest.
    pub alpha: f32,
}

/// Turns variable frame time into whole fixed steps.
///
/// Time is accumulated in integer nanoseconds so that the same sequence of frame times
/// always yields the same sequence of step counts.
#[derive(Clone, Debug)]
pub struct FixedClock {
    accumulator_ns: u64,
    max_steps_per_frame: u32,
}

const STEP_NS: u64 = 1_000_000_000 / STEP_HZ as u64;

impl Default for FixedClock {
    fn default() -> Self {
        Self::new()
    }
}

impl FixedClock {
    pub fn new() -> Self {
        Self { accumulator_ns: 0, max_steps_per_frame: 5 }
    }

    /// Feed the real time since the last frame. A long stall is clamped: the game slows
    /// down for that frame and does not try to catch up with a burst of steps.
    pub fn advance(&mut self, frame_ns: u64) -> StepPlan {
        self.accumulator_ns += frame_ns;
        let mut steps = (self.accumulator_ns / STEP_NS) as u32;
        if steps > self.max_steps_per_frame {
            steps = self.max_steps_per_frame;
            self.accumulator_ns = steps as u64 * STEP_NS;
        }
        self.accumulator_ns -= steps as u64 * STEP_NS;
        StepPlan { steps, alpha: self.accumulator_ns as f32 / STEP_NS as f32 }
    }

    /// A capture or a test: exactly one step each frame, drawn at the newest snapshot.
    pub fn fixed() -> StepPlan {
        StepPlan { steps: 1, alpha: 1.0 }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn whole_steps_and_remainder() {
        let mut c = FixedClock::new();
        let p = c.advance(STEP_NS * 2 + STEP_NS / 2);
        assert_eq!(p.steps, 2);
        assert!((p.alpha - 0.5).abs() < 1e-3);
        let p = c.advance(STEP_NS / 2);
        assert_eq!(p.steps, 1);
        assert!(p.alpha < 1e-3);
    }

    #[test]
    fn a_stall_is_clamped() {
        let mut c = FixedClock::new();
        let p = c.advance(STEP_NS * 500);
        assert_eq!(p.steps, 5);
        assert_eq!(c.advance(0).steps, 0);
    }

    #[test]
    fn same_frame_times_same_plan() {
        let times = [16_000_000u64, 17_300_000, 15_900_000, 33_000_000, 8_000_000];
        let run = || {
            let mut c = FixedClock::new();
            times.iter().cycle().take(1000).map(|&t| c.advance(t).steps).collect::<Vec<_>>()
        };
        assert_eq!(run(), run());
    }
}
