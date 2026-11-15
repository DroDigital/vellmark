//! A Redactor's line, through `World::step`: flying low it strikes through the things its
//! line passes, and takes their Inscriptions; it wounds a player it catches badly but never
//! kills; flying high it strikes nothing; and what it struck stays struck in a saved game.

use glam::{Affine3A, Vec3};
use vellmark_sim::emendation::{Resistance, Target};
use vellmark_sim::redactor::Redactor;
use vellmark_sim::{BreakableThing, PlayerInput, World, WorldConfig, STEP_HZ};

fn world() -> World {
    let mut w = World::new(WorldConfig::default());
    // A milestone by the road, and one well off it.
    let stone = |id: u64, x: f32| BreakableThing { id, to_world: Affine3A::from_translation(Vec3::new(x, 0.0, -30.0)), boxes: vec![(Vec3::new(-0.2, 0.0, -0.2), Vec3::new(0.2, 1.0, 0.2), 0)], mass: 0.0, pieces: Vec::new() };
    let floor = vellmark_sim::debris::Solid { min: Vec3::new(-50.0, -1.0, -100.0), max: Vec3::new(50.0, 0.0, 10.0) };
    w.set_scene(vec![floor], vec![stone(11, 1.0), stone(12, 9.0)]);
    w.set_targets(vec![Target::new(21, 0, Vec3::new(-1.0, 1.0, -50.0), 0.5, Resistance::Plain, &[])]);
    w.add_player(Vec3::new(0.5, 0.0, -40.0), 0.0);
    w
}

fn obelus(strike: Option<(f32, f32)>) -> Redactor {
    Redactor { id: 7, path: vec![Vec3::new(0.0, 1.5, 0.0), Vec3::new(0.0, 1.5, -80.0)], speed: 16.0, flown: 0.0, strike, caught: Vec::new() }
}

fn fly(w: &mut World, seconds: f32) {
    for _ in 0..(seconds * STEP_HZ as f32) as u32 {
        w.step(&[PlayerInput::default()]);
    }
}

#[test]
fn flying_low_it_strikes_through_its_line_and_wounds_without_killing() {
    let mut w = world();
    w.add_redactor(obelus(Some((2.5, 2.5))));
    fly(&mut w, 1.0);
    assert!(!w.snapshot().redactors.is_empty(), "in flight");
    fly(&mut w, 5.0);
    assert!(w.snapshot().redactors.is_empty(), "gone on, its path flown");
    let struck: Vec<u64> = w.struck().iter().map(|s| s.id).collect();
    assert!(struck.contains(&11), "the milestone by the road");
    assert!(!struck.contains(&12), "not the one well off it");
    assert!(struck.contains(&21) && w.snapshot().targets.iter().all(|t| t.id != 21), "struck through, it carries no Inscription");
    let health = w.snapshot().players[0].health;
    assert!(health < 50.0 && health >= 1.0, "wounded badly, not killed: {health}");
    // Caught again when nearly dead, it still does not kill.
    w.add_redactor(obelus(Some((2.5, 2.5))));
    fly(&mut w, 6.0);
    assert!(w.snapshot().players[0].health >= 1.0);
    // Saved and loaded, what it struck stays struck.
    let bytes = w.save().to_bytes();
    let mut fresh = world();
    fresh.load(&vellmark_sim::save::SaveGame::from_bytes(&bytes).unwrap());
    let again: Vec<u64> = fresh.struck().iter().map(|s| s.id).collect();
    assert_eq!(again, struck);
    assert!(fresh.snapshot().targets.iter().all(|t| t.id != 21));
}

#[test]
fn flying_high_it_strikes_nothing() {
    let mut w = world();
    w.add_redactor(obelus(None));
    fly(&mut w, 6.0);
    assert!(w.struck().is_empty());
    assert_eq!(w.snapshot().players[0].health, vellmark_sim::melee::HEALTH_MAX);
}
