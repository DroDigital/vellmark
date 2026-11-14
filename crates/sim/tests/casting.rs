//! Casting through the rules: a word in a hand, a tap or a charge, the word flying and
//! landing, the Inscription changed, Ink spent and coming back, the preview at the
//! crosshair, and both hands together.

use glam::Vec3;
use vellmark_sim::casting::Holding;
use vellmark_sim::emendation::{Landing, Preview, Resistance, RuleOutcome, Target, WordRule, WordRules, INK_MAX};
use vellmark_sim::{buttons, PlayerInput, World, WorldConfig, STEP_HZ};

fn rules() -> WordRules {
    let e = |effect: &str| RuleOutcome::Effect { effect: effect.into(), strength: 1.0, preview: 100 };
    WordRules {
        classes: vec!["door".into(), "stone".into()],
        words: vec![
            WordRule { spelling: "OPEN".into(), class: "Access".into(), ink: 8.0, flight_speed: 14.0, outcomes: vec![e("open"), RuleOutcome::Refusal(7)], weapon: RuleOutcome::Refusal(8), typo: "OVEN".into(), typo_outcomes: vec![e("heat"), e("heat")] },
            WordRule { spelling: "BURNING".into(), class: "Temperature".into(), ink: 10.0, flight_speed: 15.0, outcomes: vec![e("ignite"), e("heat")], weapon: e("ignite"), typo: "TURNING".into(), typo_outcomes: vec![e("turn"), RuleOutcome::Refusal(9)] },
        ],
        compounds: vec![],
    }
}

/// A player facing north at a door five metres off, holding OPEN in the off hand.
fn world(resistance: Resistance) -> World {
    let mut w = World::new(WorldConfig::default());
    let id = w.add_player(Vec3::new(0.0, 0.0, 0.0), 0.0);
    w.set_word_rules(rules());
    w.set_targets(vec![Target::new(42, 0, Vec3::new(0.0, 1.6, -5.0), 1.1, resistance, &[("LOCKED".into(), "Access".into())])]);
    w.hold_word(id, 1, "OPEN");
    w
}

fn hold(w: &mut World, buttons: u32, steps: u32) {
    for _ in 0..steps {
        w.step(&[PlayerInput { buttons, ..Default::default() }]);
    }
}

fn words(w: &World) -> Vec<String> {
    w.snapshot().targets[0].words.clone()
}

#[test]
fn a_tap_writes_the_word_which_flies_and_lands() {
    let mut w = world(Resistance::Plain);
    hold(&mut w, 0, 2);
    let s = w.snapshot();
    assert_eq!(s.players[0].holding[1], Holding::Word(0));
    assert_eq!(s.players[0].preview[1], Some((42, Preview::Result(100))), "the crosshair says what it would do");
    hold(&mut w, buttons::ATTACK_OFF, 3);
    hold(&mut w, 0, 1);
    assert_eq!(w.snapshot().flights.len(), 1, "the word is in flight");
    let spent = INK_MAX - w.snapshot().players[0].ink;
    assert!(spent > 5.0, "Ink is spent: {spent}");
    // About a third of a second to fly five metres.
    let mut landed = None;
    for _ in 0..STEP_HZ {
        hold(&mut w, 0, 1);
        if let Some(l) = w.snapshot().landings.first() {
            landed = Some(l.clone());
            break;
        }
    }
    let l = landed.expect("it lands");
    assert_eq!(l.landing, Landing::Took { effect: "open".into(), strength: 1.0 });
    assert_eq!(words(&w), ["OPEN"]);
    assert_eq!(w.snapshot().targets[0].effects, ["open"]);
    // Ink comes back.
    let before = w.snapshot().players[0].ink;
    hold(&mut w, 0, STEP_HZ);
    assert!(w.snapshot().players[0].ink > before);
}

#[test]
fn a_tough_target_needs_a_charge() {
    let mut w = world(Resistance::Tough);
    hold(&mut w, 0, 2);
    assert_eq!(w.snapshot().players[0].preview[1], Some((42, Preview::NeedsCharge(0.5))));
    // A tap lands and is resisted.
    hold(&mut w, buttons::ATTACK_OFF, 3);
    hold(&mut w, 0, STEP_HZ);
    assert_eq!(words(&w), ["LOCKED"]);
    // A charge of a second reaches it.
    hold(&mut w, buttons::ATTACK_OFF, STEP_HZ);
    assert!(w.snapshot().players[0].charge[1] > 0.5);
    assert_eq!(w.snapshot().players[0].preview[1], Some((42, Preview::Result(100))));
    hold(&mut w, 0, STEP_HZ);
    assert_eq!(words(&w), ["OPEN"]);
}

#[test]
fn both_hands_write_one_word_together_and_it_lasts_longer() {
    let mut w = world(Resistance::Hard);
    let id = w.snapshot().players[0].id;
    w.hold_word(id, 0, "OPEN");
    hold(&mut w, buttons::ATTACK_MAIN | buttons::ATTACK_OFF, STEP_HZ * 3 / 4);
    assert!(w.snapshot().players[0].charge[2] > 0.5, "both hands charge together");
    hold(&mut w, 0, STEP_HZ);
    assert_eq!(words(&w), ["OPEN"], "half a charge from both hands reaches a hard target");
}

#[test]
fn a_heating_word_makes_its_target_hot_until_it_fades() {
    let mut w = world(Resistance::Plain);
    let id = w.snapshot().players[0].id;
    w.hold_word(id, 1, "BURNING");
    hold(&mut w, 0, 1);
    hold(&mut w, buttons::ATTACK_OFF, 3);
    hold(&mut w, 0, STEP_HZ);
    assert_eq!(words(&w), ["LOCKED", "BURNING"], "a word of another class joins the Inscription");
    let heat = w.snapshot().heat.iter().find(|(i, _)| *i == 42).map(|(_, h)| *h);
    assert!(heat.is_some_and(|h| h > 0.5), "{heat:?}");
    // A minute later, it has faded and the door is cool.
    hold(&mut w, 0, 50 * STEP_HZ);
    assert_eq!(words(&w), ["LOCKED"]);
    assert!(w.snapshot().heat.iter().all(|(i, _)| *i != 42));
}

#[test]
fn the_off_hand_cycles_through_the_words_known() {
    let mut w = world(Resistance::Plain);
    let id = w.snapshot().players[0].id;
    w.learn(id, "BURNING");
    hold(&mut w, 0, 1);
    assert_eq!(w.snapshot().players[0].holding[1], Holding::Word(1), "a word just learned settles into the writing hand");
    hold(&mut w, buttons::NEXT_WORD, 1);
    assert_eq!(w.snapshot().players[0].holding[1], Holding::Word(0));
    hold(&mut w, 0, 1);
    hold(&mut w, buttons::NEXT_WORD, 1);
    assert_eq!(w.snapshot().players[0].holding[1], Holding::Word(1));
}
