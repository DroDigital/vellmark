//! A merchant: a stock drawn from the merchants' list, a purse, and both made new each
//! morning. A player buys at an item's worth and sells at a share of it, and neither side
//! can pay more than it has.

use vellmark_content::LootTable;

use crate::items::{generate, Item};

/// Things in stock after each morning's restock.
pub const STOCK: usize = 10;
/// Marks in the purse each morning.
pub const PURSE: u32 = 600;
/// What a merchant pays for a thing, as a share of its worth.
pub const SELL_SHARE: f32 = 0.4;
/// The hour the morning's restock comes in.
pub const DAWN: f32 = 6.0;

#[derive(Clone, Debug, PartialEq)]
pub struct Merchant {
    /// Permanent: the merchant's ID.
    pub id: u64,
    pub stock: Vec<Item>,
    pub purse: u32,
    /// The morning its stock is from.
    pub day: u32,
}

/// Why a trade did not happen.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Refusal {
    /// There is no such thing to trade.
    Nothing,
    /// The buyer cannot pay.
    TooDear,
    /// The merchant's purse cannot pay.
    PurseEmpty,
    /// It is held or worn: put it away first.
    InUse,
}

impl Merchant {
    pub fn new(table: &LootTable, id: u64, day: u32) -> Self {
        let mut m = Merchant { id, stock: Vec::new(), purse: 0, day };
        m.restock(table, day);
        m
    }

    /// The morning's stock and purse, the same for the same merchant on the same day.
    pub fn restock(&mut self, table: &LootTable, day: u32) {
        self.day = day;
        self.purse = PURSE;
        let base = self.id.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ (day as u64).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        self.stock = (0..STOCK as u64).filter_map(|k| generate(table, "merchant", base.wrapping_add(k.wrapping_mul(0x94d0_49bb_1331_11eb)), &[])).collect();
    }

    /// What it asks for a thing in its stock, and what it would pay for one.
    pub fn asks(&self, table: &LootTable, item: &Item) -> u32 {
        item.value(table)
    }

    pub fn pays(&self, table: &LootTable, item: &Item) -> u32 {
        (item.value(table) as f32 * SELL_SHARE).floor() as u32
    }
}
