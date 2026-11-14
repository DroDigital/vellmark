//! Conversations, worked out step by step: who can be spoken to, which conversation a
//! speaker begins (the first whose conditions hold, else a Light character's next line),
//! the lines said in turn, the choices offered after a node's last line, and what a node
//! does as it is reached: flags set, Marks given, a scripted event begun. Flags are the
//! world's and are saved with it.

use std::collections::BTreeSet;

use glam::Vec3;
use vellmark_content::{Condition, DialogueTable};

/// Someone who can be spoken to.
#[derive(Clone, Debug, PartialEq)]
pub struct Talker {
    pub id: u64,
    /// The speaker's key in the dialogue table.
    pub speaker: String,
    pub feet: Vec3,
    pub height: f32,
}

impl Talker {
    /// Where the face is: where the view turns to while they speak.
    pub fn face(&self) -> Vec3 {
        self.feet + Vec3::Y * self.height * 0.93
    }
}

/// How near a talker must be to be spoken to.
pub const TALK_REACH: f32 = 3.0;

/// A conversation under way.
#[derive(Clone, Debug, PartialEq)]
pub struct Conversation {
    pub talker: u64,
    pub speaker: usize,
    /// The node, or none for a Light character's single line.
    pub node: Option<u32>,
    /// The line being said, by its index in the node.
    pub line: usize,
    /// A Light character's line.
    pub light: Option<u64>,
    /// When the line began, in world seconds: the interface writes it out from then.
    pub since: f64,
    /// The node's lines are said and its choices are offered.
    pub choosing: bool,
}

/// What reaching nodes did, for the world to carry out.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Effects {
    pub marks: u32,
    pub events: Vec<String>,
}

/// Whether every condition holds.
pub fn holds(when: &[Condition], flags: &BTreeSet<String>, witness: bool) -> bool {
    when.iter().all(|c| match c {
        Condition::Flag(f) => flags.contains(f),
        Condition::Not(f) => !flags.contains(f),
        Condition::Witness => witness,
    })
}

/// The talker in the gaze and within reach, if any: the nearest whose body the gaze
/// meets, within a hand's breadth to either side.
pub fn talker_in_gaze(talkers: &[Talker], eye: Vec3, gaze: Vec3) -> Option<&Talker> {
    let flat_gaze = Vec3::new(gaze.x, 0.0, gaze.z);
    let reach = flat_gaze.length();
    if reach < 0.1 {
        return None;
    }
    let ahead = flat_gaze / reach;
    talkers
        .iter()
        .filter_map(|t| {
            let to = Vec3::new(t.feet.x - eye.x, 0.0, t.feet.z - eye.z);
            let along = to.dot(ahead);
            let side = (to - ahead * along).length();
            // Where the gaze is, in height, as it passes them.
            let height = eye.y + gaze.y / reach * along;
            let on_body = height > t.feet.y - 0.2 && height < t.feet.y + t.height + 0.3;
            (along > 0.2 && to.length() < TALK_REACH && side < 0.45 && on_body).then_some((t, along))
        })
        .min_by(|a, b| a.1.total_cmp(&b.1))
        .map(|(t, _)| t)
}

/// The world's part in a conversation: the table, the flags, and whether a second
/// player is present.
pub struct Talking<'a> {
    pub table: &'a DialogueTable,
    pub flags: &'a mut BTreeSet<String>,
    pub witness: bool,
    pub now: f64,
    pub effects: Effects,
}

impl Talking<'_> {
    /// Begins a conversation with a speaker. A Light character says their next line;
    /// `light_said` counts how many they have said. None if there is nothing to say.
    pub fn begin(&mut self, speaker: usize, talker: u64, light_said: &mut u32) -> Option<Conversation> {
        let s = self.table.speakers.get(speaker)?;
        let mut c = Conversation { talker, speaker, node: None, line: 0, light: None, since: self.now, choosing: false };
        if let Some(talk) = s.talks.iter().find(|t| holds(&t.when, self.flags, self.witness)) {
            return self.enter(c, talk.start);
        }
        if s.light.is_empty() {
            return None;
        }
        c.light = Some(s.light[*light_said as usize % s.light.len()]);
        *light_said += 1;
        Some(c)
    }

    /// Reaches a node: does what it does, then says its first line, or offers its
    /// choices, or goes on. None when the conversation ends there.
    fn enter(&mut self, mut c: Conversation, node: u32) -> Option<Conversation> {
        let mut node = node;
        // A chain of nodes with nothing to say is followed, but never round a loop.
        for _ in 0..64 {
            let n = self.table.speakers[c.speaker].nodes.get(node as usize)?;
            for f in &n.set {
                self.flags.insert(f.clone());
            }
            self.effects.marks += n.marks;
            self.effects.events.extend(n.event.clone());
            c.node = Some(node);
            c.since = self.now;
            c.choosing = false;
            if let Some(first) = n.lines.iter().position(|l| holds(&l.when, self.flags, self.witness)) {
                c.line = first;
                return Some(c);
            }
            c.line = usize::MAX;
            if self.choices(&c).next().is_some() {
                c.choosing = true;
                return Some(c);
            }
            node = n.next?;
        }
        None
    }

    /// The choices offered now, as indices into the node's.
    pub fn choices<'b>(&'b self, c: &Conversation) -> impl Iterator<Item = usize> + 'b {
        let n = c.node.and_then(|n| self.table.speakers[c.speaker].nodes.get(n as usize));
        let (flags, witness) = (&*self.flags, self.witness);
        n.into_iter().flat_map(move |n| n.choices.iter().enumerate().filter(move |(_, ch)| holds(&ch.when, flags, witness)).map(|(i, _)| i))
    }

    /// Goes on from the line being said: to the next line, the choices, the next node, or
    /// the end. None when the conversation is over.
    pub fn advance(&mut self, mut c: Conversation) -> Option<Conversation> {
        if c.choosing {
            return Some(c);
        }
        let Some(node) = c.node else { return None };
        let n = &self.table.speakers[c.speaker].nodes[node as usize];
        if let Some(next) = n.lines.iter().enumerate().skip(c.line.saturating_add(1)).find(|(_, l)| holds(&l.when, self.flags, self.witness)).map(|(i, _)| i) {
            c.line = next;
            c.since = self.now;
            return Some(c);
        }
        if self.choices(&c).next().is_some() {
            c.choosing = true;
            return Some(c);
        }
        let next = n.next?;
        self.enter(c, next)
    }

    /// Takes the `k`th choice offered, counted from 0.
    pub fn choose(&mut self, c: Conversation, k: usize) -> Option<Conversation> {
        if !c.choosing {
            return Some(c);
        }
        let Some(i) = self.choices(&c).nth(k) else { return Some(c) };
        let n = &self.table.speakers[c.speaker].nodes[c.node? as usize];
        let choice = &n.choices[i];
        for f in &choice.set {
            self.flags.insert(f.clone());
        }
        let then = choice.then?;
        self.enter(c, then)
    }

    /// The line being said: who says it, and its string id.
    pub fn line(&self, c: &Conversation) -> Option<(String, u64)> {
        let s = &self.table.speakers[c.speaker];
        if let Some(l) = c.light {
            return Some((s.key.clone(), l));
        }
        let line = s.nodes.get(c.node? as usize)?.lines.get(c.line)?;
        Some((line.by.clone(), line.text))
    }
}
