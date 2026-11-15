//! The Line, from the script: on the road, in open country, the birds stop; Obelus comes
//! down the road from behind, low, striking through everything beside it, faster than a
//! walk, so that the player must get clear of the road; it does not turn, goes on north,
//! and is gone. In the Showcase it comes on the first night after the player has found
//! four words, once they are near the road, and the end card follows it.
//!
//! The road is given as points at the height Obelus flies over it. Everything else is
//! worked out when it comes, from where the player stands and the way they face.

use glam::Vec3;

/// The flag set once it has passed; it never comes again, and a save remembers it.
pub const FLAG: &str = "line.passed";
/// The event that asks for the end card, a while after it has gone.
pub const END_EVENT: &str = "end_card";
/// Words the player must know first: the hill's and three of the village's.
pub const WORDS_BEFORE: usize = 4;
/// How near the road the player must be, in metres across.
pub const NEAR_ROAD: f32 = 30.0;
/// Metres a second: faster than a sprint, so the way out is off the road, not along it.
pub const SPEED: f32 = 9.0;
/// How far to each side of its line and below and above it it strikes: the road, its
/// verges, and a hedge or milestone beside it.
pub const STRIKE: (f32, f32) = (4.0, 2.4);
/// How far behind the player along the road it comes down to the road.
pub const BEHIND: f32 = 90.0;
/// The long, falling approach before that, from high up the road's line, so the birds stop
/// and its strokes are heard before it is seen.
pub const APPROACH: f32 = 220.0;
pub const APPROACH_HEIGHT: f32 = 45.0;
/// How high over the ground it keeps where it is not on the road: coming in and going
/// north, it passes over hills rather than through them.
pub const CLEAR: f32 = 8.0;
/// Seconds after it is gone before the end card.
pub const END_AFTER: f32 = 7.0;
/// The Redactor's ID while it flies the Line.
pub const OBELUS: u64 = 1;

/// The hours it may come: the night.
pub fn night(hour: f32) -> bool {
    !(4.0..21.0).contains(&hour)
}

/// Where the Line stands.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum State {
    /// Not yet: waiting for the night, the words, and the road.
    #[default]
    Waiting,
    /// Obelus is coming down the road.
    Flying,
    /// Gone north, for this many seconds; the end card follows.
    Gone(f32),
    /// Over.
    Passed,
}

/// The nearest point on the road to `at`, across the ground: its distance, the segment
/// it is on, and how far along the road it is.
fn nearest(road: &[Vec3], at: Vec3) -> Option<(f32, usize, f32)> {
    let flat = |v: Vec3| Vec3::new(v.x, 0.0, v.z);
    let mut best: Option<(f32, usize, f32)> = None;
    let mut along = 0.0;
    for (i, w) in road.windows(2).enumerate() {
        let (a, b) = (flat(w[0]), flat(w[1]));
        let ab = b - a;
        let len = ab.length();
        let t = if len > 0.0 { ((flat(at) - a).dot(ab) / (len * len)).clamp(0.0, 1.0) } else { 0.0 };
        let d = (a + ab * t).distance(flat(at));
        if best.is_none_or(|b| d < b.0) {
            best = Some((d, i, along + t * len));
        }
        along += w[0].distance(w[1]);
    }
    best
}

/// How far a point is from the road, across the ground.
pub fn distance_from_road(road: &[Vec3], at: Vec3) -> f32 {
    nearest(road, at).map_or(f32::MAX, |n| n.0)
}

/// The way Obelus flies for a player at `feet` facing `facing`: down from high up the
/// road's line, onto the road `BEHIND` metres behind them, along it past them in the way
/// they face, and on to the road's end; then north and up, until it is gone.
pub fn flight(road: &[Vec3], feet: Vec3, facing: Vec3) -> Vec<Vec3> {
    let Some((_, seg, _)) = nearest(road, feet) else { return Vec::new() };
    let dir = road[seg + 1] - road[seg];
    // Run the road the way the player faces, so that it comes from behind.
    let mut road: Vec<Vec3> = road.to_vec();
    if Vec3::new(dir.x, 0.0, dir.z).dot(Vec3::new(facing.x, 0.0, facing.z)) < 0.0 {
        road.reverse();
    }
    let (_, _, here) = nearest(&road, feet).unwrap_or((0.0, 0, 0.0));
    let start_at = (here - BEHIND).max(0.0);
    // The road from that point on.
    let mut path = Vec::new();
    let mut along = 0.0;
    for w in road.windows(2) {
        let len = w[0].distance(w[1]);
        if along + len >= start_at {
            if path.is_empty() {
                let t = ((start_at - along) / len.max(1e-3)).clamp(0.0, 1.0);
                path.push(w[0].lerp(w[1], t));
            }
            path.push(w[1]);
        }
        along += len;
    }
    let Some(&first) = path.first() else { return Vec::new() };
    let lead = path.get(1).map_or(Vec3::NEG_Z, |n| (*n - first).normalize_or_zero());
    // The approach, high and falling, down the road's own line.
    let flat_lead = Vec3::new(lead.x, 0.0, lead.z).normalize_or(Vec3::NEG_Z);
    path.insert(0, first - flat_lead * APPROACH + Vec3::Y * APPROACH_HEIGHT);
    path.insert(1, first - flat_lead * (APPROACH * 0.3) + Vec3::Y * (APPROACH_HEIGHT * 0.2));
    // North (the way a view at no turn looks) and up, and gone.
    let last = *path.last().unwrap_or(&first);
    path.push(last + Vec3::new(0.0, 30.0, -120.0));
    path.push(last + Vec3::new(0.0, 140.0, -400.0));
    path
}

/// How high over the road it flies, and how often the road is measured for it.
pub const OVER_ROAD: f32 = 1.5;
const ROAD_STEP: f32 = 4.0;

/// A road given across the ground, as Obelus flies it: a point every few metres, each at
/// its height over the ground there, so that the line follows every rise.
pub fn road_over(points: &[[f32; 2]], height: impl Fn(f32, f32) -> Option<f32>) -> Vec<Vec3> {
    let mut out = Vec::new();
    for w in points.windows(2) {
        let (a, b) = (glam::Vec2::from(w[0]), glam::Vec2::from(w[1]));
        let steps = ((a.distance(b) / ROAD_STEP).ceil() as usize).max(1);
        for k in 0..steps {
            let at = a.lerp(b, k as f32 / steps as f32);
            out.push(Vec3::new(at.x, height(at.x, at.y).unwrap_or(0.0) + OVER_ROAD, at.y));
        }
    }
    if let Some(last) = points.last() {
        out.push(Vec3::new(last[0], height(last[0], last[1]).unwrap_or(0.0) + OVER_ROAD, last[1]));
    }
    out
}

/// Lifts the parts of a flight off the road, the first two and last two legs, until
/// every point along them is `CLEAR` metres over the ground. The road itself is flown as
/// it is given.
pub fn clear_ground(path: &mut [Vec3], height: impl Fn(f32, f32) -> Option<f32>) {
    let n = path.len();
    if n < 4 {
        return;
    }
    let lifted = [0, 1, n - 2, n - 1];
    for _ in 0..8 {
        let mut moved = false;
        for i in 0..n - 1 {
            let (a, b) = (path[i], path[i + 1]);
            let mut short = 0.0f32;
            for k in 0..=32 {
                let at = a.lerp(b, k as f32 / 32.0);
                if let Some(g) = height(at.x, at.z) {
                    short = short.max(g + CLEAR - at.y);
                }
            }
            if short > 0.01 {
                for j in [i, i + 1] {
                    if lifted.contains(&j) {
                        path[j].y += short;
                        moved = true;
                    }
                }
            }
        }
        if !moved {
            break;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn road() -> Vec<Vec3> {
        vec![Vec3::new(0.0, 2.0, 0.0), Vec3::new(100.0, 2.0, 0.0), Vec3::new(200.0, 2.0, -50.0)]
    }

    #[test]
    fn it_comes_from_behind_along_the_road_and_goes_north() {
        let feet = Vec3::new(120.0, 0.0, 5.0);
        let p = flight(&road(), feet, Vec3::X);
        assert!(p.len() >= 6);
        // Before the player along the road, down to its height, then past them.
        let low: Vec<_> = p.iter().filter(|v| (v.y - 2.0).abs() < 0.01).collect();
        assert!(low.first().is_some_and(|v| v.x < feet.x - 50.0), "on the road behind: {:?}", low.first());
        assert!(low.last().is_some_and(|v| v.x > feet.x), "past them: {:?}", low.last());
        assert!(p[0].y > 30.0, "the approach is high");
        let end = p.last().unwrap();
        assert!(end.z < -350.0 && end.y > 100.0, "north and up: {end:?}");
        // Facing the other way, it comes the other way.
        let back = flight(&road(), feet, -Vec3::X);
        let low: Vec<_> = back.iter().filter(|v| (v.y - 2.0).abs() < 0.01).collect();
        assert!(low.first().is_some_and(|v| v.x > feet.x), "from behind again: {:?}", low.first());
    }

    #[test]
    fn the_road_is_measured_across_the_ground() {
        assert!((distance_from_road(&road(), Vec3::new(50.0, 40.0, 12.0)) - 12.0).abs() < 1e-3);
        assert!(night(23.0) && night(2.0) && !night(12.0));
    }
}
