//! Reading through the rules: a text in focus is read, with the hidden lines the
//! reader's Perception reaches; with nothing in focus, Read is close reading, and in solo
//! play the world slows to a third while it is held.

use glam::Vec3;
use vellmark_sim::reading::{ReadableText, ReadingSnap};
use vellmark_sim::{buttons, PlayerInput, World, WorldConfig, STEP_HZ};

fn world() -> World {
    let mut w = World::new(WorldConfig::default());
    // Facing north (-Z), a metre and a half from a wall with a text on it.
    w.add_player(Vec3::new(0.0, 0.0, -1.0), 0.0);
    w.set_readables(vec![ReadableText {
        id: 7,
        text: 70,
        min: Vec3::new(-1.0, 0.0, -3.0),
        max: Vec3::new(1.0, 2.5, -2.8),
        hidden: vec![(71, 2), (72, 5)],
        teaches: String::new(),
    }]);
    w
}

fn read() -> PlayerInput {
    PlayerInput { buttons: buttons::READ, ..Default::default() }
}

#[test]
fn a_text_in_focus_is_read_with_the_lines_perception_reaches() {
    let mut w = world();
    w.step(&[PlayerInput::default()]);
    assert_eq!(w.snapshot().players[0].in_focus, Some(7), "the wall's text is in focus");
    w.step(&[read()]);
    assert_eq!(w.snapshot().players[0].reading, ReadingSnap::Text { id: 7, text: 70, hidden: vec![] });
    // A sharper reader sees the first hidden line, not the second.
    let id = w.snapshot().players[0].id;
    w.set_perception(id, 3);
    w.step(&[read()]);
    assert_eq!(w.snapshot().players[0].reading, ReadingSnap::Text { id: 7, text: 70, hidden: vec![71] });
    // Letting go stops reading.
    w.step(&[PlayerInput::default()]);
    assert_eq!(w.snapshot().players[0].reading, ReadingSnap::None);
}

#[test]
fn with_nothing_in_focus_read_is_close_reading_and_slows_time() {
    let mut w = world();
    // Turned round, away from the wall.
    w.step(&[PlayerInput { look_yaw: std::f32::consts::PI, ..Default::default() }]);
    let walk = |w: &mut World, buttons: u32| {
        let start = w.snapshot().players[0].feet;
        let hour = w.snapshot().time_of_day;
        for _ in 0..STEP_HZ {
            w.step(&[PlayerInput { move_y: 1.0, buttons, ..Default::default() }]);
        }
        (w.snapshot().players[0].feet.distance(start), w.snapshot().time_of_day - hour)
    };
    let (normal, hours) = walk(&mut w, 0);
    let (slowed, slow_hours) = walk(&mut w, buttons::READ);
    assert_eq!(w.snapshot().players[0].reading, ReadingSnap::Close);
    assert!((slowed / normal - 1.0 / 3.0).abs() < 0.05, "walked {slowed} against {normal}");
    assert!((slow_hours / hours - 1.0 / 3.0).abs() < 0.05);
}

#[test]
fn in_co_op_close_reading_never_slows_the_shared_world() {
    let mut w = world();
    w.add_player(Vec3::new(5.0, 0.0, 5.0), 0.0);
    let hour = w.snapshot().time_of_day;
    for _ in 0..STEP_HZ {
        w.step(&[PlayerInput { look_yaw: 0.0, buttons: buttons::READ, ..Default::default() }, PlayerInput::default()]);
    }
    // The second player faces nothing: it close-reads, and the world keeps its pace.
    let s = w.snapshot();
    assert_eq!(s.players[1].reading, ReadingSnap::None);
    let elapsed = s.time_of_day - hour;
    let mut solo = world();
    let solo_hour = solo.snapshot().time_of_day;
    for _ in 0..STEP_HZ {
        solo.step(&[PlayerInput::default()]);
    }
    assert!((elapsed - (solo.snapshot().time_of_day - solo_hour)).abs() < 1.0e-6);
}
