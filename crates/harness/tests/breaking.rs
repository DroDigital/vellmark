//! Breaking a thing of the real yard: its letters fall with the rules' physics and all
//! come to rest on the floor, the step, or the wall, none inside anything.

use vellmark_content::{AssetId, AssetKind, Scene};
use vellmark_sim::{World, WorldConfig};

fn yard() -> Scene {
    let pack = vellmark_harness::capture::baked_base_pack().expect("the base pack bakes");
    Scene::from_bytes(
        pack.get(AssetKind::Scene, AssetId::from_key("scene.yard"))
            .expect("the yard is in the pack"),
    )
    .expect("the yard reads")
}

#[test]
fn the_yard_door_breaks_into_tens_of_thousands_of_letters_that_settle_on_the_floor() {
    let scene = yard();
    let (solids, breakables) = vellmark_render::scene_rules(&scene);
    let mut world = World::new(WorldConfig::default());
    world.set_scene(solids.clone(), breakables);
    let door = AssetId::from_key("scene.yard.door").0;
    let made = world
        .break_thing(door, glam::Vec3::new(0.0, 1.1, -6.6))
        .expect("the door can be broken");
    assert!(
        made > 20_000,
        "the door is tens of thousands of letters: {made}"
    );
    assert!(
        world.break_thing(door, glam::Vec3::ZERO).is_none(),
        "a thing breaks once"
    );

    for _ in 0..60 * 15 {
        world.step(&[]);
    }
    assert_eq!(
        world.letters_moving(),
        0,
        "every letter is at rest within fifteen seconds"
    );
    let snapshot = world.snapshot();
    assert_eq!(snapshot.letters.len(), made);
    assert_eq!(&*snapshot.broken, &[door]);
    if std::env::var_os("VELLMARK_PRINT").is_some() {
        let n = made as f32;
        let mean_z = snapshot.letters.iter().map(|l| l.position.z).sum::<f32>() / n;
        let high = snapshot
            .letters
            .iter()
            .filter(|l| l.position.y > 0.05)
            .count();
        let (min_z, max_z) = snapshot.letters.iter().fold((f32::MAX, f32::MIN), |a, l| {
            (a.0.min(l.position.z), a.1.max(l.position.z))
        });
        eprintln!("made {made} mean z {mean_z} z {min_z}..{max_z} resting above the floor {high}");
        for s in &solids {
            eprintln!("solid {:?} {:?}", s.min, s.max);
        }
    }
    let mut spread = 0;
    for letter in snapshot.letters.iter() {
        let p = letter.position;
        assert!(
            p.y > 0.0 && p.y < 3.0,
            "a letter is under the ground or in the air: {p}"
        );
        // A resting letter sits a hair above what it lies on, so test a point just above it.
        let q = p + glam::Vec3::Y * 0.004;
        assert!(
            !solids
                .iter()
                .any(|s| q.cmpgt(s.min).all() && q.cmplt(s.max).all()),
            "a letter came to rest inside something: {p}"
        );
        if (p.z + 6.2).abs() > 0.6 {
            spread += 1;
        }
    }
    assert!(
        spread > made / 4,
        "the letters scatter across the floor, not into a heap: {spread} of {made}"
    );
}

#[test]
fn a_soft_blow_crumbles_the_door_into_a_heap_on_its_threshold() {
    let scene = yard();
    let (solids, breakables) = vellmark_render::scene_rules(&scene);
    let mut world = World::new(WorldConfig::default());
    world.set_scene(solids, breakables);
    let door = AssetId::from_key("scene.yard.door").0;
    world
        .break_thing_with(door, glam::Vec3::new(0.0, 1.5, -6.9), 0.12)
        .expect("the door can be broken");
    for _ in 0..60 * 15 {
        world.step(&[]);
    }
    assert_eq!(world.letters_moving(), 0);
    let snapshot = world.snapshot();
    let highest = snapshot
        .letters
        .iter()
        .map(|l| l.position.y)
        .fold(0.0, f32::max);
    let near = snapshot
        .letters
        .iter()
        .filter(|l| (l.position.z + 6.2).abs() < 0.6 && l.position.x.abs() < 1.0)
        .count();
    if std::env::var_os("VELLMARK_PRINT").is_some() {
        eprintln!(
            "heap: highest {highest}, {near} of {} near the threshold",
            snapshot.letters.len()
        );
    }
    assert!(
        near > snapshot.letters.len() * 3 / 4,
        "most of the door lies where it stood"
    );
    assert!(highest > 0.1, "it lies in a heap, not a carpet: {highest}");
}
