//! A Standing Page's learning spectacle, in the real valley at the Mended Bridge: reading
//! the Page long enough lifts `WHOLE` from the stone; the reader is held, looking at it;
//! then it goes out once to everything round the Page it would change, and the broken
//! bridge stands whole without a second word written. Drawn for the evidence.

use glam::Vec3;
use vellmark_content::{AssetId, AssetKind, Scene};
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_harness::image::{contact_sheet, Image};
use vellmark_render::{Camera, FrameInput, ShaderLibrary, View};
use vellmark_sim::{buttons, PlayerInput, World, WorldConfig, STEP_HZ};

#[test]
fn whole_lifts_from_the_page_and_mends_the_bridge_by_itself() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let scene = Scene::from_bytes(pack.get(AssetKind::Scene, AssetId::from_key("scene.valley")).unwrap()).unwrap();
    let rules = vellmark_render::word_rules(&pack);
    let mut w = World::new(WorldConfig { hours_per_second: 0.0, start_time_of_day: 17.6, ..Default::default() });
    let (solids, breakables) = vellmark_render::scene_rules(&scene);
    w.set_scene(solids, breakables);
    w.set_readables(vellmark_render::scene_readables(&pack, "scene.valley"));
    w.set_targets(vellmark_render::scene_targets(&pack, "scene.valley", &rules));
    w.set_word_rules(rules);
    w.set_ground(std::sync::Arc::new(vellmark_world::Terrain::from_pack(&pack).unwrap()));
    // A little aside of the Page, so that the bridge shows past it.
    let me = w.add_player(Vec3::new(127.3, 0.0, -96.4), 0.0);
    w.step(&[PlayerInput::default()]);
    let bridge = AssetId::from_key("scene.valley.mended_bridge").0;
    assert!(w.snapshot().broken.contains(&bridge), "the bridge starts broken");
    // Look at the Page and read it.
    let page = Vec3::new(128.05, 7.6, -98.3);
    let s = w.snapshot();
    let to = page - s.views[0].eye;
    w.step(&[PlayerInput { look_yaw: to.x.atan2(-to.z) - s.players[0].yaw, look_pitch: (to.y / Vec3::new(to.x, 0.0, to.z).length()).atan() - s.players[0].pitch, ..Default::default() }]);
    let mut began = None;
    for k in 0..STEP_HZ * 3 {
        w.step(&[PlayerInput { buttons: buttons::READ, ..Default::default() }]);
        if began.is_none() && !w.snapshot().learning.is_empty() {
            began = Some(k);
            break;
        }
    }
    assert!(began.is_some(), "the Page gives its word");
    let first = w.snapshot().learning[0].clone();
    assert_eq!(first.word, "WHOLE");

    let (width, height) = (480u32, 300u32);
    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), width, height, pack, Some("scene.valley")) {
        Ok(o) => Some(o),
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => None,
        Err(e) => panic!("{e}"),
    };
    let mut frames: Vec<Image> = Vec::new();
    let shots = [0.5f32, 1.4, 2.4, 6.0, 9.0, 12.0];
    let start = w.snapshot().players[0].feet;
    let mut mended_at = None;
    let mut rose = 0.0f32;
    // The bridge knits back over some seconds once the word has landed.
    for step in 0..STEP_HZ * 16 {
        // Trying to walk off while it is given: held still.
        let trying = step < STEP_HZ * 5;
        // Once free, step beside the stone to watch the bridge knit.
        if step == STEP_HZ * 5 + 30 {
            w.place_player(me, Vec3::new(129.9, 0.0, -97.6), 0.0);
            let s = w.snapshot();
            let to = Vec3::new(131.4, 5.5, -106.8) - s.views[0].eye;
            w.step(&[PlayerInput { look_yaw: to.x.atan2(-to.z) - s.players[0].yaw, look_pitch: (to.y / Vec3::new(to.x, 0.0, to.z).length()).atan() - s.players[0].pitch, ..Default::default() }]);
        }
        w.step(&[if trying { PlayerInput { move_y: 1.0, look_yaw: 0.05, ..Default::default() } } else { PlayerInput::default() }]);
        let s = w.snapshot();
        let age = step as f32 / STEP_HZ as f32;
        if let Some(l) = s.learning.first().filter(|l| !l.released) {
            rose = rose.max(l.at.distance(l.page));
        }
        if age < 4.8 {
            let f = s.players[0].feet;
            assert!(Vec3::new(f.x - start.x, 0.0, f.z - start.z).length() < 0.05, "held while the word is given, at {age} s");
        }
        if mended_at.is_none() && !s.broken.contains(&bridge) {
            mended_at = Some(age);
        }
        if let (Some(o), Some(_)) = (offscreen.as_mut(), shots.iter().find(|t| (**t - age).abs() < 0.5 / STEP_HZ as f32)) {
            let v = s.views[0];
            let views = [View::full(Camera::looking(v.eye, v.yaw, v.pitch))];
            let input = FrameInput {
                views: &views,
                time_of_day: 17.6,
                seconds: s.seconds as f32,
                glyph_layer: true,
                close_reading: 0.0,
                dive: 0.0,
                lamps: 1.0,
                tuning: Default::default(),
                reveal_mask: false,
                letters: &s.letters,
                letters_step: s.letters_step,
                broken: &s.broken,
                heat: &s.heat,
                weather: Default::default(),
                wind_gust: None,
                relief: Default::default(),
                ui: None,
            };
            o.renderer.observe(&s, s.seconds as f32);
            frames.push(o.frame(&input));
        }
    }
    assert!(rose > 0.5, "the word lifts from the stone: {rose} m");
    let mended_at = mended_at.expect("the bridge stands whole with no word written by hand");
    assert!(mended_at > 2.5, "only after the view has held on the word: {mended_at} s");
    assert!(w.snapshot().players[0].holding[1] == vellmark_sim::casting::Holding::Word(w.word_rules().word("WHOLE").unwrap()));
    if !frames.is_empty() {
        let out = vellmark_harness::repo_root().join("build/captures/spectacle");
        std::fs::create_dir_all(&out).unwrap();
        contact_sheet(&frames, 3).save(&out.join("sheet.png")).unwrap();
    }
}
