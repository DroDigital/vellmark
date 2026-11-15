//! Dialogue, from `assets/dialogue/*.ron`, one file to a speaker: checked whole, so that
//! every line, choice, and speaker has its string, every node named is there, and every
//! node can be reached. Nodes are kept in the order of their names.

use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use serde::Deserialize;
use vellmark_content::{Choice, Condition, DialogueTable, Line, Node, Speaker, StringId, Talk};

use crate::BakeError;

#[derive(Deserialize)]
enum When {
    Flag(String),
    Not(String),
    Witness,
}

impl From<When> for Condition {
    fn from(w: When) -> Self {
        match w {
            When::Flag(f) => Condition::Flag(f),
            When::Not(f) => Condition::Not(f),
            When::Witness => Condition::Witness,
        }
    }
}

#[derive(Deserialize)]
struct AuthoredSpeaker {
    key: String,
    #[serde(default)]
    talks: Vec<AuthoredTalk>,
    #[serde(default)]
    nodes: BTreeMap<String, AuthoredNode>,
    #[serde(default)]
    light: Vec<String>,
}

#[derive(Deserialize)]
struct AuthoredTalk {
    #[serde(default)]
    when: Vec<When>,
    start: String,
}

/// A line: its string key alone, said by the file's speaker, or with who says it and when.
#[derive(Deserialize)]
#[serde(untagged)]
enum AuthoredLine {
    Plain(String),
    Full {
        text: String,
        #[serde(default)]
        by: Option<String>,
        #[serde(default)]
        when: Vec<When>,
    },
}

#[derive(Deserialize)]
struct AuthoredChoice {
    text: String,
    #[serde(default)]
    when: Vec<When>,
    /// The node it leads to; left out, it ends the conversation.
    #[serde(default)]
    then: String,
    #[serde(default)]
    set: Vec<String>,
}

#[derive(Deserialize)]
struct AuthoredNode {
    #[serde(default)]
    lines: Vec<AuthoredLine>,
    #[serde(default)]
    choices: Vec<AuthoredChoice>,
    #[serde(default)]
    next: String,
    #[serde(default)]
    set: Vec<String>,
    #[serde(default)]
    marks: u32,
    #[serde(default)]
    event: String,
}

/// The scripted events a node may begin. The game answers each.
pub const EVENTS: &[&str] = &["name_fades", "trade"];

pub fn bake_dialogue(dir: &Path, strings: &BTreeMap<String, String>) -> Result<DialogueTable, BakeError> {
    let mut table = DialogueTable::default();
    for file in crate::sorted_files(dir, "ron")? {
        let text = std::fs::read_to_string(&file).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
        let authored: AuthoredSpeaker = ron::from_str(&text).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
        let at = |what: String| BakeError(format!("{}: {what}", file.display()));
        let has = |key: &str| -> Result<u64, BakeError> {
            if strings.contains_key(key) {
                Ok(StringId::from_key(key).0)
            } else {
                Err(at(format!("no string `{key}`")))
            }
        };
        has(&format!("speaker.{}", authored.key))?;
        if table.speaker(&authored.key).is_some() {
            return Err(at(format!("the speaker `{}` is given twice", authored.key)));
        }
        let names: Vec<String> = authored.nodes.keys().cloned().collect();
        let index: BTreeMap<&str, u32> = names.iter().enumerate().map(|(i, k)| (k.as_str(), i as u32)).collect();
        let node = |name: &str| index.get(name).copied().ok_or_else(|| at(format!("no node `{name}`")));
        let key = authored.key.clone();
        let mut speaker = Speaker { key: key.clone(), ..Default::default() };
        let mut reached = BTreeSet::new();
        for t in authored.talks {
            let start = node(&t.start)?;
            reached.insert(start);
            speaker.talks.push(Talk { when: t.when.into_iter().map(Into::into).collect(), start });
        }
        for (name, n) in authored.nodes {
            let mut out = Node { marks: n.marks, set: n.set, ..Default::default() };
            if !n.event.is_empty() {
                let event = n.event;
                if !EVENTS.contains(&event.as_str()) {
                    return Err(at(format!("node `{name}`: no event `{event}`")));
                }
                out.event = Some(event);
            }
            for line in n.lines {
                let (text, by, when) = match line {
                    AuthoredLine::Plain(text) => (text, None, Vec::new()),
                    AuthoredLine::Full { text, by, when } => (text, by, when),
                };
                let by = by.unwrap_or_else(|| key.clone());
                has(&format!("speaker.{by}"))?;
                out.lines.push(Line { by, text: has(&text)?, when: when.into_iter().map(Into::into).collect() });
            }
            for c in n.choices {
                let then = Some(c.then.as_str()).filter(|t| !t.is_empty()).map(node).transpose()?;
                reached.extend(then);
                out.choices.push(Choice { text: has(&c.text)?, when: c.when.into_iter().map(Into::into).collect(), then, set: c.set });
            }
            if !n.next.is_empty() {
                if !out.choices.is_empty() {
                    return Err(at(format!("node `{name}` has both choices and a next node")));
                }
                let next = node(&n.next)?;
                reached.insert(next);
                out.next = Some(next);
            }
            if out.lines.is_empty() && out.choices.is_empty() && out.next.is_none() && out.set.is_empty() && out.marks == 0 && out.event.is_none() {
                return Err(at(format!("node `{name}` does nothing")));
            }
            speaker.nodes.push(out);
        }
        for (name, i) in &index {
            if !reached.contains(i) {
                return Err(at(format!("node `{name}` can never be reached")));
            }
        }
        for line in authored.light {
            speaker.light.push(has(&line)?);
        }
        if speaker.talks.is_empty() && speaker.light.is_empty() {
            return Err(at("a speaker with nothing to say".into()));
        }
        table.speakers.push(speaker);
    }
    Ok(table)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn strings() -> BTreeMap<String, String> {
        ["speaker.wick", "line.a", "line.b", "choice.c"].iter().map(|k| (k.to_string(), "x".to_string())).collect()
    }

    fn bake(text: &str) -> Result<DialogueTable, BakeError> {
        let dir = std::env::temp_dir().join(format!("vellmark-dialogue-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("wick.ron"), text).unwrap();
        let out = bake_dialogue(&dir, &strings());
        let _ = std::fs::remove_dir_all(&dir);
        out
    }

    #[test]
    fn a_tree_bakes_and_every_mistake_is_caught() {
        let good = r#"(key: "wick", talks: [(start: "a")], nodes: {
            "a": (lines: ["line.a", (text: "line.b", when: [Witness])], choices: [(text: "choice.c", then: "b", set: ["asked"])]),
            "b": (lines: ["line.b"], set: ["done"], marks: 12),
        })"#;
        let t = bake(good).unwrap();
        assert_eq!(t.speakers[0].nodes.len(), 2);
        assert_eq!(t.speakers[0].nodes[0].lines[1].when, vec![Condition::Witness]);
        assert_eq!(t.speakers[0].nodes[0].choices[0].then, Some(1));
        assert!(bake(&good.replace("\"line.b\"]", "\"line.z\"]")).is_err(), "a missing string");
        assert!(bake(&good.replace("then: \"b\"", "then: \"c\"")).is_err(), "a missing node");
        assert!(bake(&good.replace("then: \"b\", ", "")).is_err(), "an unreachable node");
    }
}
