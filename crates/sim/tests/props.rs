//! Loose props, through `World::step` as a bot drives it: a crate that falls and rests,
//! is picked up and carried, is thrown and breaks on a wall, and a chest too heavy to lift.

use std::sync::Arc;

use glam::{Affine3A, Vec2, Vec3};
use vellmark_sim::character::HeightPatch;
use vellmark_sim::debris::Solid;
use vellmark_sim::{BreakableThing, PlayerInput, STEP_HZ, World, WorldConfig, buttons};

#[derive(Debug)]
struct Flat;

impl vellmark_sim::Ground for Flat {
    fn height(&self, _x: f32, _z: f32) -> Option<f32> {
        Some(0.0)
    }
    fn water(&self, _x: f32, _z: f32) -> Option<f32> {
        None
    }
    fn patches(&self) -> Vec<HeightPatch> {
        vec![HeightPatch {
            origin: Vec2::new(-32.0, -32.0),
            spacing: 1.0,
            samples: 65,
            heights: vec![0.0; 65 * 65],
        }]
    }
}

const CRATE: u64 = 5;
const CHEST: u64 = 6;

fn cube(half: f32) -> Vec<(Vec3, Vec3, u32)> {
    vec![(Vec3::splat(-half), Vec3::splat(half), 1)]
}

/// A player facing north; a crate of 8 kg dropped a little way before them, a chest of
/// 200 kg off to the east, and a wall 5 m north.
fn world() -> World {
    let mut w = World::new(WorldConfig::default());
    w.add_player(Vec3::ZERO, 0.0);
    w.set_ground(Arc::new(Flat));
    w.set_scene(
        vec![Solid {
            min: Vec3::new(-4.0, 0.0, -5.4),
            max: Vec3::new(4.0, 3.0, -5.0),
        }],
        vec![
            BreakableThing {
                id: CRATE,
                to_world: Affine3A::from_translation(Vec3::new(0.0, 0.8, -1.4)),
                boxes: cube(0.25),
                mass: 8.0,
            },
            BreakableThing {
                id: CHEST,
                to_world: Affine3A::from_translation(Vec3::new(3.0, 0.4, 0.0)),
                boxes: cube(0.4),
                mass: 200.0,
            },
        ],
    );
    w
}

fn go(w: &mut World, seconds: f32, input: PlayerInput) {
    for _ in 0..((seconds * STEP_HZ as f32) as u32).max(1) {
        w.step(&[input]);
    }
}

fn place(w: &World, id: u64) -> Vec3 {
    w.snapshot()
        .props
        .iter()
        .find(|(t, _)| *t == id)
        .map(|(_, p)| Vec3::from(p.translation))
        .expect("still a loose thing")
}

fn press(w: &mut World, button: u32) {
    go(
        w,
        0.0,
        PlayerInput {
            buttons: button,
            ..Default::default()
        },
    );
    go(w, 0.0, PlayerInput::default());
}

/// Settles the crate, looks down at it, and picks it up.
fn pick_up_the_crate(w: &mut World) {
    go(w, 1.5, PlayerInput::default());
    go(
        w,
        0.0,
        PlayerInput {
            look_pitch: -0.75,
            ..Default::default()
        },
    );
    press(w, buttons::INTERACT);
    assert_eq!(
        w.carrying(w.snapshot().players[0].id),
        Some(CRATE),
        "the crate is in hand"
    );
    go(
        w,
        0.0,
        PlayerInput {
            look_pitch: 0.75,
            ..Default::default()
        },
    );
}

#[test]
fn a_dropped_crate_falls_and_rests_on_the_ground() {
    let mut w = world();
    go(&mut w, 2.0, PlayerInput::default());
    let at = place(&w, CRATE);
    assert!(
        (at.y - 0.25).abs() < 0.03 && (at.z + 1.4).abs() < 0.05,
        "resting on the ground where it fell: {at}"
    );
}

#[test]
fn a_crate_is_picked_up_and_carried_before_the_eye() {
    let mut w = world();
    pick_up_the_crate(&mut w);
    go(&mut w, 1.0, PlayerInput::default());
    go(
        &mut w,
        0.5,
        PlayerInput {
            move_y: 1.0,
            ..Default::default()
        },
    );
    go(&mut w, 0.5, PlayerInput::default());
    let s = w.snapshot();
    let (feet, at) = (s.players[0].feet, place(&w, CRATE));
    assert!(feet.z < -1.2, "the carrier walked on: {feet}");
    assert!(
        (at.z - feet.z + 1.3).abs() < 0.35 && at.y > 1.0,
        "the crate is held before the eye: {at}, feet {feet}"
    );
    // Interact puts it down: it falls.
    press(&mut w, buttons::INTERACT);
    go(&mut w, 1.5, PlayerInput::default());
    assert!(place(&w, CRATE).y < 0.3, "let go, it falls");
}

#[test]
fn a_thrown_crate_breaks_on_a_wall() {
    let mut w = world();
    pick_up_the_crate(&mut w);
    go(&mut w, 0.5, PlayerInput::default());
    press(&mut w, buttons::ATTACK_MAIN);
    assert_eq!(w.carrying(w.snapshot().players[0].id), None);
    go(&mut w, 1.5, PlayerInput::default());
    let s = w.snapshot();
    assert_eq!(
        *s.broken,
        [CRATE],
        "it struck the wall hard and fell to its letters"
    );
    assert!(!s.letters.is_empty());
    assert!(
        s.letters.iter().all(|l| l.position.z > -5.1),
        "its letters lie on this side of the wall"
    );
}

#[test]
fn a_chest_is_too_heavy_to_lift() {
    let mut w = world();
    go(&mut w, 1.0, PlayerInput::default());
    // Turn to face the chest, east, and look down at it.
    go(
        &mut w,
        0.0,
        PlayerInput {
            look_yaw: std::f32::consts::FRAC_PI_2,
            look_pitch: -0.4,
            ..Default::default()
        },
    );
    go(
        &mut w,
        0.6,
        PlayerInput {
            move_y: 1.0,
            ..Default::default()
        },
    );
    press(&mut w, buttons::INTERACT);
    assert_eq!(w.carrying(w.snapshot().players[0].id), None);
}

#[test]
fn the_same_inputs_move_the_props_the_same_way() {
    let run = || {
        let mut w = world();
        pick_up_the_crate(&mut w);
        press(&mut w, buttons::ATTACK_MAIN);
        go(&mut w, 0.3, PlayerInput::default());
        w.snapshot()
            .props
            .iter()
            .map(|(_, p)| p.translation.to_array())
            .collect::<Vec<_>>()
    };
    assert_eq!(run(), run());
}
