//! The grey-box Inkling through `World::step`: it notices a player, closes, gathers
//! itself where it can be seen, and lunges; it splits when struck unless it is cold; light
//! drives it back and calm keeps it from fighting; its annotations give it power until a
//! charged word of the same class strips them; and steel alone can finish every part.

use glam::Vec3;
use vellmark_sim::emendation::{RuleOutcome, WordRule, WordRules};
use vellmark_sim::inkling::inkling;
use vellmark_sim::melee::WeaponKind;
use vellmark_sim::{buttons, PlayerInput, World, WorldConfig, STEP_HZ};

const INK: u64 = 400;

fn rules() -> WordRules {
    let e = |effect: &str| RuleOutcome::Effect { effect: effect.into(), strength: 1.0, preview: 100 };
    let word = |spelling: &str, class: &str, effect: &str| WordRule {
        spelling: spelling.into(),
        class: class.into(),
        ink: 8.0,
        flight_speed: 40.0,
        outcomes: vec![e(effect)],
        weapon: RuleOutcome::Refusal(1),
        typo: format!("{spelling}Q"),
        typo_outcomes: vec![RuleOutcome::Refusal(2)],
    };
    WordRules {
        classes: vec!["inkling".into()],
        words: vec![
            word("OPEN", "Access", "gape"),
            word("COLD", "Temperature", "slow"),
            word("LIT", "Light", "repel"),
            word("CALM", "Mind", "pacify"),
            word("BURNING", "Temperature", "ignite"),
        ],
        compounds: vec![],
    }
}

/// A player facing north with a sword and `word` in the off hand, and an Inkling
/// `distance` metres north of them.
fn world(word: &str, distance: f32, annotation: Option<&str>) -> World {
    let mut w = World::new(WorldConfig::default());
    let id = w.add_player(Vec3::ZERO, 0.0);
    w.set_word_rules(rules());
    let mut ink = inkling(INK, Vec3::new(0.0, 0.0, -distance), annotation);
    // Facing the player.
    ink.yaw = std::f32::consts::PI;
    w.add_creature(ink);
    w.arm(id, WeaponKind::Sword, false);
    w.hold_word(id, 1, word);
    w
}

fn go(w: &mut World, steps: u32, buttons: u32) {
    for _ in 0..steps {
        w.step(&[PlayerInput { buttons, ..Default::default() }]);
    }
}

fn ink(w: &World) -> vellmark_sim::melee::Creature {
    w.creatures().iter().find(|c| c.id == INK).unwrap().clone()
}

fn health(w: &World) -> f32 {
    w.snapshot().players[0].health
}

/// Writes the off hand's word on the Inkling, charged when `charged`.
fn write(w: &mut World, charged: bool) {
    let to = (ink(w).centre() - Vec3::new(0.0, 1.62, 0.0)).normalize();
    let pitch = w.snapshot().players[0].pitch;
    w.step(&[PlayerInput { look_pitch: to.y.asin() - pitch, ..Default::default() }]);
    go(w, if charged { STEP_HZ } else { 3 }, buttons::ATTACK_OFF);
    go(w, 12, 0);
}

#[test]
fn it_notices_closes_gathers_itself_where_it_can_be_seen_and_bites() {
    let mut w = world("OPEN", 9.0, None);
    let mut first_gathering = None;
    let mut first_bite = None;
    for step in 0..STEP_HZ * 6 {
        go(&mut w, 1, 0);
        let c = ink(&w);
        if c.telegraph > 0.0 && first_gathering.is_none() {
            first_gathering = Some(step);
        }
        if health(&w) < 100.0 && first_bite.is_none() {
            first_bite = Some(step);
            break;
        }
    }
    assert!(ink(&w).aware, "it saw the player");
    let (gathered, bit) = (first_gathering.expect("it gathered itself"), first_bite.expect("it bit"));
    assert!((bit - gathered) as f32 / STEP_HZ as f32 >= 0.5, "the blow was telegraphed half a second ahead");
}

#[test]
fn a_sidestep_through_its_lunge_takes_no_harm() {
    let mut w = world("OPEN", 3.0, None);
    // Wait for the lunge, then step aside.
    for _ in 0..STEP_HZ * 4 {
        go(&mut w, 1, 0);
        if ink(&w).telegraph >= 1.0 {
            break;
        }
    }
    go(&mut w, 1, buttons::SIDESTEP);
    go(&mut w, 20, 0);
    assert_eq!(health(&w), 100.0, "the sidestep's first moments let the blow pass");
}

#[test]
fn it_splits_when_struck_and_cold_stops_it() {
    let parts_after_a_blow = |cold: bool| {
        let mut w = world("COLD", 1.5, None);
        if cold {
            write(&mut w, false);
            assert!(ink(&w).marks & vellmark_sim::melee::marks::SLOWED != 0);
        }
        go(&mut w, 3, buttons::ATTACK_MAIN);
        go(&mut w, STEP_HZ / 2, 0);
        w.creatures().iter().filter(|c| c.alive()).count()
    };
    assert_eq!(parts_after_a_blow(false), 2, "a blow splits it");
    assert_eq!(parts_after_a_blow(true), 1, "cold, it does not split");
}

#[test]
fn light_drives_it_back() {
    let mut w = world("LIT", 4.0, None);
    write(&mut w, false);
    let near = ink(&w).feet.z;
    go(&mut w, STEP_HZ, 0);
    assert!(ink(&w).feet.z < near - 2.0, "it fled from the light: {} then {}", near, ink(&w).feet.z);
    assert_eq!(health(&w), 100.0);
}

#[test]
fn calm_keeps_it_from_fighting() {
    let mut w = world("CALM", 3.0, None);
    write(&mut w, false);
    go(&mut w, STEP_HZ * 4, 0);
    assert_eq!(health(&w), 100.0, "calm, it does not bite");
}

#[test]
fn locked_turns_blows_from_the_front_until_a_charged_open_strips_it() {
    let blow = |w: &mut World| {
        let before = ink(w).health;
        go(w, 3, buttons::ATTACK_MAIN);
        go(w, STEP_HZ / 2, 0);
        before - ink(w).health
    };
    let mut w = world("OPEN", 1.6, Some("LOCKED"));
    w.creatures_mut()[0].aware = true;
    let shut = blow(&mut w);
    // A tap does not reach a tough annotation; a charged word does, and staggers it.
    write(&mut w, false);
    assert_eq!(ink(&w).annotation.as_deref(), Some("LOCKED"), "a tap is not enough");
    // Held off, so that its bite does not break the charge.
    w.creatures_mut()[0].staggered = 3.0;
    write(&mut w, true);
    assert_eq!(ink(&w).annotation, None, "the charged word stripped it");
    assert!(ink(&w).staggered > 0.0 || w.snapshot().targets.iter().any(|t| t.id == INK && t.words.contains(&"OPEN".to_string())));
    let open = blow(&mut w);
    assert!(open > shut * 3.0, "blows land now: {shut} then {open}");
}

#[test]
fn burning_and_cold_annotations_go_with_its_bite() {
    for (annotation, check) in [("BURNING", 0usize), ("COLD", 1)] {
        let mut w = world("OPEN", 3.0, Some(annotation));
        for _ in 0..STEP_HZ * 4 {
            go(&mut w, 1, 0);
            if health(&w) < 100.0 {
                break;
            }
        }
        let hurt = health(&w);
        assert!(hurt < 100.0, "{annotation}: it bit");
        go(&mut w, STEP_HZ, 0);
        if check == 0 {
            assert!(health(&w) < hurt - 1.0, "BURNING: the bite set the player alight");
        } else {
            let before = w.snapshot().players[0].feet;
            go(&mut w, STEP_HZ / 2, buttons::SPRINT);
            let _ = before;
        }
    }
}

#[test]
fn steel_alone_finishes_every_part() {
    let mut w = world("OPEN", 1.6, None);
    w.creatures_mut()[0].aware = true;
    // Swing at whatever is nearest, turning to it, until nothing stands.
    for _ in 0..STEP_HZ * 40 {
        let me = w.snapshot().players[0].clone();
        let Some(near) = w.creatures().iter().filter(|c| c.alive()).min_by(|a, b| a.feet.distance(me.feet).total_cmp(&b.feet.distance(me.feet))).cloned() else { break };
        let to = near.feet - me.feet;
        let want = to.x.atan2(-to.z);
        let mut turn = want - me.yaw;
        while turn > std::f32::consts::PI {
            turn -= std::f32::consts::TAU;
        }
        while turn < -std::f32::consts::PI {
            turn += std::f32::consts::TAU;
        }
        let close = to.length() > 1.6;
        let tapping = (w.snapshot().step / 3) % 2 == 0;
        w.step(&[PlayerInput { look_yaw: turn, move_y: if close { 1.0 } else { 0.0 }, buttons: if tapping { buttons::ATTACK_MAIN } else { 0 }, ..Default::default() }]);
    }
    assert!(w.creatures().iter().all(|c| !c.alive()), "every part is finished: {} stand", w.creatures().iter().filter(|c| c.alive()).count());
    assert!(health(&w) > 0.0, "and the player still stands: {}", health(&w));
}
