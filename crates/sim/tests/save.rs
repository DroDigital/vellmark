//! Saving and loading: a saved game survives being written and read, and a world played
//! a while, saved, and loaded into a fresh world of the same scene is the same world: the
//! player, their pack and words, the clock, a broken thing, a moved prop, a fallen creature
//! that stays fallen, an item lying where it fell, and a merchant's stock.

use std::sync::Arc;

use glam::{Affine3A, Vec2, Vec3};
use vellmark_content::{LootMaterial, LootSource, LootTable, LootType};
use vellmark_sim::character::HeightPatch;
use vellmark_sim::emendation::{RuleOutcome, WordRule, WordRules};
use vellmark_sim::items::Item;
use vellmark_sim::save::SaveGame;
use vellmark_sim::{buttons, BreakableThing, PlayerInput, World, WorldConfig, STEP_HZ};

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
        vec![HeightPatch { origin: Vec2::new(-32.0, -32.0), spacing: 1.0, samples: 65, heights: vec![0.0; 65 * 65] }]
    }
}

fn table() -> LootTable {
    LootTable {
        materials: vec![LootMaterial { key: "iron".into(), damage: 1.0, armour: 1.0, weight: 1.0, value: 1.0, trait_key: "honest".into() }],
        types: vec![LootType { key: "sword".into(), slot: "main".into(), damage: 12.0, armour: 0.0, weight: 3.0, value: 30 }],
        sources: vec![
            LootSource { key: "enemy".into(), tiers: [0.5, 0.5, 0.0, 0.0, 0.0], types: vec![(0, 1)], materials: vec![(0, 1)] },
            LootSource { key: "merchant".into(), tiers: [0.5, 0.5, 0.0, 0.0, 0.0], types: vec![(0, 1)], materials: vec![(0, 1)] },
        ],
        ..Default::default()
    }
}

fn rules() -> WordRules {
    WordRules {
        classes: vec!["creature".into()],
        words: vec![WordRule { spelling: "COLD".into(), class: "Temperature".into(), ink: 8.0, flight_speed: 20.0, outcomes: vec![RuleOutcome::Refusal(1)], weapon: RuleOutcome::Refusal(2), typo: "GOLD".into(), typo_outcomes: vec![RuleOutcome::Refusal(3)] }],
        compounds: vec![],
    }
}

/// The same scene, freshly loaded: a crate, a breakable stool, a creature, a merchant.
fn fresh() -> World {
    let mut w = World::new(WorldConfig::default());
    w.add_player(Vec3::ZERO, 0.0);
    w.set_ground(Arc::new(Flat));
    let cube = |h: f32| vec![(Vec3::splat(-h), Vec3::splat(h), 1)];
    w.set_scene(
        Vec::new(),
        vec![
            BreakableThing { id: 5, to_world: Affine3A::from_translation(Vec3::new(2.0, 0.6, -2.0)), boxes: cube(0.25), mass: 8.0, pieces: Vec::new() },
            BreakableThing { id: 6, to_world: Affine3A::from_translation(Vec3::new(-2.0, 0.3, -2.0)), boxes: cube(0.3), mass: 0.0, pieces: Vec::new() },
        ],
    );
    w.set_word_rules(rules());
    w.set_loot(table());
    let me = w.snapshot().players[0].id;
    let sword = w.give_item(me, Item::plain(&table(), "sword", "iron", 1).unwrap()).unwrap();
    w.equip(me, sword);
    let mut c = vellmark_sim::melee::Creature::new(70, Vec3::new(0.0, 0.0, -1.6), 0.4, 1.8, 5.0, 100.0);
    c.aware = true;
    w.add_creature(c);
    w.add_creature(vellmark_sim::melee::Creature::new(71, Vec3::new(8.0, 0.0, 8.0), 0.4, 1.8, 50.0, 100.0));
    w.add_merchant(9);
    w
}

#[test]
fn a_played_world_saved_and_loaded_is_the_same_world() {
    let mut w = fresh();
    let me = w.snapshot().players[0].id;
    // Play: fell the creature in front (it drops an item), learn a word, earn Marks,
    // break the stool, move on a little.
    for b in [buttons::ATTACK_MAIN; 3].into_iter().chain([0; 40]) {
        w.step(&[PlayerInput { buttons: b, ..Default::default() }]);
    }
    w.learn(me, "COLD");
    w.give_marks(me, 120);
    w.break_thing(6, Vec3::new(-2.0, 1.0, -1.0));
    for _ in 0..STEP_HZ * 2 {
        w.step(&[PlayerInput { move_x: 0.5, ..Default::default() }]);
    }
    assert_eq!(w.dropped().len(), 1);
    let saved = w.save();
    // Written and read back, it is the same.
    let bytes = saved.to_bytes();
    assert_eq!(SaveGame::from_bytes(&bytes).unwrap(), saved);
    // Loaded into a fresh world of the same scene, it saves the same again.
    let mut again = fresh();
    again.load(&SaveGame::from_bytes(&bytes).unwrap());
    let resaved = again.save();
    assert_eq!(resaved.players, saved.players);
    assert_eq!(resaved.broken, saved.broken);
    assert_eq!(resaved.creatures, saved.creatures, "the fallen creature stays fallen");
    assert_eq!(resaved.dropped, saved.dropped);
    assert_eq!(resaved.merchants, saved.merchants);
    assert_eq!((resaved.time_of_day, resaved.day, resaved.seconds), (saved.time_of_day, saved.day, saved.seconds));
    for ((a, pa, _), (b, pb, _)) in resaved.props.iter().zip(&saved.props) {
        assert_eq!(a, b);
        assert!(pa.distance(*pb) < 0.02, "the crate where it rested: {pa} against {pb}");
    }
    assert!(again.snapshot().broken.contains(&6), "the stool is broken still");
    assert!(!again.snapshot().letters.is_empty(), "and its letters lie about");
}

#[test]
fn a_damaged_save_is_refused_not_believed() {
    let w = fresh();
    let bytes = w.save().to_bytes();
    assert!(SaveGame::from_bytes(&bytes[..bytes.len() - 3]).is_err());
    assert!(SaveGame::from_bytes(b"VLMKSAV9........").is_err());
}
