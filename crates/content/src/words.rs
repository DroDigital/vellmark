//! The words, as the bake compiles them from one sheet per word
//! (`assets/data/words/<word>.ron`) and one file per compound. The code reads only this
//! table: what each word does to each class of target, or the line with which that
//! target refuses it; its typo, likewise; what it does on a held weapon; its motif; and
//! the settings of its five-part effect.

use crate::StringId;

const MAGIC: &[u8; 8] = b"VLMKWRD1";

/// The asset key of the one table of words.
pub const WORDS_KEY: &str = "words.table";

/// What a word does to one class of target.
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    /// An effect the rules carry out, by its name, how strongly, and the plain words the
    /// crosshair shows before the cast.
    Effect { effect: String, strength: f32, preview: StringId },
    /// The target declines, in this line.
    Refusal(StringId),
}

/// How a word's magic looks, in its five parts.
#[derive(Clone, Debug, PartialEq)]
pub struct EffectLook {
    /// The light of its class.
    pub colour: [f32; 3],
    /// How many characters gather at the hand at a full charge.
    pub charge_glyphs: u32,
    /// How fast it flies, in metres a second.
    pub flight_speed: f32,
    /// The impact's shape, by name: `burst`, `creep`, `knit`, `drift`, `drop`, `thin`.
    pub impact: String,
    /// How fast the rewrite wave spreads, in metres a second.
    pub wave_speed: f32,
    /// What the surface shows while the word holds, by name.
    pub lasting: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Typo {
    pub word: String,
    pub caption: StringId,
    /// One for each target class, in the table's order.
    pub outcomes: Vec<Outcome>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Word {
    pub word: String,
    pub class: String,
    pub ink: f32,
    /// MIDI note numbers, three to five.
    pub motif: Vec<u8>,
    pub caption: StringId,
    /// What it does in a fight, in a line.
    pub combat: StringId,
    /// What it does written on a held weapon, or the weapon's refusal.
    pub weapon: Outcome,
    pub look: EffectLook,
    /// One for each target class, in the table's order.
    pub outcomes: Vec<Outcome>,
    pub typo: Typo,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Compound {
    pub name: String,
    pub caption: StringId,
    /// The two words, by their spelling.
    pub words: [String; 2],
    pub outcomes: Vec<Outcome>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct WordTable {
    /// The target classes, by key: `door`, `lamp`, `creature`.
    pub classes: Vec<String>,
    pub words: Vec<Word>,
    pub compounds: Vec<Compound>,
}

impl WordTable {
    pub fn class_index(&self, class: &str) -> Option<usize> {
        self.classes.iter().position(|c| c == class)
    }

    pub fn word(&self, spelling: &str) -> Option<&Word> {
        self.words.iter().find(|w| w.word == spelling)
    }

    /// The compound two words make, in either order.
    pub fn compound(&self, a: &str, b: &str) -> Option<&Compound> {
        self.compounds.iter().find(|c| (c.words[0] == a && c.words[1] == b) || (c.words[0] == b && c.words[1] == a))
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut w = Writer(MAGIC.to_vec());
        w.u32(self.classes.len() as u32);
        for c in &self.classes {
            w.str(c);
        }
        w.u32(self.words.len() as u32);
        for word in &self.words {
            w.str(&word.word);
            w.str(&word.class);
            w.f32(word.ink);
            w.u32(word.motif.len() as u32);
            w.0.extend_from_slice(&word.motif);
            w.u64(word.caption.0);
            w.u64(word.combat.0);
            w.outcome(&word.weapon);
            let l = &word.look;
            for v in l.colour {
                w.f32(v);
            }
            w.u32(l.charge_glyphs);
            w.f32(l.flight_speed);
            w.str(&l.impact);
            w.f32(l.wave_speed);
            w.str(&l.lasting);
            w.outcomes(&word.outcomes);
            w.str(&word.typo.word);
            w.u64(word.typo.caption.0);
            w.outcomes(&word.typo.outcomes);
        }
        w.u32(self.compounds.len() as u32);
        for c in &self.compounds {
            w.str(&c.name);
            w.u64(c.caption.0);
            w.str(&c.words[0]);
            w.str(&c.words[1]);
            w.outcomes(&c.outcomes);
        }
        w.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 12 || &bytes[..8] != MAGIC {
            return Err("not a table of words".into());
        }
        let mut r = Reader(bytes, 8);
        let n = r.u32()? as usize;
        let classes = (0..n).map(|_| r.str()).collect::<Result<Vec<_>, _>>()?;
        let n = r.u32()? as usize;
        let mut words = Vec::with_capacity(n.min(256));
        for _ in 0..n {
            let word = r.str()?;
            let class = r.str()?;
            let ink = r.f32()?;
            let m = r.u32()? as usize;
            let motif = r.take(m)?.to_vec();
            let caption = StringId(r.u64()?);
            let combat = StringId(r.u64()?);
            let weapon = r.outcome()?;
            let colour = [r.f32()?, r.f32()?, r.f32()?];
            let look = EffectLook { colour, charge_glyphs: r.u32()?, flight_speed: r.f32()?, impact: r.str()?, wave_speed: r.f32()?, lasting: r.str()? };
            let outcomes = r.outcomes()?;
            let typo = Typo { word: r.str()?, caption: StringId(r.u64()?), outcomes: r.outcomes()? };
            words.push(Word { word, class, ink, motif, caption, combat, weapon, look, outcomes, typo });
        }
        let n = r.u32()? as usize;
        let mut compounds = Vec::with_capacity(n.min(256));
        for _ in 0..n {
            compounds.push(Compound { name: r.str()?, caption: StringId(r.u64()?), words: [r.str()?, r.str()?], outcomes: r.outcomes()? });
        }
        Ok(Self { classes, words, compounds })
    }
}

struct Writer(Vec<u8>);

impl Writer {
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn f32(&mut self, v: f32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn str(&mut self, s: &str) {
        self.u32(s.len() as u32);
        self.0.extend_from_slice(s.as_bytes());
    }
    fn outcome(&mut self, o: &Outcome) {
        match o {
            Outcome::Effect { effect, strength, preview } => {
                self.0.push(0);
                self.str(effect);
                self.f32(*strength);
                self.u64(preview.0);
            }
            Outcome::Refusal(line) => {
                self.0.push(1);
                self.u64(line.0);
            }
        }
    }
    fn outcomes(&mut self, os: &[Outcome]) {
        self.u32(os.len() as u32);
        for o in os {
            self.outcome(o);
        }
    }
}

struct Reader<'a>(&'a [u8], usize);

impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let b = self.0.get(self.1..self.1 + n).ok_or_else(|| "the table of words is cut short".to_string())?;
        self.1 += n;
        Ok(b)
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
    fn str(&mut self) -> Result<String, String> {
        let n = self.u32()? as usize;
        String::from_utf8(self.take(n)?.to_vec()).map_err(|_| "a name in the table of words is not text".to_string())
    }
    fn outcome(&mut self) -> Result<Outcome, String> {
        match self.take(1)?[0] {
            0 => Ok(Outcome::Effect { effect: self.str()?, strength: self.f32()?, preview: StringId(self.u64()?) }),
            1 => Ok(Outcome::Refusal(StringId(self.u64()?))),
            _ => Err("an outcome in the table of words is neither an effect nor a refusal".into()),
        }
    }
    fn outcomes(&mut self) -> Result<Vec<Outcome>, String> {
        let n = self.u32()? as usize;
        (0..n).map(|_| self.outcome()).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_table_round_trips() {
        let o = vec![Outcome::Effect { effect: "open".into(), strength: 1.0, preview: StringId(5) }, Outcome::Refusal(StringId(6))];
        let t = WordTable {
            classes: vec!["door".into(), "stone".into()],
            words: vec![Word {
                word: "OPEN".into(),
                class: "Access".into(),
                ink: 8.0,
                motif: vec![62, 69, 74],
                caption: StringId(1),
                combat: StringId(2),
                weapon: Outcome::Refusal(StringId(3)),
                look: EffectLook { colour: [1.0, 0.9, 0.5], charge_glyphs: 24, flight_speed: 14.0, impact: "burst".into(), wave_speed: 3.0, lasting: "glow".into() },
                outcomes: o.clone(),
                typo: Typo { word: "OVEN".into(), caption: StringId(4), outcomes: o.clone() },
            }],
            compounds: vec![Compound { name: "Rapt".into(), caption: StringId(7), words: ["LIT".into(), "CALM".into()], outcomes: o }],
        };
        assert_eq!(WordTable::from_bytes(&t.to_bytes()).unwrap(), t);
        assert!(t.compound("CALM", "LIT").is_some());
    }
}
