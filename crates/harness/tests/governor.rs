//! With everything the governor may give up given up, the frame is still whole: the
//! scene, drawn at 85%, is stretched to fill the screen, and it is the same place as at
//! full quality, a little softer.

use vellmark_harness::capture::Offscreen;
use vellmark_render::{Camera, FrameInput, Relief, ShaderLibrary, View};

fn frame(offscreen: &mut Offscreen, relief: Relief) -> vellmark_harness::image::Image {
    let ground = offscreen.terrain.as_ref().and_then(|t| t.height_at(-104.0, 112.0)).unwrap_or(0.0);
    let views = [View::full(Camera::looking((-104.0, ground + 1.7, 112.0).into(), 22f32.to_radians(), -0.1))];
    let input = FrameInput {
        views: &views,
        time_of_day: 10.0,
        seconds: 0.0,
        glyph_layer: true,
        close_reading: 0.0,
        dive: 0.0,
        lamps: 1.0,
        tuning: Default::default(),
        reveal_mask: false,
        letters: &[],
        letters_step: 0,
        broken: &[],
        heat: &[],
        weather: vellmark_sim::Weather { rain: 0.6, wetness: 1.0, ..Default::default() },
        wind_gust: None,
        relief,
        ui: None,
    };
    offscreen.frame(&input)
}

#[test]
fn everything_given_up_still_draws_the_same_place() {
    let mut offscreen = match Offscreen::new(&Default::default(), &ShaderLibrary::embedded(), 320, 180, Some("scene.valley")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => {
            return eprintln!("skipped: {e}");
        }
        Err(e) => panic!("{e}"),
    };
    let full = frame(&mut offscreen, Relief::default());
    let least = frame(&mut offscreen, Relief { levels: [1.0; 6] });
    assert!((least.width, least.height) == (full.width, full.height));
    // The bottom right corner is drawn, not left empty by the smaller scene.
    let corner = |img: &vellmark_harness::image::Image| {
        let i = ((img.height - 2) * img.width + img.width - 2) as usize * 4;
        img.rgba[i] as u32 + img.rgba[i + 1] as u32 + img.rgba[i + 2] as u32
    };
    assert!(corner(&least) > 30, "the stretched scene reaches the corner");
    // The same place: on average the two frames are close.
    let diff: f64 = full.rgba.iter().zip(&least.rgba).map(|(a, b)| (*a as f64 - *b as f64).abs()).sum::<f64>() / full.rgba.len() as f64;
    assert!(diff < 12.0, "a mean difference of {diff:.2} levels");
}
