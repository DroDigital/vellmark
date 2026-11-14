//! The grey-box Inklings in their yards by the bridge road, as they are drawn: the plain
//! one gathering itself to strike, the LOCKED one beside it, the BURNING and COLD ones in
//! the other yard, and one spilt into a pool. Saved for the evidence.

use glam::Vec3;
use vellmark_content::{AssetId, AssetKind, Scene};
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_render::{Camera, FrameInput, ShaderLibrary, View};
use vellmark_sim::{PlayerInput, Snapshot, World, WorldConfig, STEP_HZ};

const HOUR: f32 = 11.0;

fn frame(offscreen: &mut Offscreen, s: &Snapshot, camera: Camera) -> vellmark_harness::image::Image {
    offscreen.renderer.observe(s, s.seconds as f32);
    let views = [View::full(camera)];
    offscreen.frame(&FrameInput {
        views: &views,
        time_of_day: HOUR,
        seconds: s.seconds as f32,
        glyph_layer: true,
        close_reading: 0.0,
        dive: 0.0,
        lamps: 0.0,
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
    })
}

fn looking_at(eye: Vec3, at: Vec3) -> Camera {
    let d = at - eye;
    Camera::looking(eye, d.x.atan2(-d.z), (d.y / Vec3::new(d.x, 0.0, d.z).length()).atan())
}

#[test]
fn the_inklings_are_drawn_in_their_yards() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let scene = Scene::from_bytes(pack.get(AssetKind::Scene, AssetId::from_key("scene.valley")).expect("the valley")).unwrap();
    assert_eq!(scene.creatures.len(), 4, "a plain Inkling and three annotated ones");
    let terrain = std::sync::Arc::new(vellmark_world::Terrain::from_pack(&pack).unwrap());
    let rules = vellmark_render::word_rules(&pack);
    let mut world = World::new(WorldConfig { hours_per_second: 0.0, start_time_of_day: HOUR, ..Default::default() });
    let (solids, breakables) = vellmark_render::scene_rules(&scene);
    world.set_scene(solids, breakables);
    world.set_ground(terrain.clone());
    world.set_targets(vellmark_render::scene_targets(&pack, "scene.valley", &rules));
    world.set_word_rules(rules);
    // The player at the gap in the first yard's fence, looking in.
    // The player at the gap in the south yard's fence, looking in, west.
    let feet = Vec3::new(57.0, terrain.height_at(57.0, -62.5).unwrap(), -62.5);
    world.add_player(feet, -std::f32::consts::FRAC_PI_2);
    for c in vellmark_render::scene_creatures(&scene) {
        world.add_creature(c);
    }
    let plain = AssetId::from_key("scene.valley.yard_inkling").0;
    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), 480, 270, pack, Some("scene.valley")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    // Turned away at first, so that the yard can be seen before it notices.
    let side = looking_at(Vec3::new(58.5, feet.y + 2.6, -54.5), Vec3::new(49.5, feet.y + 0.4, -63.0));
    let mut shots = vec![("yard", frame(&mut offscreen, &world.snapshot(), side))];
    // Waiting at the gap: it notices, closes, and gathers itself.
    let mut gathering = false;
    for k in 0..STEP_HZ * 8 {
        // A few steps into the yard, then wait.
        world.step(&[PlayerInput { move_y: if k < STEP_HZ / 2 { 0.6 } else { 0.0 }, ..Default::default() }]);
        if world.creatures().iter().any(|c| c.telegraph > 0.6) {
            gathering = true;
            break;
        }
    }
    assert!(gathering, "the Inkling gathers itself to strike");
    let c = world.creatures().iter().find(|c| c.telegraph > 0.6).unwrap().clone();
    shots.push(("gathering", frame(&mut offscreen, &world.snapshot(), looking_at(world.snapshot().players[0].feet + Vec3::new(1.4, 1.5, 0.8), c.centre()))));
    // The other yard.
    let other = looking_at(Vec3::new(56.0, feet.y + 1.9, -43.0), Vec3::new(47.5, feet.y + 0.3, -43.0));
    shots.push(("annotated", frame(&mut offscreen, &world.snapshot(), other)));
    // Spilt: dead, it runs out into a pool.
    world.creatures_mut().iter_mut().find(|c| c.id == plain).unwrap().health = 0.0;
    for _ in 0..STEP_HZ {
        world.step(&[PlayerInput::default()]);
    }
    let c = world.creatures().iter().find(|c| c.id == plain).unwrap().clone();
    shots.push(("spilt", frame(&mut offscreen, &world.snapshot(), looking_at(c.feet + Vec3::new(1.6, 1.4, 1.6), c.feet))));
    let out = vellmark_harness::repo_root().join("build/captures/inklings");
    std::fs::create_dir_all(&out).unwrap();
    for (name, img) in &shots {
        img.save(&out.join(format!("{name}.png"))).unwrap();
    }
}
