//! The shimmer check. A slow camera move is captured frame by frame, and the part of
//! each pixel's change that is not smooth motion is measured.
//!
//! For a pixel that changes steadily, the middle frame is the average of its
//! neighbours in time. Crawling edges, moire, and letters that swim or pop break that:
//! the pixel flickers against its own trend. The measure is the size of that flicker.

use crate::image::Image;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Shimmer {
    /// Mean flicker over all pixels and frames, in luminance, 0 to 1.
    pub mean: f32,
    /// The flicker that 99% of pixels stay below.
    pub p99: f32,
    /// Fraction of pixels that flicker by more than 4% of full brightness.
    pub flickering: f32,
}

pub fn measure(frames: &[Image]) -> Result<Shimmer, String> {
    if frames.len() < 3 {
        return Err("the shimmer check needs at least three frames".into());
    }
    let luminance: Vec<Vec<f32>> = frames.iter().map(Image::luminance).collect();
    let pixels = luminance[0].len();
    if luminance.iter().any(|l| l.len() != pixels) {
        return Err("frames differ in size".into());
    }
    let mut flicker = Vec::with_capacity(pixels * (frames.len() - 2));
    for t in 1..frames.len() - 1 {
        let (before, now, after) = (&luminance[t - 1], &luminance[t], &luminance[t + 1]);
        for i in 0..pixels {
            flicker.push((now[i] - 0.5 * (before[i] + after[i])).abs());
        }
    }
    let mean = flicker.iter().sum::<f32>() / flicker.len() as f32;
    let flickering = flicker.iter().filter(|&&f| f > 0.04).count() as f32 / flicker.len() as f32;
    let k = ((flicker.len() as f32 * 0.99) as usize).min(flicker.len() - 1);
    let (_, p99, _) = flicker.select_nth_unstable_by(k, |a, b| a.total_cmp(b));
    Ok(Shimmer {
        mean,
        p99: *p99,
        flickering,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn grey(v: u8) -> Image {
        Image::filled(8, 8, [v, v, v, 255])
    }

    #[test]
    fn steady_change_does_not_shimmer() {
        let s = measure(&[grey(10), grey(20), grey(30), grey(40)]).unwrap();
        assert!(s.mean < 1e-6 && s.p99 < 1e-6 && s.flickering == 0.0);
    }

    #[test]
    fn flicker_is_measured() {
        let s = measure(&[grey(10), grey(110), grey(10), grey(110)]).unwrap();
        assert!(s.mean > 0.3);
        assert_eq!(s.flickering, 1.0);
    }

    #[test]
    fn too_few_frames() {
        assert!(measure(&[grey(1), grey(2)]).is_err());
    }
}
