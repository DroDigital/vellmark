//! A saved game: what the world must remember to be itself again, and nothing it can
//! work out afresh. Players: where they stand, Health, Stamina, Ink, the words they know
//! and hold, and what they carry. The world: the clock, the days, what is broken, the
//! words that last on things, the creatures and how they are, where loose things came to
//! rest, the items lying about, and the merchants. Letters in flight, words that fade,
//! and motion are not saved: a broken thing's letters lie where they settle again.
//!
//! Words are saved by their spelling, so a save outlives a change in the order of the
//! word sheets.

use glam::{Quat, Vec3};

use crate::items::{Item, Tier};

const MAGIC: &[u8; 8] = b"VLMKSAV2";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SaveGame {
    pub seconds: f64,
    pub time_of_day: f32,
    pub day: u32,
    pub mornings: u32,
    pub rng: u64,
    pub players: Vec<SavedPlayer>,
    pub broken: Vec<u64>,
    /// Lasting words on things: each thing's ID, and its words with their classes.
    pub inscriptions: Vec<(u64, Vec<(String, String)>)>,
    pub creatures: Vec<SavedCreature>,
    /// Where each loose thing came to rest.
    pub props: Vec<(u64, Vec3, Quat)>,
    pub dropped: Vec<(Item, Vec3)>,
    pub merchants: Vec<SavedMerchant>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SavedPlayer {
    pub feet: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub health: f32,
    pub stamina: f32,
    pub ink: f32,
    pub arrows: u32,
    /// The words known, by spelling, in the order learned.
    pub known: Vec<String>,
    /// What each hand holds: 0 empty, 1 a weapon, 2 a shield, 3 a word (by spelling).
    pub holding: [(u8, String); 2],
    pub items: Vec<Item>,
    /// Main, off, body, head, and the two trinkets: indices into `items`, or none.
    pub equipped: [Option<u32>; 6],
    pub marks: u32,
    /// Favourite pairings: the main hand's item, and the other's (0 empty, 1 item, 2 word)
    /// with its item index or the word's spelling.
    pub favourites: Vec<(Option<u32>, u8, u32, String)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SavedCreature {
    pub id: u64,
    pub class: String,
    pub feet: Vec3,
    pub yaw: f32,
    pub health: f32,
    pub max_health: f32,
    pub size: f32,
    pub annotation: Option<String>,
    pub elite: bool,
    pub drops: bool,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct SavedMerchant {
    pub id: u64,
    pub day: u32,
    pub purse: u32,
    pub stock: Vec<Item>,
}

/// Bytes out.
struct W(Vec<u8>);

impl W {
    fn u8(&mut self, v: u8) {
        self.0.push(v);
    }
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn f64(&mut self, v: f64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn v3(&mut self, v: Vec3) {
        for x in v.to_array() {
            self.f32(x);
        }
    }
    fn s(&mut self, v: &str) {
        self.u32(v.len() as u32);
        self.0.extend_from_slice(v.as_bytes());
    }
    fn opt_u32(&mut self, v: Option<u32>) {
        self.u32(v.map_or(u32::MAX, |x| x));
    }
    fn item(&mut self, i: &Item) {
        self.u64(i.id);
        self.u32(i.kind as u32);
        self.u32(i.material as u32);
        self.u8(i.tier as u8);
        self.u32(i.bonuses.len() as u32);
        for b in &i.bonuses {
            self.u32(*b as u32);
        }
        self.u32(i.clauses.len() as u32);
        for c in &i.clauses {
            self.u32(*c as u32);
        }
        self.opt_u32(i.word_clause.map(|w| w as u32));
        self.u32(i.look);
        self.u32(i.made);
        self.opt_u32(i.canonical.map(|c| c as u32));
    }
}

/// Bytes in.
struct R<'a>(&'a [u8], usize);

impl R<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let b = self.0.get(self.1..self.1 + n).ok_or("the save is cut short")?;
        self.1 += n;
        Ok(b)
    }
    fn u8(&mut self) -> Result<u8, String> {
        Ok(self.take(1)?[0])
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32, String> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn f64(&mut self) -> Result<f64, String> {
        Ok(f64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn v3(&mut self) -> Result<Vec3, String> {
        Ok(Vec3::new(self.f32()?, self.f32()?, self.f32()?))
    }
    fn s(&mut self) -> Result<String, String> {
        let n = self.count()?;
        String::from_utf8(self.take(n)?.to_vec()).map_err(|_| "a name in the save is not text".to_string())
    }
    fn count(&mut self) -> Result<usize, String> {
        let n = self.u32()? as usize;
        if n > self.0.len() {
            return Err("the save is cut short".into());
        }
        Ok(n)
    }
    fn opt_u32(&mut self) -> Result<Option<u32>, String> {
        let v = self.u32()?;
        Ok((v != u32::MAX).then_some(v))
    }
    fn item(&mut self) -> Result<Item, String> {
        let id = self.u64()?;
        let kind = self.u32()? as usize;
        let material = self.u32()? as usize;
        let tier = match self.u8()? {
            0 => Tier::Plain,
            1 => Tier::Fair,
            2 => Tier::Endorsed,
            3 => Tier::Engrossed,
            4 => Tier::Illuminated,
            5 => Tier::Canonical,
            _ => return Err("an item in the save has no tier".into()),
        };
        let mut bonuses = Vec::new();
        for _ in 0..self.count()? {
            bonuses.push(self.u32()? as usize);
        }
        let mut clauses = Vec::new();
        for _ in 0..self.count()? {
            clauses.push(self.u32()? as usize);
        }
        let word_clause = self.opt_u32()?.map(|w| w as usize);
        Ok(Item { id, kind, material, tier, bonuses, clauses, word_clause, look: self.u32()?, made: self.u32()?, canonical: self.opt_u32()?.map(|c| c as usize) })
    }
}

impl SaveGame {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut w = W(MAGIC.to_vec());
        w.f64(self.seconds);
        w.f32(self.time_of_day);
        w.u32(self.day);
        w.u32(self.mornings);
        w.u64(self.rng);
        w.u32(self.players.len() as u32);
        for p in &self.players {
            w.v3(p.feet);
            for v in [p.yaw, p.pitch, p.health, p.stamina, p.ink] {
                w.f32(v);
            }
            w.u32(p.arrows);
            w.u32(p.known.len() as u32);
            for k in &p.known {
                w.s(k);
            }
            for (kind, word) in &p.holding {
                w.u8(*kind);
                w.s(word);
            }
            w.u32(p.items.len() as u32);
            for i in &p.items {
                w.item(i);
            }
            for e in p.equipped {
                w.opt_u32(e);
            }
            w.u32(p.marks);
            w.u32(p.favourites.len() as u32);
            for (main, kind, item, word) in &p.favourites {
                w.opt_u32(*main);
                w.u8(*kind);
                w.u32(*item);
                w.s(word);
            }
        }
        w.u32(self.broken.len() as u32);
        for b in &self.broken {
            w.u64(*b);
        }
        w.u32(self.inscriptions.len() as u32);
        for (id, words) in &self.inscriptions {
            w.u64(*id);
            w.u32(words.len() as u32);
            for (word, class) in words {
                w.s(word);
                w.s(class);
            }
        }
        w.u32(self.creatures.len() as u32);
        for c in &self.creatures {
            w.u64(c.id);
            w.s(&c.class);
            w.v3(c.feet);
            for v in [c.yaw, c.health, c.max_health, c.size] {
                w.f32(v);
            }
            w.s(c.annotation.as_deref().unwrap_or(""));
            w.u8(c.elite as u8 | (c.drops as u8) << 1);
        }
        w.u32(self.props.len() as u32);
        for (id, at, turn) in &self.props {
            w.u64(*id);
            w.v3(*at);
            for x in turn.to_array() {
                w.f32(x);
            }
        }
        w.u32(self.dropped.len() as u32);
        for (item, at) in &self.dropped {
            w.item(item);
            w.v3(*at);
        }
        w.u32(self.merchants.len() as u32);
        for m in &self.merchants {
            w.u64(m.id);
            w.u32(m.day);
            w.u32(m.purse);
            w.u32(m.stock.len() as u32);
            for i in &m.stock {
                w.item(i);
            }
        }
        w.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 8 || &bytes[..8] != MAGIC {
            return Err("not a Vellmark save".into());
        }
        let mut r = R(bytes, 8);
        let mut s = SaveGame { seconds: r.f64()?, time_of_day: r.f32()?, day: r.u32()?, mornings: r.u32()?, rng: r.u64()?, ..Default::default() };
        for _ in 0..r.count()? {
            let feet = r.v3()?;
            let (yaw, pitch, health, stamina, ink) = (r.f32()?, r.f32()?, r.f32()?, r.f32()?, r.f32()?);
            let arrows = r.u32()?;
            let mut known = Vec::new();
            for _ in 0..r.count()? {
                known.push(r.s()?);
            }
            let holding = [(r.u8()?, r.s()?), (r.u8()?, r.s()?)];
            let mut items = Vec::new();
            for _ in 0..r.count()? {
                items.push(r.item()?);
            }
            let mut equipped = [None; 6];
            for e in &mut equipped {
                *e = r.opt_u32()?;
            }
            let marks = r.u32()?;
            let mut favourites = Vec::new();
            for _ in 0..r.count()? {
                favourites.push((r.opt_u32()?, r.u8()?, r.u32()?, r.s()?));
            }
            s.players.push(SavedPlayer { feet, yaw, pitch, health, stamina, ink, arrows, known, holding, items, equipped, marks, favourites });
        }
        for _ in 0..r.count()? {
            s.broken.push(r.u64()?);
        }
        for _ in 0..r.count()? {
            let id = r.u64()?;
            let mut words = Vec::new();
            for _ in 0..r.count()? {
                words.push((r.s()?, r.s()?));
            }
            s.inscriptions.push((id, words));
        }
        for _ in 0..r.count()? {
            let id = r.u64()?;
            let class = r.s()?;
            let feet = r.v3()?;
            let (yaw, health, max_health, size) = (r.f32()?, r.f32()?, r.f32()?, r.f32()?);
            let annotation = Some(r.s()?).filter(|a| !a.is_empty());
            let flags = r.u8()?;
            s.creatures.push(SavedCreature { id, class, feet, yaw, health, max_health, size, annotation, elite: flags & 1 != 0, drops: flags & 2 != 0 });
        }
        for _ in 0..r.count()? {
            let id = r.u64()?;
            let at = r.v3()?;
            let turn = Quat::from_xyzw(r.f32()?, r.f32()?, r.f32()?, r.f32()?);
            s.props.push((id, at, turn));
        }
        for _ in 0..r.count()? {
            s.dropped.push((r.item()?, r.v3()?));
        }
        for _ in 0..r.count()? {
            let (id, day, purse) = (r.u64()?, r.u32()?, r.u32()?);
            let mut stock = Vec::new();
            for _ in 0..r.count()? {
                stock.push(r.item()?);
            }
            s.merchants.push(SavedMerchant { id, day, purse, stock });
        }
        if r.1 != bytes.len() {
            return Err("the save has trailing bytes".into());
        }
        Ok(s)
    }
}
