//! Steel, through `World::step` as a bot drives it: the tap chain, the heavy attack and
//! its Stamina, Ink from hits, sneak attacks, staggers, the block and the shield bash, the
//! sidestep's cost, and close reading's drain.

use glam::Vec3;
use vellmark_sim::melee::{Creature, SwingKind, WeaponKind, SIDESTEP_COST, STAMINA_MAX};
use vellmark_sim::{buttons, PlayerInput, World, WorldConfig, STEP_HZ};

const POST: u64 = 9;

/// A player facing north with a sword, and a creature 1.5 m before them.
fn world() -> World {
    let mut w = World::new(WorldConfig::default());
    let id = w.add_player(Vec3::ZERO, 0.0);
    w.arm(id, WeaponKind::Sword, false);
    w.add_creature(Creature::new(POST, Vec3::new(0.0, 0.0, -1.5), 0.4, 1.8, 200.0, 60.0));
    w
}

fn go(w: &mut World, steps: u32, buttons: u32) {
    for _ in 0..steps {
        w.step(&[PlayerInput { buttons, ..Default::default() }]);
    }
}

fn tap(w: &mut World, button: u32) {
    go(w, 3, button);
    go(w, 1, 0);
}

fn post(w: &World) -> Creature {
    w.creatures().iter().find(|c| c.id == POST).unwrap().clone()
}

/// Every blow that landed while stepping, with the swing it came from.
fn swing_and_collect(w: &mut World, steps: u32, buttons: u32, hits: &mut Vec<(SwingKind, f32, bool)>) {
    for _ in 0..steps {
        w.step(&[PlayerInput { buttons, ..Default::default() }]);
        hits.extend(w.snapshot().hits.iter().map(|h| (h.kind, h.damage, h.sneak)));
    }
}

#[test]
fn taps_make_a_chain_of_alternating_swings_that_each_strike_once() {
    let mut w = world();
    let mut hits = Vec::new();
    // Three taps, each made as the swing before ends.
    for _ in 0..3 {
        swing_and_collect(&mut w, 3, buttons::ATTACK_MAIN, &mut hits);
        swing_and_collect(&mut w, STEP_HZ / 2, 0, &mut hits);
    }
    let kinds: Vec<SwingKind> = hits.iter().map(|h| h.0).collect();
    assert_eq!(
        kinds,
        [SwingKind::Light { from_left: false }, SwingKind::Light { from_left: true }, SwingKind::Light { from_left: false }],
        "right, left, right, each striking once"
    );
    assert!(hits[0].2, "the first blow found it unaware: a sneak attack");
    assert!(!hits[1].2, "after that it knows");
    assert!(post(&w).health < 200.0 - 12.0 * 4.0, "it was hurt: {}", post(&w).health);
}

#[test]
fn a_tap_during_a_swing_follows_it() {
    let mut w = world();
    let mut hits = Vec::new();
    swing_and_collect(&mut w, 3, buttons::ATTACK_MAIN, &mut hits);
    swing_and_collect(&mut w, 2, 0, &mut hits);
    swing_and_collect(&mut w, 3, buttons::ATTACK_MAIN, &mut hits);
    swing_and_collect(&mut w, STEP_HZ * 2, 0, &mut hits);
    assert_eq!(hits.len(), 2, "the second tap waited for the first swing to end: {hits:?}");
}

#[test]
fn a_held_attack_is_one_heavy_blow_that_costs_stamina_and_staggers() {
    let mut w = world();
    // Make it aware first, so that the blow is not a sneak attack.
    w.creatures_mut()[0].aware = true;
    let mut hits = Vec::new();
    swing_and_collect(&mut w, STEP_HZ / 2, buttons::ATTACK_MAIN, &mut hits);
    assert!(w.snapshot().players[0].drawing.is_some(), "drawn back while held");
    swing_and_collect(&mut w, STEP_HZ, 0, &mut hits);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].0, SwingKind::Heavy);
    assert!(hits[0].1 > 20.0, "heavy: {}", hits[0].1);
    let s = w.snapshot();
    assert!(s.players[0].stamina < STAMINA_MAX - 15.0, "it cost Stamina: {}", s.players[0].stamina);
    assert!(post(&w).staggered > 0.0 || post(&w).poise < 60.0, "it shook the creature's poise");
    // Stamina comes back.
    go(&mut w, STEP_HZ * 3, 0);
    assert_eq!(w.snapshot().players[0].stamina, STAMINA_MAX);
}

#[test]
fn hits_give_ink() {
    let mut w = world();
    w.creatures_mut()[0].aware = true;
    // Spend some Ink first, so there is room for it to come back.
    let id = w.snapshot().players[0].id;
    w.set_ink(id, 40.0);
    tap(&mut w, buttons::ATTACK_MAIN);
    go(&mut w, 10, 0);
    let ink = w.snapshot().players[0].ink;
    assert!(ink > 44.0, "a sword's hit returns Ink: {ink}");
}

#[test]
fn nothing_in_reach_is_struck() {
    let mut w = world();
    w.creatures_mut()[0].feet.z = -4.0;
    let mut hits = Vec::new();
    swing_and_collect(&mut w, 3, buttons::ATTACK_MAIN, &mut hits);
    swing_and_collect(&mut w, STEP_HZ, 0, &mut hits);
    assert!(hits.is_empty());
}

#[test]
fn the_free_hand_blocks_and_a_shield_takes_more() {
    let blocked = |shield: bool| {
        let mut w = world();
        let id = w.snapshot().players[0].id;
        w.arm(id, WeaponKind::Sword, shield);
        go(&mut w, 3, buttons::ATTACK_OFF);
        assert!(w.snapshot().players[0].blocking);
        w.hurt_player(id, 20.0, Vec3::new(0.0, 1.0, -2.0))
    };
    let (open, weapon, shield) = {
        let mut w = world();
        let id = w.snapshot().players[0].id;
        (w.hurt_player(id, 20.0, Vec3::new(0.0, 1.0, -2.0)), blocked(false), blocked(true))
    };
    assert_eq!(open, 20.0);
    assert!(weapon < 10.0 && shield < weapon, "a block takes most, a shield more: {weapon} {shield}");
    // A blow from behind is not blocked.
    let mut w = world();
    let id = w.snapshot().players[0].id;
    go(&mut w, 3, buttons::ATTACK_OFF);
    assert_eq!(w.hurt_player(id, 20.0, Vec3::new(0.0, 1.0, 2.0)), 20.0);
}

#[test]
fn a_hand_that_holds_a_word_does_not_block() {
    let mut w = world();
    let id = w.snapshot().players[0].id;
    w.set_word_rules(vellmark_sim::emendation::WordRules {
        classes: vec!["door".into()],
        words: vec![vellmark_sim::emendation::WordRule {
            spelling: "OPEN".into(),
            class: "Access".into(),
            ink: 8.0,
            flight_speed: 14.0,
            outcomes: vec![vellmark_sim::emendation::RuleOutcome::Refusal(1)],
            weapon: vellmark_sim::emendation::RuleOutcome::Refusal(2),
            typo: "OVEN".into(),
            typo_outcomes: vec![vellmark_sim::emendation::RuleOutcome::Refusal(3)],
        }],
        compounds: vec![],
    });
    w.hold_word(id, 1, "OPEN");
    go(&mut w, 3, buttons::ATTACK_OFF);
    assert!(!w.snapshot().players[0].blocking);
}

#[test]
fn a_shield_bashes_once_from_a_block_and_staggers() {
    let mut w = world();
    let id = w.snapshot().players[0].id;
    w.arm(id, WeaponKind::Sword, true);
    w.creatures_mut()[0].aware = true;
    go(&mut w, STEP_HZ / 4, buttons::ATTACK_OFF);
    let mut hits = Vec::new();
    swing_and_collect(&mut w, 3, buttons::ATTACK_OFF | buttons::ATTACK_MAIN, &mut hits);
    swing_and_collect(&mut w, STEP_HZ / 2, buttons::ATTACK_OFF, &mut hits);
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].0, SwingKind::Bash);
    assert!(post(&w).staggered > 0.0, "the bash staggers");
    // Only once from the same block.
    swing_and_collect(&mut w, 3, buttons::ATTACK_OFF | buttons::ATTACK_MAIN, &mut hits);
    swing_and_collect(&mut w, STEP_HZ / 2, buttons::ATTACK_OFF, &mut hits);
    assert_eq!(hits.len(), 1, "once from a block");
}

#[test]
fn a_dagger_sneak_attack_hits_hardest() {
    let first_blow = |weapon: WeaponKind| {
        let mut w = world();
        let id = w.snapshot().players[0].id;
        w.arm(id, weapon, false);
        let mut hits = Vec::new();
        swing_and_collect(&mut w, 3, buttons::ATTACK_MAIN, &mut hits);
        swing_and_collect(&mut w, STEP_HZ, 0, &mut hits);
        hits[0].1
    };
    assert!(first_blow(WeaponKind::Dagger) > first_blow(WeaponKind::Sword));
}

#[test]
fn the_sidestep_costs_stamina_and_needs_it() {
    let mut w = world();
    go(&mut w, 2, buttons::SIDESTEP);
    go(&mut w, STEP_HZ / 2, 0);
    let after = w.snapshot().players[0].stamina;
    assert!((STAMINA_MAX - after - SIDESTEP_COST).abs() < 1.0, "one sidestep: {after}");
    // Sidestep until tired: it stops answering.
    let mut moved = Vec::new();
    for _ in 0..8 {
        let before = w.snapshot().players[0].feet;
        go(&mut w, 2, buttons::SIDESTEP);
        go(&mut w, 25, 0);
        moved.push(w.snapshot().players[0].feet.distance(before));
    }
    assert!(moved.iter().any(|d| *d < 0.1), "with no Stamina, no sidestep: {moved:?}");
}

#[test]
fn close_reading_drains_stamina_and_ends_when_it_is_gone() {
    let mut w = world();
    go(&mut w, STEP_HZ, buttons::READ);
    let s = w.snapshot();
    assert_eq!(s.players[0].reading, vellmark_sim::reading::ReadingSnap::Close);
    assert!(s.players[0].stamina < STAMINA_MAX - 5.0, "{}", s.players[0].stamina);
    go(&mut w, STEP_HZ * 15, buttons::READ);
    assert_eq!(w.snapshot().players[0].reading, vellmark_sim::reading::ReadingSnap::None, "out of Stamina, close reading ends");
}
