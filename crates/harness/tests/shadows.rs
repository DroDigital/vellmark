//! Shadows are drawn only when they must be: the near cascades every frame, the far one
//! once and kept until the light or the eye has moved enough or a still thing changes,
//! and a lantern's cube once and kept until something near it changes.

use vellmark_content::AssetId;
use vellmark_harness::capture::Offscreen;
use vellmark_render::{Camera, FrameInput, ShaderLibrary, View};

fn frame(offscreen: &mut Offscreen, eye: (f32, f32, f32), hour: f32, broken: &[u64]) -> (usize, usize) {
    let views = [View::full(Camera::looking(eye.into(), 0.0, 0.0))];
    let input = FrameInput {
        views: &views,
        time_of_day: hour,
        seconds: 0.0,
        glyph_layer: true,
        close_reading: 0.0,
        dive: 0.0,
        lamps: 1.0,
        tuning: Default::default(),
        reveal_mask: false,
        letters: &[],
        letters_step: 0,
        broken,
        heat: &[],
        weather: Default::default(),
        wind_gust: None,
        relief: Default::default(),
        ui: None,
    };
    offscreen.frame(&input);
    offscreen.renderer.shadows_drawn_last_frame()
}

#[test]
fn shadows_are_kept_until_something_that_matters_changes() {
    let mut offscreen = match Offscreen::new(&Default::default(), &ShaderLibrary::embedded(), 320, 180, Some("scene.yard")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => {
            return eprintln!("skipped: {e}");
        }
        Err(e) => panic!("{e}"),
    };
    let eye = (0.0, 1.68, 4.0);
    // The first frame draws everything: three cascades and the lantern's cube.
    assert_eq!(frame(&mut offscreen, eye, 15.0, &[]), (3, 1));
    // Nothing has changed: only the two near cascades are drawn again.
    assert_eq!(frame(&mut offscreen, eye, 15.0, &[]), (2, 0), "the far cascade and the cube are kept");
    // A step to the side is not enough to move the far cascade.
    assert_eq!(frame(&mut offscreen, (0.5, 1.68, 4.0), 15.0, &[]), (2, 0));
    // The sun moving by more than a quarter of a degree redraws it; the cube stays.
    assert_eq!(frame(&mut offscreen, (0.5, 1.68, 4.0), 15.2, &[]), (3, 0));
    // Breaking the door changes what casts shadows: the far cascade and the cube redraw.
    let door = AssetId::from_key("scene.yard.door").0;
    assert_eq!(frame(&mut offscreen, (0.5, 1.68, 4.0), 15.2, &[door]), (3, 1));
    assert_eq!(frame(&mut offscreen, (0.5, 1.68, 4.0), 15.2, &[door]), (2, 0));
    // Walking forty metres crosses into another cell of the far cascade's grid.
    assert_eq!(frame(&mut offscreen, (0.5, 1.68, 44.0), 15.2, &[door]), (3, 0));
}
