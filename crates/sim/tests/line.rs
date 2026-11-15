//! The Line through `World::step`: it waits for the night, the words, and the road;
//! then Obelus comes down the road from behind, strikes what stands beside it, wounds a
//! player still on the road without killing, and goes north; the flag is set, the end
//! card is asked for once, and it never comes again.

use glam::{Affine3A, Vec3};
use vellmark_sim::emendation::{RuleOutcome, WordRule, WordRules};
use vellmark_sim::line::{State, END_EVENT, FLAG};
use vellmark_sim::{BreakableThing, PlayerInput, World, WorldConfig, STEP_HZ};

fn rules() -> WordRules {
    let word = |s: &str| WordRule { spelling: s.into(), class: "Access".into(), ink: 8.0, flight_speed: 14.0, outcomes: vec![RuleOutcome::Refusal(1)], weapon: RuleOutcome::Refusal(1), typo: format!("{s}X"), typo_outcomes: vec![RuleOutcome::Refusal(1)] };
    WordRules { classes: vec!["thing".into()], words: ["OPEN", "LIT", "SHARP", "COLD"].map(word).to_vec(), compounds: vec![] }
}

/// A straight road running north, flown at 1.5 m, with a milestone beside it and the
/// player on it, facing north, at ten at night.
fn world(hour: f32) -> World {
    let mut w = World::new(WorldConfig { start_time_of_day: hour, hours_per_second: 0.0, ..Default::default() });
    let floor = vellmark_sim::debris::Solid { min: Vec3::new(-400.0, -1.0, -600.0), max: Vec3::new(400.0, 0.0, 400.0) };
    let stone = BreakableThing { id: 31, to_world: Affine3A::from_translation(Vec3::new(2.0, 0.0, -60.0)), boxes: vec![(Vec3::new(-0.2, 0.0, -0.2), Vec3::new(0.2, 1.0, 0.2), 0)], mass: 0.0, pieces: Vec::new() };
    w.set_scene(vec![floor], vec![stone]);
    w.set_word_rules(rules());
    w.add_player(Vec3::new(0.0, 0.0, -100.0), 0.0);
    w.set_line_road((0..=6).map(|k| Vec3::new(0.0, 1.5, 100.0 - k as f32 * 60.0)).collect());
    w
}

fn learn_four(w: &mut World) {
    let me = w.snapshot().players[0].id;
    for s in ["OPEN", "LIT", "SHARP", "COLD"] {
        w.hold_word(me, 1, s);
    }
}

fn run(w: &mut World, seconds: f32) -> Vec<String> {
    let mut events = Vec::new();
    for _ in 0..(seconds * STEP_HZ as f32) as u32 {
        w.step(&[PlayerInput::default()]);
        events.extend(w.events().iter().map(|(_, e)| e.clone()));
    }
    events
}

#[test]
fn it_waits_for_the_night_and_the_words() {
    let mut w = world(14.0);
    learn_four(&mut w);
    run(&mut w, 1.0);
    assert_eq!(*w.line(), State::Waiting, "not by day");
    let mut w = world(22.0);
    run(&mut w, 1.0);
    assert_eq!(*w.line(), State::Waiting, "not before four words");
    learn_four(&mut w);
    run(&mut w, 0.2);
    assert_eq!(*w.line(), State::Flying, "at night, with the words, on the road");
}

#[test]
fn it_comes_from_behind_strikes_wounds_goes_and_asks_for_the_end_card_once() {
    let mut w = world(22.0);
    learn_four(&mut w);
    run(&mut w, 0.2);
    let start = w.snapshot().redactors[0].at;
    assert!(start.z > -100.0 && start.y > 20.0, "from behind, high: {start:?}");
    let events = run(&mut w, 150.0);
    assert!(w.snapshot().redactors.is_empty(), "gone north");
    assert!(w.struck().iter().any(|s| s.id == 31), "the milestone is struck through");
    let p = &w.snapshot().players[0];
    assert!(p.health > 0.0 && p.health < 100.0, "caught on the road: wounded, not killed ({})", p.health);
    assert!(w.flags().contains(FLAG));
    assert_eq!(events.iter().filter(|e| *e == END_EVENT).count(), 1, "the end card, once");
    assert_eq!(*w.line(), State::Passed);
    // Never again, even after a save is loaded.
    let save = w.save();
    let mut again = world(22.0);
    again.load(&save);
    learn_four(&mut again);
    let events = run(&mut again, 5.0);
    assert!(again.snapshot().redactors.is_empty() && events.is_empty(), "it does not come again");
}
