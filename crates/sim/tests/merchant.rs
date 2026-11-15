//! The merchant through `World`: the stock is the same all day and new each morning;
//! buying costs an item's worth and selling earns a share; neither side pays more than it
//! has; what is held cannot be sold until it is put away; and nothing Illuminated is sold.

use glam::Vec3;
use vellmark_content::{LootMaterial, LootSource, LootTable, LootType};
use vellmark_sim::items::{Item, Tier};
use vellmark_sim::merchant::{Refusal, PURSE, SELL_SHARE, STOCK};
use vellmark_sim::{PlayerInput, World, WorldConfig, STEP_HZ};

const MERCHANT: u64 = 77;

fn table() -> LootTable {
    LootTable {
        materials: vec![LootMaterial { key: "iron".into(), damage: 1.0, armour: 1.0, weight: 1.0, value: 1.0, trait_key: "honest".into() }],
        types: vec![LootType { key: "sword".into(), slot: "main".into(), damage: 12.0, armour: 0.0, weight: 3.0, value: 30 }],
        sources: vec![LootSource { key: "merchant".into(), tiers: [0.6, 0.32, 0.08, 0.0, 0.0], types: vec![(0, 1)], materials: vec![(0, 1)] }],
        ..Default::default()
    }
}

/// A world whose clock runs an hour a second, starting at five in the morning.
fn world() -> World {
    let mut w = World::new(WorldConfig { start_time_of_day: 5.0, hours_per_second: 1.0, ..Default::default() });
    w.add_player(Vec3::ZERO, 0.0);
    w.set_loot(table());
    w.add_merchant(MERCHANT);
    w
}

fn hours(w: &mut World, h: f32) {
    for _ in 0..(h * STEP_HZ as f32) as u32 {
        w.step(&[PlayerInput::default()]);
    }
}

#[test]
fn the_stock_is_new_each_morning_and_never_illuminated() {
    let mut w = world();
    let first = w.merchant(MERCHANT).unwrap().stock.clone();
    assert_eq!(first.len(), STOCK);
    hours(&mut w, 0.5);
    assert_eq!(w.merchant(MERCHANT).unwrap().stock, first, "the same before dawn");
    hours(&mut w, 1.0);
    let second = w.merchant(MERCHANT).unwrap().stock.clone();
    assert_ne!(second, first, "new at dawn");
    assert!(second.iter().all(|i| i.tier < Tier::Illuminated));
}

#[test]
fn buying_and_selling_move_items_and_marks_and_stop_at_what_each_side_has() {
    let mut w = world();
    let me = w.snapshot().players[0].id;
    let price = w.merchant(MERCHANT).unwrap().stock[0].value(&table());
    assert_eq!(w.buy(me, MERCHANT, 0), Err(Refusal::TooDear), "no Marks");
    w.give_marks(me, price + 5);
    assert_eq!(w.buy(me, MERCHANT, 0), Ok(()));
    assert_eq!(w.inventory(me).unwrap().marks, 5);
    assert_eq!(w.inventory(me).unwrap().items.len(), 1);
    assert_eq!(w.merchant(MERCHANT).unwrap().stock.len(), STOCK - 1);
    // Held, it cannot be sold.
    w.equip(me, 0);
    assert_eq!(w.sell(me, MERCHANT, 0), Err(Refusal::InUse));
    // A second thing, sold for a share of its worth.
    let spare = Item::plain(&table(), "sword", "iron", 5).unwrap();
    let i = w.give_item(me, spare.clone()).unwrap();
    assert_eq!(w.sell(me, MERCHANT, i), Ok(()));
    assert_eq!(w.inventory(me).unwrap().marks, 5 + (spare.value(&table()) as f32 * SELL_SHARE) as u32);
    // The purse runs out.
    let costly = Item { tier: Tier::Engrossed, ..Item::plain(&table(), "sword", "iron", 6).unwrap() };
    let mut refused = None;
    for k in 0..20 {
        let i = w.give_item(me, Item { id: 100 + k, ..costly.clone() }).unwrap();
        if let Err(r) = w.sell(me, MERCHANT, i) {
            refused = Some(r);
            break;
        }
    }
    assert_eq!(refused, Some(Refusal::PurseEmpty));
    assert!(w.merchant(MERCHANT).unwrap().purse < (costly.value(&table()) as f32 * SELL_SHARE) as u32);
    let _ = PURSE;
}
