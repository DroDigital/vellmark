//! The five parts of a word's magic, as they are drawn: a charge gathering at the hand,
//! the word in flight as a moving light, its impact, the rewrite wave running over the
//! door, and the lasting state. Saved as a contact sheet for the evidence.

use glam::Vec3;
use vellmark_content::{AssetId, AssetKind, StringTable};
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_render::{Camera, FrameInput, LoadedAssets, ShaderLibrary, View};
use vellmark_sim::{buttons, PlayerInput, Snapshot, World, WorldConfig};
use vellmark_ui::hud::Hud;
use vellmark_ui::Canvas;

fn frame(offscreen: &mut Offscreen, s: &Snapshot, seconds: f32, hud: &Hud<'_>) -> vellmark_harness::image::Image {
    offscreen.renderer.observe(s, seconds);
    let mut canvas = Canvas::new();
    hud.draw(&mut canvas, &s.players[0]);
    let views = [View::full(Camera::looking(s.views[0].eye, s.views[0].yaw, s.views[0].pitch))];
    let input = FrameInput {
        views: &views,
        time_of_day: 18.6,
        seconds,
        glyph_layer: true,
        close_reading: 0.0,
        dive: 0.0,
        lamps: 1.0,
        tuning: Default::default(),
        reveal_mask: false,
        letters: &[],
        letters_step: 0,
        broken: &[],
        heat: &s.heat,
        weather: Default::default(),
        wind_gust: None,
        relief: Default::default(),
        ui: Some(&canvas),
    };
    offscreen.frame(&input)
}

#[test]
fn a_words_magic_is_drawn_in_its_five_parts() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    let assets = LoadedAssets::from_pack(&pack).unwrap();
    let rules = vellmark_render::word_rules(&pack);
    let targets = vellmark_render::scene_targets(&pack, "scene.yard", &rules);
    let door = targets.iter().find(|t| t.id == AssetId::from_key("scene.yard.door").0).unwrap().clone();
    let mut world = World::new(WorldConfig { hours_per_second: 0.0, start_time_of_day: 18.6, ..Default::default() });
    let eye = Vec3::new(door.centre.x + 0.8, 1.68, door.centre.z + 5.0);
    let id = world.add_player(eye - Vec3::Y * 1.68, -0.12);
    world.set_word_rules(rules.clone());
    world.set_targets(targets);
    world.hold_word(id, 1, "BURNING");
    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), 640, 360, pack, Some("scene.yard")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    let hud = Hud { atlas: &assets.glyphs, strings: &strings, rules: &rules, size: (640, 360), density: 1.0 };
    let dt = 1.0 / 60.0;
    let mut seconds = 0.0f32;
    let mut step = |world: &mut World, buttons: u32, seconds: &mut f32| {
        world.step(&[PlayerInput { buttons, ..Default::default() }]);
        *seconds += dt;
    };
    let pitch = ((door.centre.y - eye.y) / 5.0).atan();
    world.step(&[PlayerInput { look_pitch: pitch, ..Default::default() }]);
    // 1. The charge, a second in.
    for _ in 0..60 {
        step(&mut world, buttons::ATTACK_OFF, &mut seconds);
    }
    let mut shots = vec![("charge", frame(&mut offscreen, &world.snapshot(), seconds, &hud))];
    assert!(offscreen.renderer.magic_active().1 >= 1, "the charge has an emitter");
    // 2. The flight: let go, and a few steps on.
    for _ in 0..8 {
        step(&mut world, 0, &mut seconds);
    }
    assert!(!world.snapshot().flights.is_empty(), "the word is in flight");
    shots.push(("flight", frame(&mut offscreen, &world.snapshot(), seconds, &hud)));
    // 3 and 4. The impact and the wave.
    let mut landed = false;
    for _ in 0..60 {
        step(&mut world, 0, &mut seconds);
        offscreen.renderer.observe(&world.snapshot(), seconds);
        if !world.snapshot().landings.is_empty() {
            landed = true;
            break;
        }
    }
    assert!(landed, "the word lands");
    for _ in 0..6 {
        step(&mut world, 0, &mut seconds);
    }
    shots.push(("impact", frame(&mut offscreen, &world.snapshot(), seconds, &hud)));
    assert!(offscreen.renderer.magic_active().0 >= 1, "a rewrite wave is running");
    for _ in 0..24 {
        step(&mut world, 0, &mut seconds);
    }
    shots.push(("wave", frame(&mut offscreen, &world.snapshot(), seconds, &hud)));
    // 5. The lasting state, the wave gone.
    for _ in 0..240 {
        step(&mut world, 0, &mut seconds);
    }
    shots.push(("lasting", frame(&mut offscreen, &world.snapshot(), seconds, &hud)));
    assert_eq!(offscreen.renderer.magic_active().0, 0, "the wave has sunk back");

    let out = vellmark_harness::repo_root().join("build/captures/magic");
    std::fs::create_dir_all(&out).unwrap();
    for (name, img) in &shots {
        img.save(&out.join(format!("{name}.png"))).unwrap();
    }
    // The charge and the flight bring light to the scene: brighter than the lasting frame
    // somewhere.
    let brightest = |img: &vellmark_harness::image::Image| img.rgba.chunks(4).map(|p| p[0] as u32 + p[1] as u32 + p[2] as u32).max().unwrap();
    assert!(brightest(&shots[1].1) > 500, "the word in flight is bright");
}
