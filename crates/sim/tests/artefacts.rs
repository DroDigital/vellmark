//! The Showcase's two Canonical items: Wick's Spade buries what it slays and puts the
//! loot straight in the pack; the Promised Lamp lights its wearer and warns of Inklings.

use glam::Vec3;
use vellmark_content::{LootCanonical, LootMaterial, LootSource, LootTable, LootType};
use vellmark_sim::items::{Item, Tier};
use vellmark_sim::melee::Creature;
use vellmark_sim::{buttons, PlayerInput, World, WorldConfig};

fn table() -> LootTable {
    let ty = |key: &str, slot: &str, damage: f32| LootType { key: key.into(), slot: slot.into(), damage, armour: 0.0, weight: 2.0, value: 20 };
    LootTable {
        materials: vec![LootMaterial { key: "iron".into(), damage: 1.0, armour: 1.0, weight: 1.0, value: 1.0, trait_key: "honest".into() }],
        types: vec![ty("axe", "main", 13.0), ty("charm", "trinket", 0.0)],
        sources: vec![LootSource { key: "enemy".into(), tiers: [1.0, 0.0, 0.0, 0.0, 0.0], types: vec![(0, 1)], materials: vec![(0, 1)] }],
        canonical: vec![
            LootCanonical { key: "wicks_spade".into(), kind: 0, material: 0, effect: "buries".into() },
            LootCanonical { key: "promised_lamp".into(), kind: 1, material: 0, effect: "lit".into() },
        ],
        ..Default::default()
    }
}

fn world() -> World {
    let mut w = World::new(WorldConfig::default());
    w.add_player(Vec3::ZERO, 0.0);
    w.set_loot(table());
    w
}

#[test]
fn wicks_spade_buries_what_it_slays_and_its_loot_is_in_the_pack() {
    let mut w = world();
    let me = w.snapshot().players[0].id;
    let spade = Item::canonical(&table(), "wicks_spade", 14).unwrap();
    assert_eq!(spade.tier, Tier::Canonical);
    let i = w.give_item(me, spade).unwrap();
    w.equip(me, i);
    let mut c = Creature::new(60, Vec3::new(0.0, 0.0, -1.6), 0.4, 1.8, 5.0, 100.0);
    c.aware = true;
    w.add_creature(c);
    for b in [buttons::ATTACK_MAIN; 3].into_iter().chain([0; 30]) {
        w.step(&[PlayerInput { buttons: b, ..Default::default() }]);
    }
    assert!(w.creatures().is_empty(), "it buried itself at once");
    assert!(w.dropped().is_empty(), "nothing is left lying");
    assert_eq!(w.inventory(me).unwrap().items.len(), 2, "its loot is in the pack");
}

#[test]
fn the_promised_lamp_lights_its_wearer_and_warns_of_inklings() {
    let mut w = world();
    let me = w.snapshot().players[0].id;
    let lamp = Item::canonical(&table(), "promised_lamp", 16).unwrap();
    let i = w.give_item(me, lamp).unwrap();
    w.equip(me, i);
    w.step(&[PlayerInput::default()]);
    assert!(w.snapshot().players[0].lit);
    assert!(!w.snapshot().players[0].warned);
    w.add_creature(vellmark_sim::inkling::inkling(61, Vec3::new(0.0, 0.0, -12.0), None));
    w.step(&[PlayerInput::default()]);
    assert!(w.snapshot().players[0].warned, "an Inkling within fifteen metres");
}
