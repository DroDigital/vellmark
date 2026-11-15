//! What a player carries and wears, and what it does for them. Items come from the
//! generator (`crate::items`); equipping one changes the weapon in hand and the numbers
//! steel and words read, through [`Gear`].

use vellmark_content::LootTable;

use crate::items::Item;
use crate::melee::WeaponKind;

/// What a player's equipment adds up to. Multipliers are 1 when nothing changes them;
/// additions are 0.
#[derive(Clone, Debug, PartialEq)]
pub struct Gear {
    /// The weapon's damage against its type's own.
    pub damage: f32,
    /// Swings come faster by this share.
    pub speed: f32,
    /// Metres more reach.
    pub reach: f32,
    pub stagger: f32,
    pub heavy_stagger: f32,
    pub ink_per_hit: f32,
    /// More damage to a staggered creature, an unaware one, and the first blow on one
    /// unhurt, as shares.
    pub staggered_damage: f32,
    pub unaware_damage: f32,
    pub first_blow: f32,
    pub block_cost: f32,
    pub stamina: f32,
    /// Armour worn: blows lose `armour / (armour + 100)` of themselves.
    pub armour: f32,
    /// Words cost this much of their Ink, and Ink comes back this much faster.
    pub word_cost: f32,
    pub ink_refill: f32,
    pub ink_per_kill: f32,
    /// A word written on the weapon lasts this much longer.
    pub word_on_weapon: f32,
    /// The held weapon's word clause: the word, and what it does.
    pub word_clause: Option<(String, String)>,
}

impl Default for Gear {
    fn default() -> Self {
        Self {
            damage: 1.0,
            speed: 0.0,
            reach: 0.0,
            stagger: 1.0,
            heavy_stagger: 1.0,
            ink_per_hit: 1.0,
            staggered_damage: 0.0,
            unaware_damage: 0.0,
            first_blow: 0.0,
            block_cost: 1.0,
            stamina: 1.0,
            armour: 0.0,
            word_cost: 1.0,
            ink_refill: 1.0,
            ink_per_kill: 0.0,
            word_on_weapon: 0.0,
            word_clause: None,
        }
    }
}

/// Where an item is held or worn.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Slot {
    Main,
    Off,
    Body,
    Head,
    Trinket(usize),
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Inventory {
    pub items: Vec<Item>,
    /// What is held and worn, by index into `items`.
    pub main: Option<usize>,
    pub off: Option<usize>,
    pub body: Option<usize>,
    pub head: Option<usize>,
    pub trinkets: [Option<usize>; 2],
    /// Money, in Marks.
    pub marks: u32,
    /// Pairings to swap to at once: an item for the main hand, and for the other either
    /// an item or a word, by its index in the rules.
    pub favourites: Vec<(Option<usize>, Favourite)>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Favourite {
    Empty,
    Item(usize),
    Word(usize),
}

/// The weapon kind of an item's type, if it is a weapon.
pub fn weapon_kind(table: &LootTable, item: &Item) -> Option<WeaponKind> {
    Some(match table.types.get(item.kind)?.key.as_str() {
        "sword" => WeaponKind::Sword,
        "axe" => WeaponKind::Axe,
        "mace" => WeaponKind::Mace,
        "dagger" => WeaponKind::Dagger,
        "bow" => WeaponKind::Bow,
        "staff" => WeaponKind::Staff,
        _ => return None,
    })
}

impl Inventory {
    pub fn equipped(&self) -> impl Iterator<Item = (Slot, &Item)> {
        let slots = [(Slot::Main, self.main), (Slot::Off, self.off), (Slot::Body, self.body), (Slot::Head, self.head), (Slot::Trinket(0), self.trinkets[0]), (Slot::Trinket(1), self.trinkets[1])];
        slots.into_iter().filter_map(move |(s, i)| i.and_then(|i| self.items.get(i)).map(|item| (s, item)))
    }

    /// Puts on or takes up an item, in the slot its type wears it in. A two-handed one
    /// frees the other hand. Returns the slot.
    pub fn equip(&mut self, table: &LootTable, index: usize) -> Option<Slot> {
        let item = self.items.get(index)?;
        let slot = match item.slot(table) {
            "main" => Slot::Main,
            "both" => {
                self.off = None;
                Slot::Main
            }
            "off" => Slot::Off,
            "body" => Slot::Body,
            "head" => Slot::Head,
            _ => Slot::Trinket(if self.trinkets[0].is_none() { 0 } else { 1 }),
        };
        // A one-handed thing in the other hand while a two-handed one is held: that goes.
        if slot == Slot::Off && self.main.and_then(|m| self.items.get(m)).is_some_and(|m| m.slot(table) == "both") {
            self.main = None;
        }
        match slot {
            Slot::Main => self.main = Some(index),
            Slot::Off => self.off = Some(index),
            Slot::Body => self.body = Some(index),
            Slot::Head => self.head = Some(index),
            Slot::Trinket(k) => self.trinkets[k] = Some(index),
        }
        Some(slot)
    }

    /// Takes an item into the pack. Returns its index.
    pub fn take(&mut self, item: Item) -> usize {
        self.items.push(item);
        self.items.len() - 1
    }

    /// Lets go of an item: it leaves the pack, and whatever pointed past it moves down.
    pub fn remove(&mut self, index: usize) -> Option<Item> {
        if index >= self.items.len() {
            return None;
        }
        let item = self.items.remove(index);
        let fix = |slot: &mut Option<usize>| {
            *slot = match *slot {
                Some(i) if i == index => None,
                Some(i) if i > index => Some(i - 1),
                other => other,
            }
        };
        fix(&mut self.main);
        fix(&mut self.off);
        fix(&mut self.body);
        fix(&mut self.head);
        for t in &mut self.trinkets {
            fix(t);
        }
        // A pairing that named the thing is forgotten; the rest follow it down the list.
        self.favourites.retain(|(main, off)| *main != Some(index) && *off != Favourite::Item(index));
        for (main, off) in &mut self.favourites {
            fix(main);
            if let Favourite::Item(i) = off {
                if *i > index {
                    *i -= 1;
                }
            }
        }
        Some(item)
    }

    /// What everything held and worn adds up to.
    pub fn gear(&self, table: &LootTable) -> Gear {
        let mut g = Gear::default();
        for (slot, item) in self.equipped() {
            let e = |effect: &str| item.effect(table, effect);
            g.armour += item.armour(table);
            g.stamina *= 1.0 + e("stamina");
            g.word_cost *= 1.0 + e("word_cost");
            g.ink_refill *= 1.0 + e("ink_refill");
            g.ink_per_kill += e("ink_per_kill");
            g.block_cost *= 1.0 + e("block_cost");
            if slot == Slot::Main {
                let base = table.types[item.kind].damage.max(1e-3);
                g.damage = item.damage(table) / base;
                g.speed += e("speed");
                g.reach += e("reach");
                g.stagger *= 1.0 + e("stagger");
                g.ink_per_hit *= 1.0 + e("ink_per_hit");
                g.staggered_damage += e("staggered_damage");
                g.unaware_damage += e("unaware_damage");
                g.first_blow += e("first_blow");
                g.word_on_weapon += e("word_on_weapon");
                g.word_clause = item.word_clause.map(|w| (table.word_clauses[w].word.clone(), table.word_clauses[w].effect.clone()));
                // The material's trait, in the hand.
                match table.materials[item.material].trait_key.as_str() {
                    "keen" => g.heavy_stagger *= 1.3,
                    "living" => g.ink_refill *= 1.25,
                    _ => {}
                }
            }
        }
        g
    }
}
