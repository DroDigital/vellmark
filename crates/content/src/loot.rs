//! The loot table, as baked from `assets/data/loot.ron`: materials, the types of thing
//! that can be found, bonuses, clauses, word clauses, and where loot comes from with the
//! chance of each tier. One table, `loot.table`.

/// The asset key of the loot table.
pub const LOOT_KEY: &str = "loot.table";

#[derive(Clone, Debug, PartialEq)]
pub struct LootMaterial {
    pub key: String,
    pub damage: f32,
    pub armour: f32,
    pub weight: f32,
    pub value: f32,
    pub trait_key: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LootType {
    pub key: String,
    /// Where it is held or worn: `main`, `off`, `both`, `body`, `head`, `trinket`.
    pub slot: String,
    pub damage: f32,
    pub armour: f32,
    pub weight: f32,
    pub value: u32,
}

/// A bonus or a clause: one effect, by name, and how much.
#[derive(Clone, Debug, PartialEq)]
pub struct LootModifier {
    pub key: String,
    pub effect: String,
    pub amount: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LootWordClause {
    pub key: String,
    pub word: String,
    pub effect: String,
}

/// A Canonical item: its type and material by index, and its own effect.
#[derive(Clone, Debug, PartialEq)]
pub struct LootCanonical {
    pub key: String,
    pub kind: u32,
    pub material: u32,
    pub effect: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct LootSource {
    pub key: String,
    /// The chance of each tier, Plain to Illuminated.
    pub tiers: [f32; 5],
    /// Types and materials, by index into the table's lists, each with a weight.
    pub types: Vec<(u32, u32)>,
    pub materials: Vec<(u32, u32)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct LootTable {
    pub materials: Vec<LootMaterial>,
    pub types: Vec<LootType>,
    pub bonuses: Vec<LootModifier>,
    pub clauses: Vec<LootModifier>,
    pub word_clauses: Vec<LootWordClause>,
    pub sources: Vec<LootSource>,
    pub canonical: Vec<LootCanonical>,
}

struct W(Vec<u8>);

impl W {
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn s(&mut self, v: &str) {
        self.u32(v.len() as u32);
        self.0.extend_from_slice(v.as_bytes());
    }
}

struct R<'a>(&'a [u8], usize);

impl R<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let b = self.0.get(self.1..self.1 + n).ok_or("the loot table is cut short")?;
        self.1 += n;
        Ok(b)
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn s(&mut self) -> Result<String, String> {
        let n = self.u32()? as usize;
        String::from_utf8(self.take(n)?.to_vec()).map_err(|_| "a name in the loot table is not text".to_string())
    }
    /// A count, kept to what the rest of the table could hold.
    fn count(&mut self) -> Result<usize, String> {
        let n = self.u32()? as usize;
        if n > self.0.len() {
            return Err("the loot table is cut short".into());
        }
        Ok(n)
    }
}

impl LootTable {
    const MAGIC: &'static [u8; 8] = b"VLMKLOT2";

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut w = W(Self::MAGIC.to_vec());
        w.u32(self.materials.len() as u32);
        for m in &self.materials {
            w.s(&m.key);
            for v in [m.damage, m.armour, m.weight, m.value] {
                w.f32(v);
            }
            w.s(&m.trait_key);
        }
        w.u32(self.types.len() as u32);
        for t in &self.types {
            w.s(&t.key);
            w.s(&t.slot);
            for v in [t.damage, t.armour, t.weight] {
                w.f32(v);
            }
            w.u32(t.value);
        }
        for list in [&self.bonuses, &self.clauses] {
            w.u32(list.len() as u32);
            for m in list {
                w.s(&m.key);
                w.s(&m.effect);
                w.f32(m.amount);
            }
        }
        w.u32(self.word_clauses.len() as u32);
        for c in &self.word_clauses {
            w.s(&c.key);
            w.s(&c.word);
            w.s(&c.effect);
        }
        w.u32(self.sources.len() as u32);
        for s in &self.sources {
            w.s(&s.key);
            for v in s.tiers {
                w.f32(v);
            }
            for list in [&s.types, &s.materials] {
                w.u32(list.len() as u32);
                for (i, weight) in list {
                    w.u32(*i);
                    w.u32(*weight);
                }
            }
        }
        w.u32(self.canonical.len() as u32);
        for c in &self.canonical {
            w.s(&c.key);
            w.u32(c.kind);
            w.u32(c.material);
            w.s(&c.effect);
        }
        w.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 8 || &bytes[..8] != Self::MAGIC {
            return Err("not a loot table".into());
        }
        let mut r = R(bytes, 8);
        let mut t = LootTable::default();
        for _ in 0..r.count()? {
            t.materials.push(LootMaterial { key: r.s()?, damage: r.f32()?, armour: r.f32()?, weight: r.f32()?, value: r.f32()?, trait_key: r.s()? });
        }
        for _ in 0..r.count()? {
            t.types.push(LootType { key: r.s()?, slot: r.s()?, damage: r.f32()?, armour: r.f32()?, weight: r.f32()?, value: r.u32()? });
        }
        for which in 0..2 {
            for _ in 0..r.count()? {
                let m = LootModifier { key: r.s()?, effect: r.s()?, amount: r.f32()? };
                if which == 0 {
                    t.bonuses.push(m);
                } else {
                    t.clauses.push(m);
                }
            }
        }
        for _ in 0..r.count()? {
            t.word_clauses.push(LootWordClause { key: r.s()?, word: r.s()?, effect: r.s()? });
        }
        for _ in 0..r.count()? {
            let key = r.s()?;
            let tiers = [r.f32()?, r.f32()?, r.f32()?, r.f32()?, r.f32()?];
            let mut lists = [Vec::new(), Vec::new()];
            for list in &mut lists {
                for _ in 0..r.count()? {
                    list.push((r.u32()?, r.u32()?));
                }
            }
            let [types, materials] = lists;
            t.sources.push(LootSource { key, tiers, types, materials });
        }
        for _ in 0..r.count()? {
            t.canonical.push(LootCanonical { key: r.s()?, kind: r.u32()?, material: r.u32()?, effect: r.s()? });
        }
        if r.1 != bytes.len() {
            return Err("the loot table has trailing bytes".into());
        }
        Ok(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_loot_table_round_trips() {
        let t = LootTable {
            materials: vec![LootMaterial { key: "iron".into(), damage: 1.0, armour: 1.0, weight: 1.0, value: 1.0, trait_key: "honest".into() }],
            types: vec![LootType { key: "sword".into(), slot: "main".into(), damage: 12.0, armour: 0.0, weight: 3.0, value: 30 }],
            bonuses: vec![LootModifier { key: "tally".into(), effect: "damage".into(), amount: 0.1 }],
            clauses: vec![LootModifier { key: "escrow".into(), effect: "ink_per_kill".into(), amount: 15.0 }],
            word_clauses: vec![LootWordClause { key: "without_prejudice".into(), word: "COLD".into(), effect: "word_passes".into() }],
            sources: vec![LootSource { key: "chest".into(), tiers: [0.5, 0.28, 0.14, 0.065, 0.015], types: vec![(0, 3)], materials: vec![(0, 5)] }],
            canonical: vec![LootCanonical { key: "wicks_spade".into(), kind: 0, material: 0, effect: "buries".into() }],
        };
        assert_eq!(LootTable::from_bytes(&t.to_bytes()).unwrap(), t);
    }
}
