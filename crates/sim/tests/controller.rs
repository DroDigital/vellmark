//! The first-person body, driven as a bot drives it: through `World::step`, on flat
//! ground with a wall, a ledge, a low roof, and a pool.

use std::sync::Arc;

use glam::{Vec2, Vec3};
use vellmark_sim::character::{HeightPatch, JOG, SIDESTEP_DISTANCE, SPRINT, SWIM_DEPTH};
use vellmark_sim::debris::Solid;
use vellmark_sim::{PlayerInput, STEP_HZ, World, WorldConfig, buttons};

/// Flat ground at 0, with a pool where x > 40 whose water stands 1.8 m deep.
#[derive(Debug)]
struct Flat;

impl vellmark_sim::Ground for Flat {
    fn height(&self, x: f32, _z: f32) -> Option<f32> {
        Some(if x > 40.0 { -1.8 } else { 0.0 })
    }
    fn water(&self, x: f32, _z: f32) -> Option<f32> {
        (x > 40.0).then_some(0.0)
    }
    fn patches(&self) -> Vec<HeightPatch> {
        let n = 129;
        let heights = (0..n * n)
            .map(|i| {
                if (i % n) as f32 - 64.0 > 40.0 {
                    -1.8
                } else {
                    0.0
                }
            })
            .collect();
        vec![HeightPatch {
            origin: Vec2::new(-64.0, -64.0),
            spacing: 1.0,
            samples: n,
            heights,
        }]
    }
}

fn boxed(min: (f32, f32, f32), max: (f32, f32, f32)) -> Solid {
    Solid {
        min: Vec3::from(min),
        max: Vec3::from(max),
    }
}

fn world(feet: Vec3, yaw: f32) -> World {
    let mut w = World::new(WorldConfig::default());
    w.add_player(feet, yaw);
    w.set_ground(Arc::new(Flat));
    w.set_scene(
        vec![
            // A wall to the north of the start.
            boxed((-5.0, 0.0, -11.0), (5.0, 3.0, -10.0)),
            // A ledge 1.2 m high to the west.
            boxed((-30.0, 0.0, -5.0), (-20.0, 1.2, 5.0)),
            // A low roof over a crawlspace to the east: open below 1.3 m.
            boxed((15.0, 1.3, -5.0), (25.0, 1.6, 5.0)),
        ],
        Vec::new(),
    );
    w
}

fn go(w: &mut World, seconds: f32, input: PlayerInput) {
    for _ in 0..(seconds * STEP_HZ as f32) as u32 {
        w.step(&[input]);
    }
}

fn feet(w: &World) -> Vec3 {
    w.snapshot().players[0].feet
}

fn forward(buttons: u32) -> PlayerInput {
    PlayerInput {
        move_y: 1.0,
        buttons,
        ..Default::default()
    }
}

#[test]
fn it_jogs_and_sprints_at_the_briefs_speeds() {
    let mut w = world(Vec3::new(0.0, 0.0, 30.0), 0.0);
    // Up to speed, then a second's distance.
    go(&mut w, 0.5, forward(0));
    let a = feet(&w);
    go(&mut w, 1.0, forward(0));
    let jog = a.distance(feet(&w));
    assert!((jog - JOG).abs() < 0.15, "jogs {jog} m/s");
    go(&mut w, 0.5, forward(buttons::SPRINT));
    let b = feet(&w);
    go(&mut w, 1.0, forward(buttons::SPRINT));
    let sprint = b.distance(feet(&w));
    assert!((sprint - SPRINT).abs() < 0.2, "sprints {sprint} m/s");
    assert!(
        feet(&w).y.abs() < 0.05,
        "stays on the ground: {}",
        feet(&w).y
    );
}

#[test]
fn walls_stop_it() {
    let mut w = world(Vec3::new(0.0, 0.0, -5.0), 0.0);
    go(&mut w, 3.0, forward(buttons::SPRINT));
    let f = feet(&w);
    assert!(f.z > -10.0 && f.z < -9.4, "stopped at the wall: {f}");
}

#[test]
fn it_jumps_about_a_metre_and_lands() {
    let mut w = world(Vec3::new(0.0, 0.0, 30.0), 0.0);
    go(&mut w, 0.2, PlayerInput::default());
    w.step(&[PlayerInput {
        buttons: buttons::JUMP,
        ..Default::default()
    }]);
    let mut top: f32 = 0.0;
    for _ in 0..STEP_HZ {
        w.step(&[PlayerInput {
            buttons: buttons::JUMP,
            ..Default::default()
        }]);
        top = top.max(feet(&w).y);
    }
    assert!((0.85..1.2).contains(&top), "rose {top} m");
    assert!(
        feet(&w).y.abs() < 0.05 && w.snapshot().players[0].grounded,
        "landed: {}",
        feet(&w)
    );
    // Holding the button does not bounce it again.
    go(
        &mut w,
        0.5,
        PlayerInput {
            buttons: buttons::JUMP,
            ..Default::default()
        },
    );
    assert!(feet(&w).y.abs() < 0.05);
}

#[test]
fn it_mantles_onto_a_ledge_it_jumps_at() {
    // Facing west (yaw 270 degrees turns left), a few metres from the 1.2 m ledge.
    let yaw = 1.5 * std::f32::consts::PI;
    let mut w = world(Vec3::new(-17.0, 0.0, 0.0), yaw);
    go(&mut w, 0.5, forward(0));
    go(&mut w, 1.0, forward(buttons::JUMP));
    let f = feet(&w);
    assert!(
        (f.y - 1.2).abs() < 0.1 && f.x < -20.0,
        "on top of the ledge: {f}"
    );
}

#[test]
fn the_sidestep_goes_about_two_metres_and_is_untouchable_at_first() {
    let mut w = world(Vec3::new(0.0, 0.0, 30.0), 0.0);
    go(&mut w, 0.2, PlayerInput::default());
    let start = feet(&w);
    // Still, it steps back: +Z.
    w.step(&[PlayerInput {
        buttons: buttons::SIDESTEP,
        ..Default::default()
    }]);
    assert!(w.snapshot().players[0].untouchable);
    go(&mut w, 0.5, PlayerInput::default());
    let moved = feet(&w) - start;
    assert!(
        (moved.z - SIDESTEP_DISTANCE).abs() < 0.25 && moved.x.abs() < 0.05,
        "stepped {moved}"
    );
    assert!(!w.snapshot().players[0].untouchable);
    // Moving right, it steps right.
    let before = feet(&w);
    w.step(&[PlayerInput {
        move_x: 1.0,
        buttons: buttons::SIDESTEP,
        ..Default::default()
    }]);
    go(
        &mut w,
        0.4,
        PlayerInput {
            move_x: 1.0,
            ..Default::default()
        },
    );
    assert!(feet(&w).x - before.x > SIDESTEP_DISTANCE, "{}", feet(&w));
}

#[test]
fn crouching_fits_under_a_low_roof_and_stays_down_until_there_is_room() {
    // Facing east, into the crawlspace.
    let yaw = 0.5 * std::f32::consts::PI;
    let mut w = world(Vec3::new(10.0, 0.0, 0.0), yaw);
    // Standing, the roof's edge stops it.
    go(&mut w, 2.0, forward(0));
    assert!(feet(&w).x < 15.0, "{}", feet(&w));
    // Crouched, it goes in.
    go(&mut w, 3.0, forward(buttons::SNEAK));
    assert!(feet(&w).x > 17.0, "{}", feet(&w));
    // Letting go of crouch underneath, it stays crouched.
    go(&mut w, 0.3, PlayerInput::default());
    assert!(w.snapshot().players[0].crouching);
    let eye = w.snapshot().views[0].eye.y - feet(&w).y;
    assert!(eye < 1.2, "the eye is low: {eye}");
}

#[test]
fn deep_water_is_swum_with_the_eye_above_it() {
    // Facing east, into the pool.
    let yaw = 0.5 * std::f32::consts::PI;
    let mut w = world(Vec3::new(35.0, 0.0, 0.0), yaw);
    go(&mut w, 4.0, forward(0));
    let s = w.snapshot();
    assert!(s.players[0].swimming, "{}", feet(&w));
    let eye = s.views[0].eye.y;
    assert!(
        eye > 0.0 && eye < 0.4,
        "the eye rides just above the water: {eye}"
    );
    assert!(feet(&w).y < -SWIM_DEPTH + 1.0);
}

#[test]
fn the_same_inputs_give_the_same_body() {
    let run = || {
        let mut w = world(Vec3::new(0.0, 0.0, 30.0), 0.3);
        for i in 0..600u32 {
            let b = if i % 97 == 0 {
                buttons::JUMP
            } else if i % 131 == 0 {
                buttons::SIDESTEP
            } else {
                0
            };
            w.step(&[PlayerInput {
                move_x: (i as f32 * 0.05).sin(),
                move_y: 1.0,
                look_yaw: 0.004,
                buttons: b,
                ..Default::default()
            }]);
        }
        w.state_hash()
    };
    assert_eq!(run(), run());
}
