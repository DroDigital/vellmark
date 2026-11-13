//! Reading. Holding Read with a text in focus reads it: the text, and those of its
//! hidden lines the reader's Perception reaches. Holding Read with nothing in focus is
//! close reading: the whole view becomes text, and in solo play time slows to a third.

use glam::Vec3;

/// A text is in focus within this distance of the eye.
pub const READ_REACH: f32 = 2.6;
/// Close reading slows the world to this share of its pace, in solo play.
pub const CLOSE_READING_PACE: f32 = 1.0 / 3.0;

/// A text that can be read, where it is set.
#[derive(Clone, Debug, PartialEq)]
pub struct ReadableText {
    pub id: u64,
    /// Its string's ID in the string tables.
    pub text: u64,
    pub min: Vec3,
    pub max: Vec3,
    /// Hidden lines: each string's ID and the Perception it takes.
    pub hidden: Vec<(u64, u32)>,
}

/// What a player is doing with Read.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum Reading {
    #[default]
    None,
    /// Reading a text: its index among the world's readables.
    Text(usize),
    /// The whole view as text.
    Close,
}

/// What a player is reading, as drawn.
#[derive(Clone, Debug, Default, PartialEq)]
pub enum ReadingSnap {
    #[default]
    None,
    /// The text's string, and the hidden lines this reader sees.
    Text { id: u64, text: u64, hidden: Vec<u64> },
    Close,
}

/// Where a ray from `from` along `dir` (of length 1) enters a box, if it does within `reach`.
pub fn ray_box(from: Vec3, dir: Vec3, min: Vec3, max: Vec3, reach: f32) -> Option<f32> {
    let mut near = 0.0f32;
    let mut far = reach;
    for i in 0..3 {
        let d = dir[i];
        if d.abs() < 1.0e-8 {
            if from[i] < min[i] || from[i] > max[i] {
                return None;
            }
            continue;
        }
        let (mut a, mut b) = ((min[i] - from[i]) / d, (max[i] - from[i]) / d);
        if a > b {
            std::mem::swap(&mut a, &mut b);
        }
        near = near.max(a);
        far = far.min(b);
        if near > far {
            return None;
        }
    }
    Some(near)
}

/// The text the eye is on: the nearest whose box the gaze meets within reach, and that
/// nothing solid stands in front of.
pub fn focus(eye: Vec3, gaze: Vec3, texts: &[ReadableText], statics: Option<&crate::character::Statics>) -> Option<usize> {
    let mut best: Option<(f32, usize)> = None;
    for (i, t) in texts.iter().enumerate() {
        // A little room about the surface, so that the eye need not hit it exactly.
        let pad = Vec3::splat(0.05);
        if let Some(d) = ray_box(eye, gaze, t.min - pad, t.max + pad, READ_REACH) {
            if best.is_none_or(|(b, _)| d < b) {
                best = Some((d, i));
            }
        }
    }
    let (d, i) = best?;
    // Something nearer than the text's surface, other than the text's own wall, hides it.
    if let Some((hit, _)) = statics.and_then(|s| s.ray(eye, gaze, d)) {
        if hit < d - 0.1 {
            return None;
        }
    }
    Some(i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn wall() -> ReadableText {
        ReadableText { id: 1, text: 2, min: Vec3::new(-1.0, 0.0, -3.0), max: Vec3::new(1.0, 2.5, -2.8), hidden: vec![(3, 2)] }
    }

    #[test]
    fn the_text_ahead_within_reach_is_in_focus() {
        let texts = [wall()];
        let eye = Vec3::new(0.0, 1.6, -0.5);
        assert_eq!(focus(eye, -Vec3::Z, &texts, None), Some(0));
        // Looking away, or from too far, nothing is.
        assert_eq!(focus(eye, Vec3::Z, &texts, None), None);
        assert_eq!(focus(Vec3::new(0.0, 1.6, 2.0), -Vec3::Z, &texts, None), None);
    }
}
