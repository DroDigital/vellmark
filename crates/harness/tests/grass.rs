//! Grass grows where the ground says it does, near the eye, and is culled like
//! everything else: patches in view are drawn, nearer ones with more blades, and none
//! are drawn where there is no grass within reach.

use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_render::{Camera, FrameInput, ShaderLibrary, View};

fn drawn(offscreen: &mut Offscreen, eye: (f32, f32, f32), yaw: f32, pitch: f32) -> [u32; 4] {
    let views = [View::full(Camera::looking(eye.into(), yaw.to_radians(), pitch.to_radians()))];
    let frame = FrameInput {
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
        weather: Default::default(),
        wind_gust: None,
        ui: None,
    };
    offscreen.frame(&frame);
    offscreen.renderer.grass_drawn_last_frame(&offscreen.gpu)
}

#[test]
fn grass_is_drawn_near_the_eye_in_levels_and_culled_out_of_view() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), 320, 180, pack, Some("scene.valley")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => {
            return eprintln!("skipped: {e}");
        }
        Err(e) => panic!("{e}"),
    };
    // On Tithe Hill, looking north over the pasture and the barley.
    let (x, z) = (-104.0, 112.0);
    let ground = offscreen.terrain.as_ref().and_then(|t| t.height_at(x, z)).expect("the hill is on the terrain");
    let eye = (x, ground + 1.7, z);
    let ahead = drawn(&mut offscreen, eye, 22.0, -5.0);
    assert!(ahead[0] > 0, "patches at the eye's feet are drawn with the most blades: {ahead:?}");
    assert!(ahead[1] + ahead[2] + ahead[3] > ahead[0], "most patches in view are farther off, with fewer blades: {ahead:?}");
    // Looking the other way draws other patches, and about as many.
    let behind = drawn(&mut offscreen, eye, 202.0, -5.0);
    assert!(behind.iter().sum::<u32>() > 0, "{behind:?}");
    // Looking at the sky: only the patches around the feet can still touch the view.
    let up = drawn(&mut offscreen, eye, 22.0, 85.0);
    assert!(up.iter().sum::<u32>() * 4 < ahead.iter().sum::<u32>(), "looking up draws far less: {up:?} against {ahead:?}");
    // High above the valley no grass is within reach.
    let above = drawn(&mut offscreen, (x, ground + 400.0, z), 22.0, -60.0);
    assert_eq!(above, [0; 4], "no grass is drawn from far above");
}
