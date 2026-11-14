//! The Mended Bridge in the real valley, as it is drawn: lying in letters in the Quill,
//! knitting back from the writer's end once `WHOLE` is written on it, and standing whole.
//! Seen from the bank, beside it. Saved for the evidence.

use glam::Vec3;
use vellmark_content::{AssetId, AssetKind, Scene};
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_render::{Camera, FrameInput, ShaderLibrary, View};
use vellmark_sim::{buttons, PlayerInput, Snapshot, World, WorldConfig, STEP_HZ};

const HOUR: f32 = 10.5;

fn frame(offscreen: &mut Offscreen, s: &Snapshot, seconds: f32, camera: Camera) -> vellmark_harness::image::Image {
    offscreen.renderer.observe(s, seconds);
    let views = [View::full(camera)];
    let input = FrameInput {
        views: &views,
        time_of_day: HOUR,
        seconds,
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
    };
    offscreen.frame(&input)
}

#[test]
fn the_bridge_is_drawn_broken_knitting_and_whole() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let scene = Scene::from_bytes(pack.get(AssetKind::Scene, AssetId::from_key("scene.valley")).expect("the valley")).unwrap();
    let terrain = std::sync::Arc::new(vellmark_world::Terrain::from_pack(&pack).unwrap());
    let rules = vellmark_render::word_rules(&pack);
    let targets = vellmark_render::scene_targets(&pack, "scene.valley", &rules);
    let bridge = AssetId::from_key("scene.valley.mended_bridge").0;
    let centre = targets.iter().find(|t| t.id == bridge).expect("the bridge carries an Inscription").centre;

    let mut world = World::new(WorldConfig { hours_per_second: 0.0, start_time_of_day: HOUR, ..Default::default() });
    let (solids, breakables) = vellmark_render::scene_rules(&scene);
    world.set_scene(solids, breakables);
    world.set_trunks(vellmark_render::tree_trunks(&pack, &scene));
    world.set_ground(terrain.clone());
    world.set_word_rules(rules.clone());
    world.set_targets(targets);
    // The writer stands on the bridge road at the south end, facing along the bridge.
    let (s, c) = 173.19f32.to_radians().sin_cos();
    let along = Vec3::new(s, 0.0, c);
    let head = Vec3::new(132.25, 0.0, -113.25) - along * 13.6;
    let feet = Vec3::new(head.x, terrain.height_at(head.x, head.z).unwrap(), head.z);
    let yaw = along.x.atan2(-along.z);
    let id = world.add_player(feet, yaw);
    world.hold_word(id, 1, "WHOLE");
    let eye = feet + Vec3::Y * 1.68;
    let aim = (centre - eye).normalize();
    world.step(&[PlayerInput { look_pitch: aim.y.asin(), ..Default::default() }]);

    // The bank's eye: off to the side of the bridge, a little above it, looking at its middle.
    let right = Vec3::new(-along.z, 0.0, along.x);
    let side = centre + right * 15.0 - along * 3.0;
    let side = Vec3::new(side.x, terrain.height_at(side.x, side.z).unwrap().max(centre.y) + 2.6, side.z);
    let look = centre - side;
    let camera = Camera::looking(side, look.x.atan2(-look.z), (look.y / Vec3::new(look.x, 0.0, look.z).length()).atan());

    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), 640, 360, pack, Some("scene.valley")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    let dt = 1.0 / STEP_HZ as f32;
    let mut seconds = 0.0f32;
    let go = |world: &mut World, buttons: u32, steps: u32, seconds: &mut f32| {
        for _ in 0..steps {
            world.step(&[PlayerInput { buttons, ..Default::default() }]);
            *seconds += dt;
        }
    };
    go(&mut world, 0, 2, &mut seconds);
    assert_eq!(world.snapshot().broken[0], bridge, "the bridge lies broken at load");
    let pieces = world.snapshot().broken.len() - 1;
    assert!(pieces > 30, "and none of its {pieces} pieces is drawn");
    let mut shots = vec![("broken", frame(&mut offscreen, &world.snapshot(), seconds, camera))];
    // A charged WHOLE: the bridge is a little tough.
    go(&mut world, buttons::ATTACK_OFF, STEP_HZ, &mut seconds);
    go(&mut world, 0, 1, &mut seconds);
    let mut mending = false;
    for _ in 0..STEP_HZ * 2 {
        go(&mut world, 0, 1, &mut seconds);
        if world.snapshot().landings.iter().any(|l| l.target == bridge) {
            mending = true;
            break;
        }
    }
    assert!(mending, "WHOLE lands on the bridge");
    go(&mut world, 0, STEP_HZ * 17 / 5, &mut seconds);
    shots.push(("knitting", frame(&mut offscreen, &world.snapshot(), seconds, camera)));
    let hidden = world.snapshot().broken.len() - 1;
    assert!(world.snapshot().broken[0] == bridge && hidden > 0 && hidden < pieces, "still knitting, part of it drawn: {hidden} of {pieces} pieces still to come");
    let mut whole = false;
    for _ in 0..STEP_HZ * 8 {
        go(&mut world, 0, 1, &mut seconds);
        if world.snapshot().broken.is_empty() {
            whole = true;
            break;
        }
    }
    assert!(whole, "the bridge stands whole again");
    go(&mut world, 0, STEP_HZ / 2, &mut seconds);
    shots.push(("whole", frame(&mut offscreen, &world.snapshot(), seconds, camera)));

    let out = vellmark_harness::repo_root().join("build/captures/mending");
    std::fs::create_dir_all(&out).unwrap();
    for (name, img) in &shots {
        img.save(&out.join(format!("{name}.png"))).unwrap();
    }
    // The whole bridge is drawn where the broken one was not: the frames differ.
    let differ = shots[0].1.rgba.iter().zip(&shots[2].1.rgba).filter(|(a, b)| a.abs_diff(**b) > 24).count();
    assert!(differ > 2000, "the whole bridge is seen: {differ} channels changed");
}
