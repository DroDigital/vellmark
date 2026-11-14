//! Dialogue, as baked from `assets/dialogue/*.ron`: for each speaker, the conversations
//! they can begin (the first whose conditions hold), the nodes of their tree, and the
//! lines of a Light character, who has no tree and says one line at a time. Every line
//! and choice is a string key, so all of it is translated with the rest. One table,
//! `dialogue.table`.

/// The asset key of the dialogue table.
pub const DIALOGUE_KEY: &str = "dialogue.table";

/// When a line, a choice, or a conversation is offered.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Condition {
    /// The flag is set.
    Flag(String),
    /// The flag is not set.
    Not(String),
    /// A second player is present (the script's `WITNESS:` lines).
    Witness,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Talk {
    pub when: Vec<Condition>,
    /// The node it starts at.
    pub start: u32,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    /// Who says it, by speaker key.
    pub by: String,
    /// The string key's id.
    pub text: u64,
    pub when: Vec<Condition>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Choice {
    pub text: u64,
    pub when: Vec<Condition>,
    /// The node it leads to; none ends the conversation.
    pub then: Option<u32>,
    pub set: Vec<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Node {
    pub lines: Vec<Line>,
    /// Offered after the last line. With none, the node goes on to `next`, or ends.
    pub choices: Vec<Choice>,
    pub next: Option<u32>,
    /// Flags set, Marks given, and a scripted event begun, as the node is reached.
    pub set: Vec<String>,
    pub marks: u32,
    pub event: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Speaker {
    /// `wick`: named `speaker.wick` in the string table.
    pub key: String,
    pub talks: Vec<Talk>,
    pub nodes: Vec<Node>,
    /// A Light character's lines, said one at a time in turn when no talk is offered.
    pub light: Vec<u64>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct DialogueTable {
    pub speakers: Vec<Speaker>,
}

impl DialogueTable {
    pub fn speaker(&self, key: &str) -> Option<usize> {
        self.speakers.iter().position(|s| s.key == key)
    }
}

struct W(Vec<u8>);

impl W {
    fn u32(&mut self, v: u32) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn u64(&mut self, v: u64) {
        self.0.extend_from_slice(&v.to_le_bytes());
    }
    fn s(&mut self, v: &str) {
        self.u32(v.len() as u32);
        self.0.extend_from_slice(v.as_bytes());
    }
    fn opt(&mut self, v: Option<u32>) {
        self.u32(v.map_or(u32::MAX, |v| v));
    }
    fn strings(&mut self, v: &[String]) {
        self.u32(v.len() as u32);
        for s in v {
            self.s(s);
        }
    }
    fn when(&mut self, v: &[Condition]) {
        self.u32(v.len() as u32);
        for c in v {
            match c {
                Condition::Flag(f) => {
                    self.u32(0);
                    self.s(f);
                }
                Condition::Not(f) => {
                    self.u32(1);
                    self.s(f);
                }
                Condition::Witness => self.u32(2),
            }
        }
    }
}

struct R<'a>(&'a [u8], usize);

impl R<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let b = self.0.get(self.1..self.1 + n).ok_or("the dialogue table is cut short")?;
        self.1 += n;
        Ok(b)
    }
    fn u32(&mut self) -> Result<u32, String> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, String> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn count(&mut self) -> Result<usize, String> {
        let n = self.u32()? as usize;
        if n > self.0.len() {
            return Err("the dialogue table is cut short".into());
        }
        Ok(n)
    }
    fn s(&mut self) -> Result<String, String> {
        let n = self.count()?;
        String::from_utf8(self.take(n)?.to_vec()).map_err(|_| "a name in the dialogue table is not text".to_string())
    }
    fn opt(&mut self) -> Result<Option<u32>, String> {
        Ok(Some(self.u32()?).filter(|v| *v != u32::MAX))
    }
    fn strings(&mut self) -> Result<Vec<String>, String> {
        (0..self.count()?).map(|_| self.s()).collect()
    }
    fn when(&mut self) -> Result<Vec<Condition>, String> {
        (0..self.count()?)
            .map(|_| {
                Ok(match self.u32()? {
                    0 => Condition::Flag(self.s()?),
                    1 => Condition::Not(self.s()?),
                    2 => Condition::Witness,
                    _ => return Err("an unknown condition in the dialogue table".to_string()),
                })
            })
            .collect()
    }
}

impl DialogueTable {
    const MAGIC: &'static [u8; 8] = b"VLMKDLG1";

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut w = W(Self::MAGIC.to_vec());
        w.u32(self.speakers.len() as u32);
        for s in &self.speakers {
            w.s(&s.key);
            w.u32(s.talks.len() as u32);
            for t in &s.talks {
                w.when(&t.when);
                w.u32(t.start);
            }
            w.u32(s.nodes.len() as u32);
            for n in &s.nodes {
                w.u32(n.lines.len() as u32);
                for l in &n.lines {
                    w.s(&l.by);
                    w.u64(l.text);
                    w.when(&l.when);
                }
                w.u32(n.choices.len() as u32);
                for c in &n.choices {
                    w.u64(c.text);
                    w.when(&c.when);
                    w.opt(c.then);
                    w.strings(&c.set);
                }
                w.opt(n.next);
                w.strings(&n.set);
                w.u32(n.marks);
                w.s(n.event.as_deref().unwrap_or(""));
            }
            w.u32(s.light.len() as u32);
            for l in &s.light {
                w.u64(*l);
            }
        }
        w.0
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 8 || &bytes[..8] != Self::MAGIC {
            return Err("not a dialogue table".into());
        }
        let mut r = R(bytes, 8);
        let mut t = DialogueTable::default();
        for _ in 0..r.count()? {
            let mut s = Speaker { key: r.s()?, ..Default::default() };
            for _ in 0..r.count()? {
                s.talks.push(Talk { when: r.when()?, start: r.u32()? });
            }
            for _ in 0..r.count()? {
                let mut n = Node::default();
                for _ in 0..r.count()? {
                    n.lines.push(Line { by: r.s()?, text: r.u64()?, when: r.when()? });
                }
                for _ in 0..r.count()? {
                    n.choices.push(Choice { text: r.u64()?, when: r.when()?, then: r.opt()?, set: r.strings()? });
                }
                n.next = r.opt()?;
                n.set = r.strings()?;
                n.marks = r.u32()?;
                n.event = Some(r.s()?).filter(|e| !e.is_empty());
                s.nodes.push(n);
            }
            for _ in 0..r.count()? {
                s.light.push(r.u64()?);
            }
            t.speakers.push(s);
        }
        if r.1 != bytes.len() {
            return Err("the dialogue table has trailing bytes".into());
        }
        Ok(t)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dialogue_table_round_trips() {
        let t = DialogueTable {
            speakers: vec![Speaker {
                key: "wick".into(),
                talks: vec![Talk { when: vec![Condition::Not("wick_met".into())], start: 0 }],
                nodes: vec![
                    Node {
                        lines: vec![Line { by: "wick".into(), text: 7, when: vec![] }, Line { by: "wick".into(), text: 8, when: vec![Condition::Witness] }],
                        choices: vec![Choice { text: 9, when: vec![Condition::Flag("x".into())], then: Some(1), set: vec!["asked".into()] }],
                        next: None,
                        set: vec![],
                        marks: 0,
                        event: None,
                    },
                    Node { lines: vec![], choices: vec![], next: None, set: vec!["wick_met".into()], marks: 12, event: Some("name_fades".into()) },
                ],
                light: vec![3, 4],
            }],
        };
        assert_eq!(DialogueTable::from_bytes(&t.to_bytes()).unwrap(), t);
    }
}
