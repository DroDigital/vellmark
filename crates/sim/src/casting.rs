//! Casting, step by step: what each hand holds, the words a player knows, Ink, the hands'
//! taps and charges turned into words in flight, words landing, and what they change in
//! the world while they hold.

use glam::Vec3;

use crate::emendation::{aim, fade, ink_cost, preview, write, Flight, Landing, Preview, Target, WordRules, CHARGE_FULL, INK_MAX, INK_REFILL};
use crate::hands::{Hand, HandEvent, Hands};

/// What a hand holds.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Holding {
    Empty,
    /// A one-handed weapon.
    Weapon,
    /// A word, by its index in the rules.
    Word(usize),
}

/// A player's casting state.
#[derive(Clone, Debug, PartialEq)]
pub struct Caster {
    pub hands: Hands,
    pub holding: [Holding; 2],
    /// The words this player knows, by index in the rules, in the order learned.
    pub known: Vec<usize>,
    pub ink: f32,
    /// The target in the gaze, by index.
    pub aimed: Option<usize>,
    /// The buttons held last step, to tell a press.
    pub last_buttons: u32,
}

impl Default for Caster {
    fn default() -> Self {
        Self { hands: Hands::default(), holding: [Holding::Weapon, Holding::Empty], known: Vec::new(), ink: INK_MAX, aimed: None, last_buttons: 0 }
    }
}

impl Caster {
    pub fn learn(&mut self, word: usize) {
        if !self.known.contains(&word) {
            self.known.push(word);
        }
        // An empty hand takes up the word just learned.
        if self.holding[1] == Holding::Empty {
            self.holding[1] = Holding::Word(word);
        }
    }

    /// The off hand's word moves to the next or previous one known.
    pub fn cycle(&mut self, by: i32) {
        if self.known.is_empty() {
            return;
        }
        let at = match self.holding[1] {
            Holding::Word(w) => self.known.iter().position(|&k| k == w).unwrap_or(0) as i32,
            _ => -by,
        };
        let n = self.known.len() as i32;
        self.holding[1] = Holding::Word(self.known[(at + by).rem_euclid(n) as usize]);
    }

    pub fn holds(&self, hand: Hand) -> Holding {
        self.holding[match hand {
            Hand::Main => 0,
            Hand::Off => 1,
        }]
    }

    /// The charge of a hand now, 0 to 1, or of both hands together.
    pub fn charge(&self, hand: Option<Hand>) -> f32 {
        let held = match hand {
            Some(h) => self.hands.charging(h),
            None => self.hands.both(),
        };
        held.map_or(0.0, |t| (t / CHARGE_FULL).min(1.0))
    }
}

/// What happened when a word landed, for the effects and the sound.
#[derive(Clone, Debug, PartialEq)]
pub struct CastEvent {
    pub player: u32,
    /// The word's spelling.
    pub word: String,
    pub target: u64,
    pub at: Vec3,
    pub landing: Landing,
    pub both: bool,
    pub charge: f32,
}

/// A word set off this step, before it lands: or why it was not.
#[derive(Clone, Debug, PartialEq)]
pub enum Release {
    Flying,
    /// Nothing in the gaze to write on.
    NoTarget,
    NoInk,
}

/// Turns the hands' events into words in flight. Returns what each release came to.
#[allow(clippy::too_many_arguments)]
pub fn release(
    player: u32,
    caster: &mut Caster,
    events: &[HandEvent],
    rules: &WordRules,
    targets: &[Target],
    eye: Vec3,
    gaze: Vec3,
    right: Vec3,
    flights: &mut Vec<Flight>,
) -> Vec<Release> {
    let mut out = Vec::new();
    for e in events {
        let (word, hand, charge, both) = match *e {
            HandEvent::Tap(h) => match caster.holds(h) {
                Holding::Word(w) => (w, Some(h), 0.0, false),
                _ => continue,
            },
            HandEvent::HoldEnd(h, t) => match caster.holds(h) {
                Holding::Word(w) => (w, Some(h), (t / CHARGE_FULL).min(1.0), false),
                _ => continue,
            },
            HandEvent::BothEnd(t) => match (caster.holding[0], caster.holding[1]) {
                (Holding::Word(a), Holding::Word(b)) if a == b => (a, None, (t / CHARGE_FULL).min(1.0), true),
                _ => continue,
            },
            _ => continue,
        };
        let Some(t) = caster.aimed else {
            out.push(Release::NoTarget);
            continue;
        };
        let target = &targets[t];
        let rule = &rules.words[word];
        let cost = ink_cost(rule, target, charge, both);
        if cost > caster.ink + 1.0e-3 {
            out.push(Release::NoInk);
            continue;
        }
        caster.ink -= cost;
        // From the casting hand: a little to its side and below the eye.
        let side = match hand {
            Some(Hand::Main) => 0.22,
            Some(Hand::Off) => -0.22,
            None => 0.0,
        };
        let from = eye + right * side - Vec3::Y * 0.25 + gaze * 0.4;
        flights.push(Flight { player, word, target: target.id, from, at: from, speed: rule.flight_speed, charge, both });
        out.push(Release::Flying);
    }
    out
}

/// Moves the words in flight on by `dt`, and writes those that arrive.
pub fn fly(flights: &mut Vec<Flight>, targets: &mut [Target], rules: &WordRules, dt: f32, now: f64, events: &mut Vec<CastEvent>) {
    flights.retain_mut(|f| {
        let Some(target) = targets.iter_mut().find(|t| t.id == f.target) else { return false };
        let to = target.centre - f.at;
        let step = f.speed * dt;
        if to.length() > step + target.radius * 0.5 {
            f.at += to.normalize_or_zero() * step;
            return true;
        }
        let landing = write(rules, f.word, target, f.charge, f.both, now);
        events.push(CastEvent { player: f.player, word: rules.words[f.word].spelling.clone(), target: f.target, at: target.centre, landing, both: f.both, charge: f.charge });
        false
    });
}

/// Ink comes back while the hands are not charging.
pub fn refill(caster: &mut Caster, dt: f32) {
    if caster.hands.charging(Hand::Main).is_none() && caster.hands.charging(Hand::Off).is_none() && caster.hands.both().is_none() {
        caster.ink = (caster.ink + INK_REFILL * dt).min(INK_MAX);
    }
}

/// Lets every written word fade whose time is up. Returns whether any did.
pub fn fade_all(targets: &mut [Target], now: f64) -> bool {
    let mut changed = false;
    for t in targets.iter_mut().filter(|t| t.words.iter().any(|w| w.fades_at.is_some()) || !t.effects.is_empty()) {
        changed |= fade(t, now);
    }
    changed
}

/// What the crosshair shows for a hand's word on the target in the gaze.
pub fn preview_for(caster: &Caster, rules: &WordRules, targets: &[Target], hand: Hand) -> Option<(u64, Preview)> {
    let Holding::Word(w) = caster.holds(hand) else { return None };
    let t = &targets[caster.aimed?];
    let both = caster.hands.both().is_some();
    let charge = caster.charge(if both { None } else { Some(hand) });
    Some((t.id, preview(rules, w, t, charge, both)))
}

/// How hot or cold a thing is from the words on it: -1 frozen to 1 alight.
pub fn heat_of(target: &Target) -> f32 {
    let mut heat = 0.0f32;
    for e in &target.effects {
        heat += match e.effect.as_str() {
            "ignite" | "blaze" => 0.9,
            "flare" => 0.7,
            "heat" | "bake" | "steam" => 0.5,
            "freeze" => -1.0,
            "frost" | "rime" => -0.7,
            "chill" | "slow" => -0.45,
            _ => 0.0,
        } * e.strength.min(1.5);
    }
    heat.clamp(-1.0, 1.0)
}

/// Finds the target the gaze is on, for a caster.
pub fn take_aim(caster: &mut Caster, eye: Vec3, gaze: Vec3, targets: &[Target]) {
    caster.aimed = aim(eye, gaze, targets);
}
