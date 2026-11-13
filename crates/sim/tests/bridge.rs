//! The Mended Bridge, as the rules see it: a marked structure whose Inscription says
//! `BROKEN` lies in letters below where it stood, nobody can cross it, and `WHOLE` calls
//! its letters home until it bears a body across. Its typo knocks a whole thing down.

use std::sync::Arc;

use glam::{Affine3A, Vec2, Vec3};
use vellmark_sim::character::HeightPatch;
use vellmark_sim::emendation::{Resistance, RuleOutcome, Target, WordRule, WordRules};
use vellmark_sim::{BreakableThing, PlayerInput, STEP_HZ, World, WorldConfig, buttons};

/// A bank at 0 where z > -2, and a ravine 4 m deep beyond it.
#[derive(Debug)]
struct Ravine;

fn depth(z: f32) -> f32 {
    if z > -2.0 { 0.0 } else { -4.0 }
}

impl vellmark_sim::Ground for Ravine {
    fn height(&self, _x: f32, z: f32) -> Option<f32> {
        Some(depth(z))
    }
    fn water(&self, _x: f32, _z: f32) -> Option<f32> {
        None
    }
    fn patches(&self) -> Vec<HeightPatch> {
        let n = 129;
        let heights = (0..n * n)
            .map(|i| depth((i / n) as f32 * 0.5 - 32.0))
            .collect();
        vec![HeightPatch {
            origin: Vec2::new(-32.0, -32.0),
            spacing: 0.5,
            samples: n,
            heights,
        }]
    }
}

const BRIDGE: u64 = 77;

fn rules() -> WordRules {
    let e = |effect: &str| RuleOutcome::Effect {
        effect: effect.into(),
        strength: 1.0,
        preview: 100,
    };
    WordRules {
        classes: vec!["structure".into()],
        words: vec![WordRule {
            spelling: "WHOLE".into(),
            class: "Integrity".into(),
            ink: 12.0,
            flight_speed: 14.0,
            outcomes: vec![e("mend")],
            weapon: RuleOutcome::Refusal(1),
            typo: "HOLE".into(),
            typo_outcomes: vec![e("gap")],
        }],
        compounds: vec![],
    }
}

/// A player on the bank facing north over the ravine, `WHOLE` in the off hand, and a
/// bridge of planks across it that the world will find `inscribed`.
fn world(inscribed: &str) -> World {
    let mut w = World::new(WorldConfig::default());
    let id = w.add_player(Vec3::new(0.0, 0.0, 0.0), 0.0);
    w.set_ground(Arc::new(Ravine));
    let planks = (0..10)
        .map(|k| {
            (
                Vec3::new(-1.1, -0.12, -2.0 - (k + 1) as f32 * 1.2),
                Vec3::new(1.1, 0.0, -2.0 - k as f32 * 1.2),
                1,
            )
        })
        .collect();
    w.set_scene(
        Vec::new(),
        vec![BreakableThing {
            id: BRIDGE,
            to_world: Affine3A::IDENTITY,
            boxes: planks,
            mass: 0.0,
        }],
    );
    w.set_word_rules(rules());
    w.set_targets(vec![Target::new(
        BRIDGE,
        0,
        Vec3::new(0.0, 0.0, -8.0),
        6.0,
        Resistance::Plain,
        &[(inscribed.into(), "Integrity".into())],
    )]);
    w.hold_word(id, 1, "WHOLE");
    w
}

fn go(w: &mut World, seconds: f32, input: PlayerInput) {
    for _ in 0..(seconds * STEP_HZ as f32) as u32 {
        w.step(&[input]);
    }
}

/// Two seconds' walk north: out over the ravine, short of the far end of the bridge.
fn walk_across(w: &mut World) -> Vec3 {
    go(
        w,
        2.0,
        PlayerInput {
            move_y: 1.0,
            ..Default::default()
        },
    );
    w.snapshot().players[0].feet
}

#[test]
fn a_bridge_inscribed_broken_lies_in_letters_and_cannot_be_crossed() {
    let mut w = world("BROKEN");
    go(&mut w, 0.1, PlayerInput::default());
    let s = w.snapshot();
    assert_eq!(*s.broken, [BRIDGE], "it is broken from the start");
    assert!(
        s.letters.len() > 100,
        "its letters lie about: {}",
        s.letters.len()
    );
    let below = s.letters.iter().filter(|l| l.position.y < -3.0).count();
    assert!(
        below * 10 > s.letters.len() * 9,
        "they lie in the ravine: {below} of {}",
        s.letters.len()
    );
    assert!(
        s.letters.iter().all(|l| l.position.y < 0.3),
        "at rest, none in the air"
    );
    let feet = walk_across(&mut w);
    assert!(feet.y < -3.0, "a body walking out falls: {feet}");
}

#[test]
fn whole_calls_its_letters_home_and_it_bears_a_body_across() {
    let mut w = world("BROKEN");
    go(&mut w, 0.1, PlayerInput::default());
    go(
        &mut w,
        0.1,
        PlayerInput {
            buttons: buttons::ATTACK_OFF,
            ..Default::default()
        },
    );
    // The word flies, lands, and the letters rise and knit.
    let mut whole_after = None;
    for step in 0..STEP_HZ * 6 {
        go(&mut w, 1.0 / STEP_HZ as f32, PlayerInput::default());
        if w.snapshot().broken.is_empty() {
            whole_after = Some(step as f32 / STEP_HZ as f32);
            break;
        }
    }
    let after = whole_after.expect("the bridge is whole again");
    assert!(
        after > 1.0 && after < 4.0,
        "it comes back together over a moment, not at once: {after} s"
    );
    let s = w.snapshot();
    assert!(
        s.letters.is_empty(),
        "no letter is left lying: {}",
        s.letters.len()
    );
    assert_eq!(s.targets[0].words, ["WHOLE"], "its Inscription says so");
    let feet = walk_across(&mut w);
    assert!(
        feet.y > -0.2 && feet.z < -6.0,
        "it bears a body across: {feet}"
    );
    // Mending lasts.
    go(&mut w, 60.0, PlayerInput::default());
    assert!(w.snapshot().broken.is_empty());
    assert_eq!(w.snapshot().targets[0].words, ["WHOLE"]);
}

#[test]
fn hole_knocks_a_whole_thing_down() {
    let mut w = world("WHOLE");
    go(&mut w, 0.1, PlayerInput::default());
    assert!(w.snapshot().broken.is_empty());
    // Typing the typo as the word charges.
    go(
        &mut w,
        0.3,
        PlayerInput {
            buttons: buttons::ATTACK_OFF,
            ..Default::default()
        },
    );
    w.step(&[PlayerInput {
        buttons: buttons::ATTACK_OFF,
        typed: PlayerInput::pack_typed("HOLE"),
        ..Default::default()
    }]);
    go(
        &mut w,
        0.2,
        PlayerInput {
            buttons: buttons::ATTACK_OFF,
            ..Default::default()
        },
    );
    go(&mut w, 2.0, PlayerInput::default());
    assert_eq!(*w.snapshot().broken, [BRIDGE], "HOLE knocks it down");
    assert_eq!(w.snapshot().targets[0].words, ["BROKEN"]);
}

#[test]
fn it_knits_outward_from_the_writer_and_can_be_walked_out_on_as_it_comes() {
    let mut w = world("BROKEN");
    go(&mut w, 0.1, PlayerInput::default());
    go(
        &mut w,
        0.1,
        PlayerInput {
            buttons: buttons::ATTACK_OFF,
            ..Default::default()
        },
    );
    // A moment for the nearest planks.
    go(&mut w, 2.9, PlayerInput::default());
    assert_eq!(
        *w.snapshot().broken,
        [BRIDGE],
        "the far end is not yet home"
    );
    let mut lowest = f32::MAX;
    for _ in 0..STEP_HZ * 2 {
        go(
            &mut w,
            1.0 / STEP_HZ as f32,
            PlayerInput {
                move_y: 1.0,
                ..Default::default()
            },
        );
        lowest = lowest.min(w.snapshot().players[0].feet.y);
    }
    let feet = w.snapshot().players[0].feet;
    assert!(
        feet.z < -6.0 && lowest > -0.2,
        "the bridge came back under the walker's feet: {feet}, lowest {lowest}"
    );
}
