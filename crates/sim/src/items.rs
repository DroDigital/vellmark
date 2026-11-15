//! The item generator: from a seed and a source, the same item every time. It rolls the
//! tier from the source's chances, the type and material from its lists, and then what
//! the tier carries: a Fair item one bonus, an Endorsed one a clause, an Engrossed one a
//! clause and a bonus, an Illuminated one a clause, a word clause, and two bonuses. A
//! name is its parts, as string keys: a maker for its first bonus, the material, the
//! type, its clause, and its word clause.

use vellmark_content::LootTable;

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Tier {
    Plain,
    Fair,
    Endorsed,
    Engrossed,
    Illuminated,
    /// Never rolled: each has its one authored source.
    Canonical,
}

impl Tier {
    pub const ROLLED: [Tier; 5] = [Tier::Plain, Tier::Fair, Tier::Endorsed, Tier::Engrossed, Tier::Illuminated];

    pub fn key(self) -> &'static str {
        match self {
            Tier::Plain => "plain",
            Tier::Fair => "fair",
            Tier::Endorsed => "endorsed",
            Tier::Engrossed => "engrossed",
            Tier::Illuminated => "illuminated",
            Tier::Canonical => "canonical",
        }
    }

    /// What it carries: bonuses, clauses, and word clauses.
    pub fn carries(self) -> (usize, usize, usize) {
        match self {
            Tier::Plain | Tier::Canonical => (0, 0, 0),
            Tier::Fair => (1, 0, 0),
            Tier::Endorsed => (0, 1, 0),
            Tier::Engrossed => (1, 1, 0),
            Tier::Illuminated => (2, 1, 1),
        }
    }

    /// How much more it is worth than a Plain one.
    pub fn worth(self) -> f32 {
        match self {
            Tier::Plain => 1.0,
            Tier::Fair => 2.0,
            Tier::Endorsed => 4.0,
            Tier::Engrossed => 7.0,
            Tier::Illuminated => 12.0,
            Tier::Canonical => 20.0,
        }
    }
}

/// A thing that can be found, carried, worn, and sold.
#[derive(Clone, Debug, PartialEq)]
pub struct Item {
    /// Permanent: the seed it was made from.
    pub id: u64,
    /// Indices into the loot table's lists.
    pub kind: usize,
    pub material: usize,
    pub tier: Tier,
    pub bonuses: Vec<usize>,
    pub clauses: Vec<usize>,
    pub word_clause: Option<usize>,
    /// Which of its type's assembled looks it has.
    pub look: u32,
    /// The year it was made, for its history.
    pub made: u32,
    /// A Canonical item, by its index in the table's list: its own name and effect.
    pub canonical: Option<usize>,
    /// What it has done in its owner's hands: the creatures it has slain. A weapon, bow,
    /// staff, or shield prints it on itself.
    pub kills: u32,
}

/// An item's name, as string keys: each part, or none.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct NameParts {
    pub maker: Option<String>,
    pub material: String,
    pub kind: String,
    pub clause: Option<String>,
    pub word_clause: Option<String>,
}

/// Effects that only a held weapon can carry.
const WEAPON_ONLY: &[&str] = &["damage", "speed", "reach", "stagger", "ink_per_hit", "word_on_weapon", "first_blow", "staggered_damage", "unaware_damage"];

fn is_weapon(slot: &str) -> bool {
    matches!(slot, "main" | "both" | "off")
}

/// Deterministic random numbers from a seed.
struct Roll(u64);

impl Roll {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / (1u64 << 24) as f32
    }
    fn weighted(&mut self, list: &[(u32, u32)]) -> usize {
        let total: u32 = list.iter().map(|(_, w)| *w).sum();
        let mut pick = (self.unit() * total as f32) as u32;
        for (i, w) in list {
            if pick < *w {
                return *i as usize;
            }
            pick -= w;
        }
        list.last().map_or(0, |(i, _)| *i as usize)
    }
    /// `n` different indices below `len`, from those `allowed`.
    fn distinct(&mut self, n: usize, allowed: &[usize]) -> Vec<usize> {
        let mut pool = allowed.to_vec();
        let mut out = Vec::new();
        while out.len() < n && !pool.is_empty() {
            let k = (self.next() % pool.len() as u64) as usize;
            out.push(pool.swap_remove(k));
        }
        out
    }
}

/// Makes the item for `seed` from `source`. `known` are the words the finder owns: two
/// times in three, a word clause is for one of them.
pub fn generate(table: &LootTable, source: &str, seed: u64, known: &[String]) -> Option<Item> {
    let s = table.sources.iter().find(|s| s.key == source)?;
    let mut r = Roll(seed ^ 0x51ed_10a7_c0de_0001);
    let x = r.unit();
    let mut tier = Tier::Plain;
    let mut acc = 0.0;
    for (t, chance) in Tier::ROLLED.iter().zip(s.tiers) {
        acc += chance;
        if x < acc {
            tier = *t;
            break;
        }
        tier = *t;
    }
    let kind = r.weighted(&s.types);
    let material = r.weighted(&s.materials);
    let slot = table.types.get(kind)?.slot.clone();
    let weapon = is_weapon(&slot);
    let fits = |effect: &str| weapon || !WEAPON_ONLY.contains(&effect);
    let bonus_pool: Vec<usize> = (0..table.bonuses.len()).filter(|i| fits(&table.bonuses[*i].effect)).collect();
    let clause_pool: Vec<usize> = (0..table.clauses.len()).filter(|i| fits(&table.clauses[*i].effect)).collect();
    let (b, c, w) = tier.carries();
    let bonuses = r.distinct(b, &bonus_pool);
    let clauses = r.distinct(c, &clause_pool);
    let word_clause = if w > 0 && weapon && !table.word_clauses.is_empty() {
        let owned: Vec<usize> = (0..table.word_clauses.len()).filter(|i| known.contains(&table.word_clauses[*i].word)).collect();
        let pick_owned = !owned.is_empty() && r.unit() < 2.0 / 3.0;
        let pool: Vec<usize> = if pick_owned { owned } else { (0..table.word_clauses.len()).collect() };
        Some(pool[(r.next() % pool.len() as u64) as usize])
    } else {
        None
    };
    Some(Item { id: seed, kind, material, tier, bonuses, clauses, word_clause, look: (r.next() % 12) as u32, made: 4060 + (r.next() % 52) as u32, canonical: None, kills: 0 })
}

impl Item {
    /// A Plain item of a type and material, by their keys: what a player starts with,
    /// or what an authored place holds.
    pub fn plain(table: &LootTable, kind: &str, material: &str, id: u64) -> Option<Item> {
        let kind = table.types.iter().position(|t| t.key == kind)?;
        let material = table.materials.iter().position(|m| m.key == material)?;
        Some(Item { id, kind, material, tier: Tier::Plain, bonuses: Vec::new(), clauses: Vec::new(), word_clause: None, look: 0, made: 4100, canonical: None, kills: 0 })
    }

    /// A Canonical item, by its key: it has its one authored source.
    pub fn canonical(table: &LootTable, key: &str, id: u64) -> Option<Item> {
        let k = table.canonical.iter().position(|c| c.key == key)?;
        let c = &table.canonical[k];
        Some(Item { id, kind: c.kind as usize, material: c.material as usize, tier: Tier::Canonical, bonuses: Vec::new(), clauses: Vec::new(), word_clause: None, look: 0, made: 4071, canonical: Some(k), kills: 0 })
    }

    /// A Canonical item's own effect, if it is one.
    pub fn canonical_effect<'a>(&self, table: &'a LootTable) -> Option<&'a str> {
        self.canonical.and_then(|k| table.canonical.get(k)).map(|c| c.effect.as_str())
    }

    /// The sum of one effect over its bonuses and clauses.
    pub fn effect(&self, table: &LootTable, effect: &str) -> f32 {
        let b = self.bonuses.iter().filter_map(|i| table.bonuses.get(*i)).filter(|m| m.effect == effect).map(|m| m.amount);
        let c = self.clauses.iter().filter_map(|i| table.clauses.get(*i)).filter(|m| m.effect == effect).map(|m| m.amount);
        b.chain(c).sum()
    }

    /// Damage, from its type and material, and its bonuses.
    pub fn damage(&self, table: &LootTable) -> f32 {
        let (t, m) = (&table.types[self.kind], &table.materials[self.material]);
        t.damage * m.damage * (1.0 + self.effect(table, "damage"))
    }

    pub fn armour(&self, table: &LootTable) -> f32 {
        table.types[self.kind].armour * table.materials[self.material].armour
    }

    pub fn weight(&self, table: &LootTable) -> f32 {
        table.types[self.kind].weight * table.materials[self.material].weight
    }

    /// What a merchant asks for it, in Marks.
    pub fn value(&self, table: &LootTable) -> u32 {
        (table.types[self.kind].value as f32 * table.materials[self.material].value * self.tier.worth()).round() as u32
    }

    /// Where it is held or worn.
    pub fn slot<'a>(&self, table: &'a LootTable) -> &'a str {
        &table.types[self.kind].slot
    }

    pub fn name(&self, table: &LootTable) -> NameParts {
        // A Canonical item has its own name, whole.
        if let Some(c) = self.canonical.and_then(|k| table.canonical.get(k)) {
            return NameParts { maker: None, material: String::new(), kind: format!("item.canonical.{}", c.key), clause: None, word_clause: None };
        }
        NameParts {
            maker: self.bonuses.first().map(|i| format!("item.maker.{}", table.bonuses[*i].key)),
            material: format!("item.material.{}", table.materials[self.material].key),
            kind: format!("item.type.{}", table.types[self.kind].key),
            clause: self.clauses.first().map(|i| format!("item.clause.{}", table.clauses[*i].key)),
            word_clause: self.word_clause.map(|i| format!("item.word_clause.{}", table.word_clauses[i].key)),
        }
    }

    /// A single number for how good it is, for comparison and for the loot simulation:
    /// its damage or armour, raised by what its bonuses and clauses add.
    pub fn score(&self, table: &LootTable) -> f32 {
        let base = self.damage(table) + self.armour(table);
        let extras = self.bonuses.len() as f32 * 0.08 + self.clauses.len() as f32 * 0.12 + self.word_clause.map_or(0.0, |_| 0.15) + if self.canonical.is_some() { 0.8 } else { 0.0 };
        base * (1.0 + extras)
    }
}
