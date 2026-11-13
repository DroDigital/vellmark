//! A word written in the yard, end to end: the real word table, the door's Inscription,
//! the crosshair's preview, a tap of the off hand, the word's flight and landing, and the
//! door swung open. Saved as evidence.

use glam::{Mat4, Vec3};
use vellmark_content::{AssetId, AssetKind, Scene, StringTable};
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_render::{Camera, FrameInput, LoadedAssets, ShaderLibrary, View};
use vellmark_sim::emendation::{Landing, Preview};
use vellmark_sim::{buttons, PlayerInput, World, WorldConfig, STEP_HZ};
use vellmark_ui::hud::Hud;
use vellmark_ui::Canvas;

#[test]
fn open_is_written_on_the_yards_door_and_it_swings_open() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    let assets = LoadedAssets::from_pack(&pack).unwrap();
    let scene = Scene::from_bytes(pack.get(AssetKind::Scene, AssetId::from_key("scene.yard")).unwrap()).unwrap();
    let rules = vellmark_render::word_rules(&pack);
    assert_eq!(rules.words.len(), 7, "the seven words");
    let targets = vellmark_render::scene_targets(&pack, "scene.yard", &rules);
    let door_id = AssetId::from_key("scene.yard.door");
    let door = targets.iter().find(|t| t.id == door_id.0).expect("the door is inscribed").clone();

    // A player in the yard, looking at the door, OPEN in the off hand.
    let mut world = World::new(WorldConfig { hours_per_second: 0.0, start_time_of_day: 15.0, ..Default::default() });
    let eye = Vec3::new(door.centre.x, 1.68, door.centre.z + 4.0);
    let id = world.add_player(eye - Vec3::Y * 1.68, 0.0);
    world.set_word_rules(rules.clone());
    world.set_targets(targets);
    world.hold_word(id, 1, "OPEN");
    let pitch = ((door.centre.y - eye.y) / 4.0).atan();
    world.step(&[PlayerInput { look_pitch: pitch, ..Default::default() }]);
    let s = world.snapshot();
    assert_eq!(s.players[0].preview[1], Some((door_id.0, Preview::Result(vellmark_content::StringId::from_key("preview.open").0))));

    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), 960, 540, pack, Some("scene.yard")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    let out = vellmark_harness::repo_root().join("build/captures/casting");
    std::fs::create_dir_all(&out).unwrap();
    let mut shoot = |offscreen: &mut Offscreen, world: &World, name: &str| {
        let s = world.snapshot();
        let mut canvas = Canvas::new();
        Hud { atlas: &assets.glyphs, strings: &strings, rules: &rules, size: (960, 540), density: 1.0 }.draw(&mut canvas, &s.players[0]);
        let views = [View::full(Camera::looking(s.views[0].eye, s.views[0].yaw, s.views[0].pitch))];
        let input = FrameInput {
            views: &views,
            time_of_day: 15.0,
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
            heat: &s.heat,
            weather: Default::default(),
            wind_gust: None,
            relief: Default::default(),
            ui: Some(&canvas),
        };
        offscreen.frame(&input).save(&out.join(format!("{name}.png"))).unwrap();
    };
    shoot(&mut offscreen, &world, "before");

    // A tap of the off hand: the word flies and lands.
    for _ in 0..3 {
        world.step(&[PlayerInput { buttons: buttons::ATTACK_OFF, ..Default::default() }]);
    }
    let mut landing = None;
    for _ in 0..STEP_HZ {
        world.step(&[PlayerInput::default()]);
        if let Some(l) = world.snapshot().landings.first() {
            landing = Some(l.landing.clone());
        }
    }
    assert!(matches!(landing, Some(Landing::Took { ref effect, .. }) if effect == "open"), "{landing:?}");
    let s = world.snapshot();
    let door_now = s.targets.iter().find(|t| t.id == door_id.0).unwrap();
    assert_eq!(door_now.words, ["OPEN"]);
    // The door swings on its hinge, as the game swings it.
    let object = scene.objects.iter().find(|o| o.key == door_id).unwrap();
    let t = object.transform;
    let place = Mat4::from_cols_array(&[t[0], t[4], t[8], 0.0, t[1], t[5], t[9], 0.0, t[2], t[6], t[10], 0.0, t[3], t[7], t[11], 1.0]);
    offscreen.renderer.scene_mut().set_transform(door_id, place * Mat4::from_rotation_y(-95f32.to_radians()));
    shoot(&mut offscreen, &world, "after");
}
