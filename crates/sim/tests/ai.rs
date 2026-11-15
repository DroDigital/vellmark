//! Creatures together, through `World::step`: one that notices a player calls its kin;
//! and a pack strikes one at a time while the rest circle to the sides.

use glam::Vec3;
use vellmark_sim::inkling::inkling;
use vellmark_sim::{PlayerInput, World, WorldConfig, STEP_HZ};

fn world(inklings: &[(u64, Vec3)]) -> World {
    let mut w = World::new(WorldConfig::default());
    w.add_player(Vec3::ZERO, 0.0);
    for (id, at) in inklings {
        let mut c = inkling(*id, *at, None);
        c.yaw = std::f32::consts::PI;
        w.add_creature(c);
    }
    w
}

#[test]
fn one_that_notices_calls_its_kin() {
    // One faces the player; the other, behind it, faces away and is out of hearing.
    let mut w = world(&[(1, Vec3::new(0.0, 0.0, -9.0)), (2, Vec3::new(0.0, 0.0, -16.0))]);
    w.creatures_mut()[1].yaw = 0.0;
    w.step(&[PlayerInput::default()]);
    assert!(w.creatures()[0].aware, "the near one sees the player");
    assert!(w.creatures()[1].aware, "and the far one, called, knows too");
    assert!(w.snapshot().heard.iter().any(|h| matches!(h, vellmark_sim::Heard::Call { .. })));
}

#[test]
fn a_pack_strikes_one_at_a_time_and_the_rest_circle() {
    let mut w = world(&[(1, Vec3::new(-0.6, 0.0, -4.0)), (2, Vec3::new(0.6, 0.0, -4.2)), (3, Vec3::new(0.0, 0.0, -4.4))]);
    let mut most_at_once = 0;
    let mut sides = 0.0f32;
    for _ in 0..STEP_HZ * 6 {
        w.step(&[PlayerInput::default()]);
        let gathering = w.creatures().iter().filter(|c| c.telegraph > 0.0).count();
        most_at_once = most_at_once.max(gathering);
        for c in w.creatures() {
            sides = sides.max(c.feet.x.abs());
        }
    }
    assert_eq!(most_at_once, 1, "one strikes at a time");
    assert!(sides > 1.2, "the others circle to the player's sides: {sides}");
}
