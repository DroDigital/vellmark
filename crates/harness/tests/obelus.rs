//! Obelus, through the real renderer in the valley: its body, the obelus in black ink,
//! coming down the bridge road low, and what its line passes standing afterwards as black
//! rules with nothing behind them; and, flying high and far as at the vista, striking
//! nothing. Saved for the evidence.

use glam::Vec3;
use vellmark_content::{AssetId, AssetKind, Scene};
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_harness::image::{contact_sheet, Image};
use vellmark_render::{Camera, FrameInput, ShaderLibrary, View};
use vellmark_sim::redactor::Redactor;
use vellmark_sim::{PlayerInput, World, WorldConfig, STEP_HZ};

#[test]
fn obelus_strikes_a_black_rule_through_what_its_line_passes() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let scene = Scene::from_bytes(pack.get(AssetKind::Scene, AssetId::from_key("scene.valley")).unwrap()).unwrap();
    let terrain = std::sync::Arc::new(vellmark_world::Terrain::from_pack(&pack).unwrap());
    let mut w = World::new(WorldConfig { hours_per_second: 0.0, start_time_of_day: 15.0, ..Default::default() });
    let (solids, breakables) = vellmark_render::scene_rules(&scene);
    w.set_scene(solids, breakables);
    w.set_ground(terrain.clone());
    w.add_player(Vec3::new(112.0, 0.0, -86.0), 0.0);
    w.step(&[PlayerInput::default()]);
    // Down the bridge road past the mason's crates, low, from the west.
    let ground = |x: f32, z: f32| terrain.height_at(x, z).unwrap_or(7.0);
    let path: Vec<Vec3> = (0..=10).map(|k| {
        let (x, z) = (95.0 + k as f32 * 6.0, -93.5);
        Vec3::new(x, ground(x, z) + 1.4, z)
    }).collect();
    w.add_redactor(Redactor { id: 1, path, speed: 14.0, flown: 0.0, strike: Some((2.4, 2.2)), caught: Vec::new() });

    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), 480, 300, pack, Some("scene.valley")) {
        Ok(o) => Some(o),
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => None,
        Err(e) => panic!("{e}"),
    };
    let obelus = AssetId::from_key("scene.valley.obelus");
    let eye = Vec3::new(118.0, ground(118.0, -84.0) + 2.2, -84.0);
    let look = |at: Vec3| {
        let to = at - eye;
        Camera::looking(eye, to.x.atan2(-to.z), (to.y / Vec3::new(to.x, 0.0, to.z).length()).atan())
    };
    let mut frames: Vec<Image> = Vec::new();
    let mut seen_body = false;
    for step in 0..STEP_HZ * 6 {
        w.step(&[PlayerInput::default()]);
        let s = w.snapshot();
        let t = step as f32 / STEP_HZ as f32;
        let shoot = [1.6f32, 2.2, 5.5].iter().any(|at| (at - t).abs() < 0.5 / STEP_HZ as f32);
        if !shoot {
            continue;
        }
        let Some(o) = offscreen.as_mut() else { continue };
        match s.redactors.first() {
            Some(r) => {
                seen_body = true;
                o.renderer.scene_mut().place_kit(obelus, Some(vellmark_render::redactor_transform(r, t)));
            }
            None => o.renderer.scene_mut().place_kit(obelus, None),
        }
        o.renderer.set_struck(&s.struck);
        o.renderer.observe(&s, s.seconds as f32);
        // In flight, from up the road; after, close by the crate it struck.
        let crate_at = Vec3::new(125.5, ground(125.5, -93.0) + 0.4, -93.0);
        let camera = if s.redactors.is_empty() {
            let e = crate_at + Vec3::new(-2.6, 1.2, 3.2);
            let to = crate_at - e;
            Camera::looking(e, to.x.atan2(-to.z), (to.y / Vec3::new(to.x, 0.0, to.z).length()).atan())
        } else {
            look(Vec3::new(123.0, ground(123.0, -93.5) + 1.0, -93.5))
        };
        let views = [View::full(camera)];
        let input = FrameInput {
            views: &views,
            time_of_day: 15.0,
            seconds: s.seconds as f32,
            glyph_layer: true,
            close_reading: 0.0,
            dive: 0.0,
            lamps: 1.0,
            tuning: Default::default(),
            reveal_mask: false,
            letters: &[],
            letters_step: 0,
            broken: &s.broken,
            heat: &[],
            weather: Default::default(),
            wind_gust: None,
            relief: Default::default(),
            ui: None,
        };
        frames.push(o.frame(&input));
    }
    let struck: Vec<u64> = w.struck().iter().map(|s| s.id).collect();
    let crate_id = AssetId::from_key("scene.valley.mason_crate").0;
    assert!(struck.contains(&crate_id), "the mason's crate by the road is struck through: {struck:?}");
    assert!(!struck.contains(&AssetId::from_key("scene.valley.mended_bridge").0), "the bridge, far from its line, is not");
    if offscreen.is_some() {
        assert!(seen_body);
        let out = vellmark_harness::repo_root().join("build/captures/obelus");
        std::fs::create_dir_all(&out).unwrap();
        contact_sheet(&frames, 3).save(&out.join("sheet.png")).unwrap();
    }
}
