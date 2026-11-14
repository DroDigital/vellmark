//! The interface's letters, drawn through the real renderer.

use vellmark_content::Face;
use vellmark_harness::capture::{baked_base_pack, glyph_atlas, Offscreen};
use vellmark_harness::image::Image;
use vellmark_harness::refs::{self, RefMode};
use vellmark_harness::repo_root;
use vellmark_render::{Camera, FrameInput, ShaderLibrary, View};
use vellmark_ui::{dev_text, Canvas, TextStyle};

fn ink(image: &Image, rect: (u32, u32, u32, u32)) -> f32 {
    let (x0, y0, w, h) = rect;
    let mut dark = 0;
    for y in y0..y0 + h {
        for x in x0..x0 + w {
            let p = &image.rgba[((y * image.width + x) * 4) as usize..][..3];
            if p.iter().all(|&c| c < 90) {
                dark += 1;
            }
        }
    }
    dark as f32 / (w * h) as f32
}

#[test]
fn both_typefaces_draw_at_every_size() {
    let atlas = glyph_atlas(&baked_base_pack().unwrap()).unwrap();
    let mut offscreen = match Offscreen::new(&Default::default(), &ShaderLibrary::embedded(), 640, 360, None) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => {
            return eprintln!("skipped: {e}");
        }
        Err(e) => panic!("{e}"),
    };

    let mut canvas = Canvas::new();
    canvas.rect([0.0, 0.0, 640.0, 360.0], [0.93, 0.90, 0.82, 1.0]);
    let black = [0.01, 0.01, 0.012, 1.0];
    let mut y = 16.0;
    for (face, sizes) in [(Face::Small, [8.0, 12.0, 16.0, 32.0]), (Face::Large, [10.0, 16.0, 28.0, 56.0])] {
        for size in sizes {
            let style = TextStyle { face, size, colour: black };
            y += Canvas::line_height(&atlas, &style) * 0.85;
            let width = canvas.text(&atlas, 12.0, y, &style, &dev_text("The quick brown fox: 0123456789 {LOCKED} -> [OPEN]"));
            assert!(width > size * 10.0);
        }
    }
    // A plain rectangle of known size, to check that pixels land where they are asked.
    canvas.rect([600.0, 320.0, 30.0, 30.0], black);

    let views = [View::full(Camera::looking(glam::Vec3::new(0.0, 1.7, 0.0), 0.0, 0.0))];
    let frame = FrameInput { views: &views, time_of_day: 12.0, seconds: 0.0, glyph_layer: true, close_reading: 0.0, dive: 0.0, lamps: 1.0, tuning: Default::default(), reveal_mask: false, letters: &[], letters_step: 0, broken: &[], heat: &[], weather: Default::default(), wind_gust: None, relief: Default::default(), ui: Some(&canvas) };
    let image = offscreen.frame(&frame);
    image.save(&repo_root().join("build/captures/interface/typefaces.png")).unwrap();

    assert!(ink(&image, (600, 320, 30, 30)) > 0.99, "the rectangle is solid");
    assert!(ink(&image, (560, 320, 30, 30)) < 0.01, "and nothing spills beside it");
    let text_ink = ink(&image, (12, 10, 500, 330));
    assert!(text_ink > 0.03 && text_ink < 0.35, "text covers a believable share of the page: {text_ink}");

    if offscreen.gpu.describe().contains("llvmpipe") {
        let path = refs::reference_path(&repo_root().join("refs"), "interface", "typefaces");
        if std::env::var_os("VELLMARK_ACCEPT_REFS").is_some() {
            refs::accept(&path, &image).unwrap();
        }
        let result = refs::check(&path, &image, Default::default());
        assert!(result.passed(), "interface/typefaces moved from its reference: {result:?}");
    }
    let _ = RefMode::Ignore;
}
