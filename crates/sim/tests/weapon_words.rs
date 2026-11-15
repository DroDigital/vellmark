//! Words and steel together, through `World::step`: both hands write the off hand's word
//! on the held weapon, or are refused; what the weapon carries passes to what it strikes;
//! words on a creature open it for steel; and the bow draws, drops, strikes, and its
//! arrows are gathered again.

use std::sync::Arc;

use glam::{Vec2, Vec3};
use vellmark_sim::character::HeightPatch;
use vellmark_sim::emendation::{Landing, RuleOutcome, WordRule, WordRules};
use vellmark_sim::melee::{Creature, SwingKind, WeaponKind, QUIVER};
use vellmark_sim::{buttons, PlayerInput, World, WorldConfig, STEP_HZ};

const FOE: u64 = 21;

fn rules() -> WordRules {
    let e = |effect: &str| RuleOutcome::Effect { effect: effect.into(), strength: 1.0, preview: 100 };
    let word = |spelling: &str, class: &str, creature: RuleOutcome, weapon: RuleOutcome| WordRule {
        spelling: spelling.into(),
        class: class.into(),
        ink: 10.0,
        flight_speed: 30.0,
        outcomes: vec![creature, RuleOutcome::Refusal(5)],
        weapon,
        typo: format!("{}X", spelling),
        typo_outcomes: vec![RuleOutcome::Refusal(6), RuleOutcome::Refusal(6)],
    };
    WordRules {
        classes: vec!["creature".into(), "weapon".into()],
        words: vec![
            word("BURNING", "Temperature", e("ignite"), e("ignite")),
            word("COLD", "Temperature", e("slow"), e("frost")),
            word("OPEN", "Access", e("gape"), RuleOutcome::Refusal(7)),
            word("SHARP", "Size", RuleOutcome::Refusal(8), e("sharpen")),
        ],
        compounds: vec![],
    }
}

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
        vec![HeightPatch { origin: Vec2::new(-64.0, -64.0), spacing: 2.0, samples: 65, heights: vec![0.0; 65 * 65] }]
    }
}

/// A player facing north with a sword and `word` in the off hand, and an aware foe at
/// `distance` before them.
fn world(word: &str, distance: f32) -> World {
    let mut w = World::new(WorldConfig::default());
    let id = w.add_player(Vec3::ZERO, 0.0);
    w.set_ground(Arc::new(Flat));
    w.set_word_rules(rules());
    let mut foe = Creature::new(FOE, Vec3::new(0.0, 0.0, -distance), 0.4, 1.8, 300.0, 1000.0);
    foe.aware = true;
    w.add_creature(foe);
    w.arm(id, WeaponKind::Sword, false);
    w.hold_word(id, 1, word);
    w
}

fn go(w: &mut World, steps: u32, buttons: u32) -> Vec<vellmark_sim::melee::Hit> {
    let mut hits = Vec::new();
    for _ in 0..steps {
        w.step(&[PlayerInput { buttons, ..Default::default() }]);
        hits.extend(w.snapshot().hits.iter().cloned());
    }
    hits
}

fn both(w: &mut World) -> Vec<vellmark_sim::casting::CastEvent> {
    let mut landed = Vec::new();
    for b in [buttons::ATTACK_MAIN | buttons::ATTACK_OFF, 0] {
        for _ in 0..if b == 0 { 2 } else { 6 } {
            w.step(&[PlayerInput { buttons: b, ..Default::default() }]);
            landed.extend(w.snapshot().landings.iter().cloned());
        }
    }
    landed
}

fn foe(w: &World) -> Creature {
    w.creatures().iter().find(|c| c.id == FOE).unwrap().clone()
}

fn effects(w: &World) -> Vec<String> {
    w.snapshot().targets.iter().find(|t| t.id == FOE).unwrap().effects.clone()
}

#[test]
fn both_hands_write_the_word_on_the_weapon_and_it_passes_to_what_it_strikes() {
    let mut w = world("BURNING", 1.5);
    let landed = both(&mut w);
    assert_eq!(landed.len(), 1);
    assert_eq!(landed[0].landing, Landing::Took { effect: "ignite".into(), strength: 1.0 });
    assert!(w.snapshot().players[0].ink < 95.0, "it costs Ink");
    go(&mut w, STEP_HZ / 2, 0);
    let hits = go(&mut w, 3, buttons::ATTACK_MAIN);
    let hits = [hits, go(&mut w, STEP_HZ, 0)].concat();
    assert_eq!(hits.len(), 1);
    assert!(effects(&w).contains(&"ignite".to_string()), "the blade set it alight: {:?}", effects(&w));
    // It burns.
    let before = foe(&w).health;
    go(&mut w, STEP_HZ * 2, 0);
    assert!(foe(&w).health < before - 5.0, "burning hurts: {} then {}", before, foe(&w).health);
}

#[test]
fn a_word_that_means_nothing_on_a_weapon_is_refused() {
    let mut w = world("OPEN", 1.5);
    let landed = both(&mut w);
    assert_eq!(landed[0].landing, Landing::Refused(7));
    assert_eq!(w.snapshot().players[0].ink, vellmark_sim::emendation::INK_MAX, "a refusal costs nothing");
}

#[test]
fn a_sharp_blade_cuts_through_armour() {
    let blow = |sharp: bool| {
        let mut w = world("SHARP", 1.5);
        w.creatures_mut()[0].armour = 0.5;
        if sharp {
            both(&mut w);
            go(&mut w, STEP_HZ / 2, 0);
        }
        let hits = [go(&mut w, 3, buttons::ATTACK_MAIN), go(&mut w, STEP_HZ, 0)].concat();
        hits[0].damage
    };
    assert!(blow(true) > blow(false) * 2.0, "{} against {}", blow(true), blow(false));
}

/// Writes the off hand's word on the foe.
fn write_on_foe(w: &mut World) {
    let eye_to_foe = (foe(w).centre() - Vec3::new(0.0, 1.62, 0.0)).normalize();
    w.step(&[PlayerInput { look_pitch: eye_to_foe.y.asin(), ..Default::default() }]);
    go(w, 3, buttons::ATTACK_OFF);
    go(w, STEP_HZ, 0);
}

#[test]
fn open_gapes_a_foe_and_its_blows_do_half_again() {
    let first = |gape: bool| {
        let mut w = world("OPEN", 1.5);
        if gape {
            write_on_foe(&mut w);
            assert!(effects(&w).contains(&"gape".to_string()), "{:?}", effects(&w));
        }
        let hits = [go(&mut w, 3, buttons::ATTACK_MAIN), go(&mut w, STEP_HZ, 0)].concat();
        hits[0].damage
    };
    assert!((first(true) / first(false) - 1.5).abs() < 0.01);
}

#[test]
fn cold_slows_a_foe_and_a_heavy_blow_shatters_it() {
    let mut w = world("COLD", 1.5);
    write_on_foe(&mut w);
    assert!(effects(&w).contains(&"slow".to_string()));
    let hits = [go(&mut w, STEP_HZ / 2, buttons::ATTACK_MAIN), go(&mut w, STEP_HZ, 0)].concat();
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].kind, SwingKind::Heavy);
    assert!(hits[0].shattered, "the heavy blow shattered the frost");
    assert!(!effects(&w).contains(&"slow".to_string()), "and the frost is gone");
}

#[test]
fn an_arrow_drawn_full_flies_far_and_drops_and_one_barely_drawn_falls_short() {
    let landing = |draw_steps: u32| {
        let mut w = world("OPEN", 200.0);
        let id = w.snapshot().players[0].id;
        w.arm(id, WeaponKind::Bow, false);
        w.step(&[PlayerInput { look_pitch: 0.15, ..Default::default() }]);
        go(&mut w, draw_steps, buttons::ATTACK_MAIN);
        go(&mut w, 1, 0);
        assert_eq!(w.arrows().len(), 1, "an arrow is loosed");
        let stamina = w.snapshot().players[0].stamina;
        go(&mut w, STEP_HZ * 4, 0);
        let a = w.arrows()[0].clone();
        assert!(a.stuck, "it came down and stuck");
        (-a.at.z, stamina)
    };
    let (far, tired) = landing(STEP_HZ * 2);
    let (near, _) = landing(4);
    assert!(far > near * 3.0 && far > 80.0, "a full draw goes far: {far} m against {near} m");
    assert!(tired < 100.0 - 5.0, "a held draw drains Stamina: {tired}");
}

#[test]
fn an_arrow_strikes_a_foe_and_arrows_stuck_nearby_are_gathered() {
    let mut w = world("OPEN", 12.0);
    let id = w.snapshot().players[0].id;
    w.arm(id, WeaponKind::Bow, false);
    let to_foe = (foe(&w).centre() - Vec3::new(0.0, 1.62, 0.0)).normalize();
    w.step(&[PlayerInput { look_pitch: to_foe.y.asin() + 0.03, ..Default::default() }]);
    let mut hits = go(&mut w, STEP_HZ, buttons::ATTACK_MAIN);
    hits.extend(go(&mut w, STEP_HZ, 0));
    assert_eq!(hits.len(), 1, "the arrow struck the foe");
    assert!(hits[0].damage > 20.0);
    assert_eq!(w.snapshot().players[0].weapon, WeaponKind::Bow);
    // A tap looses a weak shot at the ground nearby, which is picked up again.
    w.step(&[PlayerInput { look_pitch: -0.9, ..Default::default() }]);
    go(&mut w, 2, buttons::ATTACK_MAIN);
    go(&mut w, STEP_HZ, 0);
    assert!(w.arrows().iter().any(|a| a.stuck), "{:?}", w.arrows());
    go(&mut w, 2, buttons::INTERACT);
    go(&mut w, 1, 0);
    assert!(w.arrows().is_empty());
    let snapshot_arrows = w.snapshot().arrows.len();
    assert_eq!(snapshot_arrows, 0);
    let _ = QUIVER;
}
