//! Typos, compounds, and the chain: a typo cast on purpose by typing it as the word
//! charges; a full charge smudged into its typo by a blow; a lesser charge lost; a
//! compound firing when its second word lands; and fire spreading from a burning hedge
//! to its neighbours, a link at a time, and dying out.

use glam::Vec3;
use vellmark_sim::emendation::{
    CompoundRule, Landing, Resistance, RuleOutcome, Target, WordRule, WordRules,
};
use vellmark_sim::{PlayerInput, STEP_HZ, World, WorldConfig, buttons};

fn rules() -> WordRules {
    let e = |effect: &str| RuleOutcome::Effect {
        effect: effect.into(),
        strength: 1.0,
        preview: 100,
    };
    WordRules {
        classes: vec!["plant".into(), "stone".into()],
        words: vec![
            WordRule {
                spelling: "LIT".into(),
                class: "Light".into(),
                ink: 6.0,
                flight_speed: 30.0,
                outcomes: vec![e("glow"), e("glow")],
                weapon: e("light"),
                typo: "LIP".into(),
                typo_outcomes: vec![e("answer"), e("answer")],
            },
            WordRule {
                spelling: "BURNING".into(),
                class: "Temperature".into(),
                ink: 10.0,
                flight_speed: 30.0,
                outcomes: vec![e("ignite"), e("heat")],
                weapon: e("ignite"),
                typo: "TURNING".into(),
                typo_outcomes: vec![e("turn"), RuleOutcome::Refusal(9)],
            },
        ],
        compounds: vec![CompoundRule {
            name: "Blazing".into(),
            words: [0, 1],
            outcomes: vec![e("blaze"), RuleOutcome::Refusal(11)],
        }],
    }
}

/// A hedge of five bushes running north, a metre and a half apart; the player faces the
/// first, four metres off.
fn world() -> World {
    let mut w = World::new(WorldConfig::default());
    let id = w.add_player(Vec3::new(0.0, 0.0, 4.0), 0.0);
    w.set_word_rules(rules());
    let bushes = (0..5)
        .map(|i| {
            Target::new(
                100 + i,
                0,
                Vec3::new(0.0, 1.6, -(i as f32) * 1.5),
                0.6,
                Resistance::Plain,
                &[],
            )
        })
        .collect();
    w.set_targets(bushes);
    w.hold_word(id, 1, "BURNING");
    w.step(&[PlayerInput::default()]);
    w
}

fn hold(w: &mut World, input: PlayerInput, steps: u32) {
    for _ in 0..steps {
        w.step(&[input]);
    }
}

fn words(w: &World, i: usize) -> Vec<String> {
    w.snapshot().targets[i].words.clone()
}

#[test]
fn typing_the_typo_as_the_word_charges_casts_the_typo() {
    let mut w = world();
    hold(
        &mut w,
        PlayerInput {
            buttons: buttons::ATTACK_OFF,
            ..Default::default()
        },
        20,
    );
    for chunk in ["TURN", "ING"] {
        w.step(&[PlayerInput {
            buttons: buttons::ATTACK_OFF,
            typed: PlayerInput::pack_typed(chunk),
            ..Default::default()
        }]);
    }
    hold(
        &mut w,
        PlayerInput {
            buttons: buttons::ATTACK_OFF,
            ..Default::default()
        },
        10,
    );
    hold(&mut w, PlayerInput::default(), STEP_HZ / 2);
    assert_eq!(words(&w, 0), ["TURNING"], "the typo was written");
    assert!(
        w.snapshot().targets[0]
            .effects
            .contains(&"turn".to_string())
    );
}

#[test]
fn a_blow_smudges_a_full_charge_into_its_typo_and_loses_a_lesser_one() {
    let mut w = world();
    let id = w.snapshot().players[0].id;
    // A lesser charge, struck: lost, and letting go does nothing.
    hold(
        &mut w,
        PlayerInput {
            buttons: buttons::ATTACK_OFF,
            ..Default::default()
        },
        30,
    );
    w.strike(id);
    hold(
        &mut w,
        PlayerInput {
            buttons: buttons::ATTACK_OFF,
            ..Default::default()
        },
        5,
    );
    hold(&mut w, PlayerInput::default(), STEP_HZ);
    assert!(
        words(&w, 0).is_empty(),
        "nothing was written: {:?}",
        words(&w, 0)
    );
    // A full charge, struck: it flies as its typo.
    hold(
        &mut w,
        PlayerInput {
            buttons: buttons::ATTACK_OFF,
            ..Default::default()
        },
        STEP_HZ * 3 / 2,
    );
    w.strike(id);
    assert_eq!(w.snapshot().flights.len(), 1);
    assert_eq!(w.snapshot().flights[0].typo, "TURNING");
    hold(&mut w, PlayerInput::default(), STEP_HZ);
    assert_eq!(words(&w, 0), ["TURNING"]);
}

#[test]
fn the_second_word_of_a_compound_fires_it() {
    let mut w = world();
    let id = w.snapshot().players[0].id;
    w.hold_word(id, 1, "LIT");
    hold(
        &mut w,
        PlayerInput {
            buttons: buttons::ATTACK_OFF,
            ..Default::default()
        },
        3,
    );
    hold(&mut w, PlayerInput::default(), STEP_HZ / 2);
    w.hold_word(id, 1, "BURNING");
    hold(
        &mut w,
        PlayerInput {
            buttons: buttons::ATTACK_OFF,
            ..Default::default()
        },
        3,
    );
    let mut compound = None;
    for _ in 0..STEP_HZ / 2 {
        w.step(&[PlayerInput::default()]);
        for l in &w.snapshot().landings {
            if l.compound.is_some() {
                compound = l.compound.clone();
            }
        }
    }
    assert_eq!(
        compound,
        Some((
            "Blazing".to_string(),
            Landing::Took {
                effect: "blaze".into(),
                strength: 1.0
            }
        ))
    );
    assert!(
        w.snapshot().targets[0]
            .effects
            .contains(&"blaze".to_string())
    );
}

#[test]
fn fire_spreads_along_the_hedge_a_link_at_a_time_and_dies_out() {
    let mut w = world();
    hold(
        &mut w,
        PlayerInput {
            buttons: buttons::ATTACK_OFF,
            ..Default::default()
        },
        3,
    );
    let mut deepest = 0;
    let mut caught = std::collections::BTreeSet::new();
    for _ in 0..25 * STEP_HZ {
        w.step(&[PlayerInput::default()]);
        for l in &w.snapshot().landings {
            deepest = deepest.max(l.link);
            caught.insert(l.target);
        }
    }
    assert_eq!(caught.len(), 5, "every bush caught: {caught:?}");
    assert!(
        deepest >= 3,
        "the fire went from bush to bush: {deepest} links"
    );
    // A minute and a half later, it has all burnt out.
    hold(&mut w, PlayerInput::default(), 90 * STEP_HZ);
    assert!((0..5).all(|i| words(&w, i).is_empty()), "it burnt out");
}
