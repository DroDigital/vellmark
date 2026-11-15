//! Photo mode's views through the real renderer, in the valley: as the eye sees it, as
//! text, and split by a divider, each side of which is the realistic frame or the text
//! frame exactly; the divider turned; and the exposure. The picture that explains the
//! game, saved for the evidence.

use vellmark_harness::capture::Offscreen;
use vellmark_harness::image::{contact_sheet, Image};
use vellmark_render::{Camera, FrameInput, ShaderLibrary, View};

fn frame(o: &mut Offscreen, close_reading: f32) -> Image {
    let views = [View::full(Camera::looking((-104.0, 50.6, 112.0).into(), 20f32.to_radians(), -6f32.to_radians()))];
    let input = FrameInput {
        views: &views,
        time_of_day: 7.0,
        seconds: 0.0,
        glyph_layer: true,
        close_reading,
        dive: 0.0,
        lamps: 1.0,
        tuning: Default::default(),
        reveal_mask: false,
        letters: &[],
        letters_step: 0,
        broken: &[],
        heat: &[],
        weather: Default::default(),
        wind_gust: None,
        relief: Default::default(),
        ui: None,
    };
    o.frame(&input)
}

/// The mean difference of two images over the pixels `keep` chooses, 0 to 255.
fn differ(a: &Image, b: &Image, keep: &dyn Fn(u32, u32) -> bool) -> f32 {
    let (mut sum, mut n) = (0u64, 0u64);
    for y in 0..a.height {
        for x in 0..a.width {
            if !keep(x, y) {
                continue;
            }
            let i = ((y * a.width + x) * 4) as usize;
            for c in 0..3 {
                sum += (a.rgba[i + c] as i32 - b.rgba[i + c] as i32).unsigned_abs() as u64;
            }
            n += 3;
        }
    }
    sum as f32 / n.max(1) as f32
}

#[test]
fn the_split_frame_is_the_realistic_frame_on_one_side_and_text_on_the_other() {
    let (w, h) = (480u32, 300u32);
    let mut o = match Offscreen::new(&Default::default(), &ShaderLibrary::embedded(), w, h, Some("scene.valley")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    let real = frame(&mut o, 0.0);
    let text = frame(&mut o, 1.0);
    assert!(differ(&real, &text, &|_, _| true) > 8.0, "the text frame differs from the realistic one");
    // Divided down the middle: text on the right.
    o.renderer.set_split(Some((0.0, 0.0)));
    let split = frame(&mut o, 0.0);
    let left = |x: u32, _: u32| x + 4 < w / 2;
    let right = |x: u32, _: u32| x > w / 2 + 4;
    assert!(differ(&split, &real, &left) < 1.5, "the left side as seen: {}", differ(&split, &real, &left));
    assert!(differ(&split, &text, &right) < 1.5, "the right side as text: {}", differ(&split, &text, &right));
    // Turned and moved: text below a line rising to the right.
    o.renderer.set_split(Some((60f32.to_radians(), 30.0)));
    let turned = frame(&mut o, 0.0);
    o.renderer.set_split(None);
    // A brighter exposure is brighter.
    o.renderer.set_exposure_scale(2.0);
    let bright = frame(&mut o, 0.0);
    o.renderer.set_exposure_scale(1.0);
    let mean = |i: &Image| i.rgba.chunks(4).map(|p| p[0] as f32 + p[1] as f32 + p[2] as f32).sum::<f32>() / (w * h) as f32;
    assert!(mean(&bright) > mean(&real) * 1.15, "exposure brightens the frame");
    let out = vellmark_harness::repo_root().join("build/captures/photo");
    std::fs::create_dir_all(&out).unwrap();
    contact_sheet(&[real, text, split, turned], 2).save(&out.join("sheet.png")).unwrap();
}
