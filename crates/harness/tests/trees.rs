//! Trees are drawn at the level their size on screen asks for, chosen in culling: the
//! finest close by, coarser levels farther off, and the far forms across the valley.

use vellmark_harness::capture::{Offscreen, baked_base_pack};
use vellmark_render::{Camera, FrameInput, ShaderLibrary, View};

fn drawn(offscreen: &mut Offscreen, eye: (f32, f32, f32), yaw: f32, pitch: f32) -> [u32; 4] {
    let views = [View::full(Camera::looking(
        eye.into(),
        yaw.to_radians(),
        pitch.to_radians(),
    ))];
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
        relief: Default::default(),
        ui: None,
    };
    offscreen.frame(&frame);
    offscreen
        .renderer
        .tree_levels_drawn_last_frame(&offscreen.gpu)
}

#[test]
fn trees_are_drawn_at_the_level_their_size_asks_for() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let mut offscreen = match Offscreen::with_pack(
        &Default::default(),
        &ShaderLibrary::embedded(),
        320,
        180,
        pack,
        Some("scene.valley"),
    ) {
        Ok(o) => o,
        Err(e)
            if e.contains("no usable graphics adapter")
                && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() =>
        {
            return eprintln!("skipped: {e}");
        }
        Err(e) => panic!("{e}"),
    };
    let ground = |o: &Offscreen, x: f32, z: f32| {
        o.terrain
            .as_ref()
            .and_then(|t| t.height_at(x, z))
            .expect("on the terrain")
    };
    // Among the orchard's rows: the nearest apples at their finest, the rest coarser.
    let g = ground(&offscreen, 132.0, -170.0);
    let orchard = drawn(&mut offscreen, (132.0, g + 1.7, -170.0), 70.0, 0.0);
    assert!(
        orchard[0] > 0,
        "the nearest trees are drawn at their finest: {orchard:?}"
    );
    assert!(
        orchard[1] + orchard[2] + orchard[3] > 0,
        "farther trees are coarser: {orchard:?}"
    );
    // From above Tithe Hill the north wood is hundreds of metres off: far forms only.
    let g = ground(&offscreen, -112.0, 128.0);
    let far = drawn(&mut offscreen, (-112.0, g + 12.0, 128.0), 0.0, 1.0);
    assert!(far[3] > 50, "the far wood is far forms: {far:?}");
    assert_eq!(
        far[0], 0,
        "nothing in the far wood is drawn at its finest: {far:?}"
    );
    // Looking at the sky draws no trees at all.
    let up = drawn(&mut offscreen, (-112.0, g + 12.0, 128.0), 0.0, 89.0);
    assert_eq!(up, [0; 4], "no trees in the sky");
}
