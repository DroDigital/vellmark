//! Redactors: flying things drawn in solid black strokes that cross out whatever they
//! touch. One flies a path at its own pace; flying low, it strikes through everything its
//! line passes, which then stands as black rules with nothing behind them, and wounds a
//! player it catches, badly but never to death. Flying high, as at the first vista, it
//! strikes nothing. Struck things stay struck until their Redactor falls.

use glam::Vec3;

#[derive(Clone, Debug, PartialEq)]
pub struct Redactor {
    pub id: u64,
    /// The way it flies, point to point.
    pub path: Vec<Vec3>,
    /// Metres a second.
    pub speed: f32,
    /// Metres flown along the path.
    pub flown: f32,
    /// How far to each side of its line, and how far below and above it, it strikes; none
    /// when it flies high and strikes nothing.
    pub strike: Option<(f32, f32)>,
    /// The players its line has caught, by ID: each is wounded once.
    pub caught: Vec<u32>,
}

/// What a blow from a Redactor's line takes from a player, and the least it leaves them:
/// it wounds badly and never kills.
pub const WOUND: f32 = 70.0;
pub const LEAVES: f32 = 1.0;

impl Redactor {
    pub fn length(&self) -> f32 {
        self.path.windows(2).map(|w| w[0].distance(w[1])).sum()
    }

    pub fn done(&self) -> bool {
        self.flown >= self.length()
    }

    /// Where it is and the way it is going, `flown` metres along.
    pub fn at(&self, flown: f32) -> (Vec3, Vec3) {
        let mut left = flown.max(0.0);
        for w in self.path.windows(2) {
            let d = w[0].distance(w[1]);
            let dir = (w[1] - w[0]).normalize_or_zero();
            if left <= d {
                return (w[0] + dir * left, dir);
            }
            left -= d;
        }
        let n = self.path.len();
        let dir = if n >= 2 { (self.path[n - 1] - self.path[n - 2]).normalize_or_zero() } else { Vec3::X };
        (self.path.last().copied().unwrap_or(Vec3::ZERO), dir)
    }

    /// Whether a point lies in what it strikes between `from` and `to` metres along: within
    /// its reach to the side of its line and within its reach below and above.
    pub fn strikes(&self, from: f32, to: f32, point: Vec3, radius: f32) -> bool {
        let Some((side, depth)) = self.strike else { return false };
        let steps = (((to - from) / 0.5).ceil() as usize).max(1);
        (0..=steps).any(|k| {
            let (p, dir) = self.at(from + (to - from) * k as f32 / steps as f32);
            let flat = Vec3::new(dir.x, 0.0, dir.z).normalize_or_zero();
            let off = point - p;
            let across = (off - flat * off.dot(flat)).with_y(0.0).length();
            let along = off.dot(flat).abs();
            across < side + radius && along < 0.6 + radius && (off.y).abs() < depth + radius
        })
    }
}

/// A thing struck through: its ID and the box it stood in, which its rules fill.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Struck {
    pub id: u64,
    pub min: Vec3,
    pub max: Vec3,
    /// When it was struck, in world seconds: its rules are drawn on across it.
    pub at: f64,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn line() -> Redactor {
        Redactor { id: 1, path: vec![Vec3::new(0.0, 1.0, 0.0), Vec3::new(0.0, 1.0, -100.0)], speed: 12.0, flown: 0.0, strike: Some((2.5, 2.0)), caught: Vec::new() }
    }

    #[test]
    fn it_flies_its_path_and_strikes_only_what_its_line_passes() {
        let r = line();
        assert_eq!(r.length(), 100.0);
        assert_eq!(r.at(50.0).0, Vec3::new(0.0, 1.0, -50.0));
        assert!(r.strikes(40.0, 60.0, Vec3::new(1.5, 0.5, -55.0), 0.3), "beside the line, within reach");
        assert!(!r.strikes(40.0, 60.0, Vec3::new(6.0, 0.5, -55.0), 0.3), "well clear of it");
        assert!(!r.strikes(40.0, 60.0, Vec3::new(0.0, 0.5, -80.0), 0.3), "not yet reached");
        let high = Redactor { strike: None, ..line() };
        assert!(!high.strikes(40.0, 60.0, Vec3::new(0.0, 1.0, -55.0), 0.3), "flying high, it strikes nothing");
    }
}
