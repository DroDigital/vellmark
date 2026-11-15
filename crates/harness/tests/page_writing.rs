//! The Tithe Hill Stone writing as it is watched, in the real valley, drawn through the
//! renderer: from the path, its first entry standing and the next writing itself as the
//! reader walks up; at the stone, "The grass bends beneath no one." as their shadow
//! crosses it; then "Nobody is here." Saved for the evidence.

use glam::Vec3;
use vellmark_content::{AssetId, AssetKind, Scene};
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_harness::image::{contact_sheet, Image};
use vellmark_render::{Camera, FrameInput, ShaderLibrary, View};
use vellmark_sim::{PlayerInput, World, WorldConfig, STEP_HZ};

#[test]
fn the_tithe_hill_stone_writes_what_happens_as_it_happens() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let scene = Scene::from_bytes(pack.get(AssetKind::Scene, AssetId::from_key("scene.valley")).unwrap()).unwrap();
    let writing = vellmark_render::scene_page_writing(&pack, "scene.valley");
    let stone_id = AssetId::from_key("scene.valley.text.page.tithe_hill").0;
    let pages = writing.iter().find(|(id, _)| *id == stone_id).map(|(_, p)| p.clone()).expect("the Stone writes as it is watched");
    let mut w = World::new(WorldConfig { hours_per_second: 0.0, start_time_of_day: 7.5, ..Default::default() });
    let (solids, breakables) = vellmark_render::scene_rules(&scene);
    w.set_scene(solids, breakables);
    w.set_readables(vellmark_render::scene_readables(&pack, "scene.valley"));
    w.set_ground(std::sync::Arc::new(vellmark_world::Terrain::from_pack(&pack).unwrap()));
    let me = w.add_player(Vec3::new(-102.0, 0.0, 135.0), 0.0);
    let shown = |w: &World| w.snapshot().pages_written.iter().find(|(id, _)| *id == stone_id).map(|(_, n)| *n).unwrap();
    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), 960, 600, pack, Some("scene.valley")) {
        Ok(o) => Some(o),
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => None,
        Err(e) => panic!("{e}"),
    };
    let stone = Vec3::new(-102.45, 48.1, 105.5);
    let mut frames: Vec<Image> = Vec::new();
    let shoot = |w: &mut World, o: &mut Option<Offscreen>, frames: &mut Vec<Image>| {
        let s = w.snapshot();
        let to = stone - s.views[0].eye;
        w.step(&[PlayerInput { look_yaw: to.x.atan2(-to.z) - s.players[0].yaw, look_pitch: (to.y / Vec3::new(to.x, 0.0, to.z).length()).atan() - s.players[0].pitch, ..Default::default() }]);
        let Some(o) = o.as_mut() else { return };
        let s = w.snapshot();
        let n = shown(w);
        o.renderer.set_written(&o.gpu, &pages, n);
        let v = s.views[0];
        let views = [View::full(Camera::looking(v.eye, v.yaw, v.pitch))];
        let input = FrameInput {
            views: &views,
            time_of_day: 7.5,
            seconds: s.seconds as f32,
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
        frames.push(o.frame(&input));
    };
    let wait = |w: &mut World, seconds: f32| {
        for _ in 0..(seconds * STEP_HZ as f32) as u32 {
            w.step(&[PlayerInput::default()]);
        }
    };
    // Far down the path: only the first entry stands.
    wait(&mut w, 1.0);
    let first = shown(&w);
    // Walking up: within twenty metres the newest line begins to write itself.
    w.place_player(me, Vec3::new(-102.2, 0.0, 122.0), 0.0);
    wait(&mut w, 0.6);
    assert!(shown(&w) > first, "the newest line writes itself as the reader comes up");
    w.place_player(me, Vec3::new(-102.1, 0.0, 109.6), 0.0);
    shoot(&mut w, &mut offscreen, &mut frames);
    wait(&mut w, 4.0);
    let second = shown(&w);
    shoot(&mut w, &mut offscreen, &mut frames);
    // At the stone: the reader's shadow crosses it, and it writes of them.
    w.place_player(me, Vec3::new(-102.0, 0.0, 107.4), 0.0);
    wait(&mut w, 1.2);
    assert!(shown(&w) > second, "the grass bends beneath no one");
    shoot(&mut w, &mut offscreen, &mut frames);
    wait(&mut w, 12.0);
    shoot(&mut w, &mut offscreen, &mut frames);
    assert!(w.flags().contains(&vellmark_sim::reading::written_flag(stone_id)), "Nobody is here: the Stone has written all it writes");
    if !frames.is_empty() {
        let out = vellmark_harness::repo_root().join("build/captures/page_writing");
        std::fs::create_dir_all(&out).unwrap();
        contact_sheet(&frames, 2).save(&out.join("sheet.png")).unwrap();
    }
}
