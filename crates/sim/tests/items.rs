//! Things found and carried, through `World::step`: a fallen creature leaves an item,
//! which is named in the gaze with how it compares, and taken up with Interact; better
//! steel strikes harder; armour takes its share of a blow; a favourite pairing is taken
//! up at once; and a word clause passes a word on.

use glam::Vec3;
use vellmark_content::{LootMaterial, LootModifier, LootSource, LootTable, LootType, LootWordClause};
use vellmark_sim::inventory::Favourite;
use vellmark_sim::items::{Item, Tier};
use vellmark_sim::melee::{Creature, WeaponKind};
use vellmark_sim::{buttons, PlayerInput, World, WorldConfig, STEP_HZ};

fn table() -> LootTable {
    let ty = |key: &str, slot: &str, damage: f32, armour: f32| LootType { key: key.into(), slot: slot.into(), damage, armour, weight: 2.0, value: 20 };
    LootTable {
        materials: vec![
            LootMaterial { key: "iron".into(), damage: 1.0, armour: 1.0, weight: 1.0, value: 1.0, trait_key: "honest".into() },
            LootMaterial { key: "marcher_steel".into(), damage: 1.2, armour: 1.2, weight: 1.0, value: 2.0, trait_key: "keen".into() },
        ],
        types: vec![ty("sword", "main", 12.0, 0.0), ty("bow", "both", 32.0, 0.0), ty("heavy_coat", "body", 0.0, 50.0)],
        bonuses: vec![LootModifier { key: "tally".into(), effect: "damage".into(), amount: 0.1 }],
        clauses: vec![LootModifier { key: "escrow".into(), effect: "ink_per_kill".into(), amount: 15.0 }],
        word_clauses: vec![LootWordClause { key: "without_prejudice".into(), word: "COLD".into(), effect: "word_passes".into() }],
        sources: vec![LootSource { key: "enemy".into(), tiers: [1.0, 0.0, 0.0, 0.0, 0.0], types: vec![(0, 1)], materials: vec![(1, 1)] }],
    }
}

fn world() -> World {
    let mut w = World::new(WorldConfig::default());
    let id = w.add_player(Vec3::ZERO, 0.0);
    w.set_loot(table());
    let sword = Item::plain(&table(), "sword", "iron", 1).unwrap();
    let i = w.give_item(id, sword).unwrap();
    assert!(w.equip(id, i));
    w
}

fn go(w: &mut World, steps: u32, buttons: u32) {
    for _ in 0..steps {
        w.step(&[PlayerInput { buttons, ..Default::default() }]);
    }
}

fn first_blow(w: &mut World) -> f32 {
    let mut c = Creature::new(50, Vec3::new(0.0, 0.0, -1.5), 0.4, 1.8, 500.0, 1000.0);
    c.aware = true;
    w.add_creature(c);
    let mut dealt = 0.0;
    for b in [buttons::ATTACK_MAIN; 3].into_iter().chain([0; 40]) {
        w.step(&[PlayerInput { buttons: b, ..Default::default() }]);
        dealt += w.snapshot().hits.iter().map(|h| h.damage).sum::<f32>();
    }
    dealt
}

#[test]
fn a_fallen_creature_leaves_an_item_that_is_named_in_the_gaze_and_taken_up() {
    let mut w = world();
    let mut c = Creature::new(50, Vec3::new(0.0, 0.0, -1.6), 0.4, 1.8, 5.0, 100.0);
    c.aware = true;
    w.add_creature(c);
    go(&mut w, 3, buttons::ATTACK_MAIN);
    go(&mut w, 30, 0);
    assert_eq!(w.dropped().len(), 1, "it left something");
    let at = w.dropped()[0].1;
    let to = (at + Vec3::Y * 0.1 - Vec3::new(0.0, 1.62, 0.0)).normalize();
    w.step(&[PlayerInput { look_pitch: to.y.asin() - w.snapshot().players[0].pitch, ..Default::default() }]);
    let (seen, held) = w.snapshot().players[0].item_in_gaze.clone().expect("it is named in the gaze");
    assert_eq!(seen.tier, Tier::Plain);
    assert!(seen.damage(&table()) > held.expect("compared with the held sword").damage(&table()), "Marcher Steel against iron");
    go(&mut w, 2, buttons::INTERACT);
    go(&mut w, 1, 0);
    assert!(w.dropped().is_empty());
    let me = w.snapshot().players[0].id;
    assert_eq!(w.inventory(me).unwrap().items.len(), 2, "it is in the pack");
}

#[test]
fn better_steel_strikes_harder_and_armour_takes_its_share() {
    let mut plain = world();
    let iron = first_blow(&mut plain);
    let mut better = world();
    let me = better.snapshot().players[0].id;
    let fine = Item { id: 9, kind: 0, material: 1, tier: Tier::Fair, bonuses: vec![0], clauses: vec![], word_clause: None, look: 0, made: 4100 };
    let i = better.give_item(me, fine).unwrap();
    better.equip(me, i);
    let steel = first_blow(&mut better);
    assert!((steel / iron - 1.2 * 1.1).abs() < 0.02, "Tally's Marcher Steel: {steel} against {iron}");
    // A heavy coat of 50 armour takes a third of a blow.
    let bare = plain.hurt_player(me, 30.0, Vec3::new(0.0, 1.0, 5.0));
    let coat = Item::plain(&table(), "heavy_coat", "iron", 7).unwrap();
    let i = plain.give_item(me, coat).unwrap();
    plain.equip(me, i);
    let worn = plain.hurt_player(me, 30.0, Vec3::new(0.0, 1.0, 5.0));
    assert!((worn / bare - 100.0 / 150.0).abs() < 0.01, "{worn} against {bare}");
}

#[test]
fn a_favourite_pairing_is_taken_up_at_once() {
    let mut w = world();
    let me = w.snapshot().players[0].id;
    let bow = Item::plain(&table(), "bow", "iron", 3).unwrap();
    let b = w.give_item(me, bow).unwrap();
    w.add_favourite(me, Some(b), Favourite::Empty);
    w.add_favourite(me, Some(0), Favourite::Empty);
    w.step(&[PlayerInput { favourite: 1, ..Default::default() }]);
    assert_eq!(w.snapshot().players[0].weapon, WeaponKind::Bow);
    w.step(&[PlayerInput { favourite: 2, ..Default::default() }]);
    assert_eq!(w.snapshot().players[0].weapon, WeaponKind::Sword);
}
