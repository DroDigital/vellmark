//! Casting, step by step: what each hand holds, the words a player knows, Ink, the hands'
//! taps and charges turned into words in flight, words landing, and what they change in
//! the world while they hold.

use glam::Vec3;

use crate::emendation::{aim, complete_compound, fade, ink_cost, preview, write_stroke, Flight, Landing, Preview, Stroke, Target, WordRules, CHARGE_FULL, INK_MAX, INK_REFILL};
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
    /// What has been typed while a word charges.
    pub typed: String,
}

impl Default for Caster {
    fn default() -> Self {
        Self { hands: Hands::default(), holding: [Holding::Weapon, Holding::Empty], known: Vec::new(), ink: INK_MAX, aimed: None, last_buttons: 0, typed: String::new() }
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
    /// It came out as its word's typo.
    pub typo: bool,
    /// The compound it completed, by name, and what that did.
    pub compound: Option<(String, Landing)>,
    /// How many links of a chain of reactions led here: 0 for a word written by hand.
    pub link: u32,
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
        // A charge begins: what is typed from here on is read when it is let go.
        if let HandEvent::HoldBegin(_) | HandEvent::BothBegin = e {
            caster.typed.clear();
        }
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
        // Typing the word as it charged makes it stronger; typing its typo casts the typo.
        let typed = std::mem::take(&mut caster.typed).to_ascii_uppercase();
        let rule = &rules.words[word];
        let stroke = Stroke { charge, both, typed: charge > 0.0 && typed == rule.spelling, typo: charge > 0.0 && typed == rule.typo };
        out.push(send(player, caster, word, hand, stroke, rules, targets, eye, gaze, right, flights));
    }
    out
}

/// Sends a word from a hand towards the target in the gaze, if there is one and Ink
/// enough.
#[allow(clippy::too_many_arguments)]
pub fn send(
    player: u32,
    caster: &mut Caster,
    word: usize,
    hand: Option<Hand>,
    stroke: Stroke,
    rules: &WordRules,
    targets: &[Target],
    eye: Vec3,
    gaze: Vec3,
    right: Vec3,
    flights: &mut Vec<Flight>,
) -> Release {
    let Some(t) = caster.aimed else { return Release::NoTarget };
    let target = &targets[t];
    let rule = &rules.words[word];
    // A typo costs the Ink of the word it came from.
    let cost = ink_cost(rule, target, stroke.charge, stroke.both);
    if cost > caster.ink + 1.0e-3 {
        return Release::NoInk;
    }
    caster.ink -= cost;
    // From the casting hand: a little to its side and below the eye.
    let side = match hand {
        Some(Hand::Main) => 0.22,
        Some(Hand::Off) => -0.22,
        None => 0.0,
    };
    let from = eye + right * side - Vec3::Y * 0.25 + gaze * 0.4;
    flights.push(Flight { player, word, target: target.id, from, at: from, speed: rule.flight_speed, charge: stroke.charge, both: stroke.both, typed: stroke.typed, typo: stroke.typo });
    Release::Flying
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
        let stroke = Stroke { charge: f.charge, both: f.both, typed: f.typed, typo: f.typo };
        let landing = write_stroke(rules, f.word, target, stroke, now);
        let compound = if matches!(landing, Landing::Took { .. }) && !f.typo {
            complete_compound(rules, f.word, target, now).map(|(c, l)| (rules.compounds[c].name.clone(), l))
        } else {
            None
        };
        let word = if f.typo { rules.words[f.word].typo.clone() } else { rules.words[f.word].spelling.clone() };
        events.push(CastEvent { player: f.player, word, target: f.target, at: target.centre, landing, both: f.both, charge: f.charge, typo: f.typo, compound, link: 0 });
        false
    });
}

/// Classes of thing that catch fire from a burning neighbour.
const FLAMMABLE: &[&str] = &["door", "container", "structure", "plant", "paper", "creature", "inkling"];
/// At most this many things burn at once, and a chain runs at most this many links.
const MOST_BURNING: usize = 24;
const LONGEST_CHAIN: u32 = 8;
/// A thing caught from a neighbour burns this long.
const CAUGHT_HOLD: f64 = 25.0;
/// A thing that has burned cannot catch again for this long after its fire was last
/// alight: there is nothing left to burn.
const BURNT_FOR: f64 = 90.0;

/// The systemic chain, run every so often: a burning thing sets its flammable neighbours
/// alight, one link further each time. `roll` gives numbers from 0 to 1, from the world's
/// own deterministic random numbers.
pub fn spread(targets: &mut [Target], rules: &WordRules, now: f64, chance: f32, roll: &mut dyn FnMut() -> f32, events: &mut Vec<CastEvent>) {
    let Some(burning_word) = rules.word("BURNING") else { return };
    let burning = |t: &Target| t.effects.iter().any(|e| matches!(e.effect.as_str(), "ignite" | "flare" | "blaze"));
    for t in targets.iter_mut().filter(|t| burning(t)) {
        t.burnt_until = now + BURNT_FOR;
    }
    let lit: Vec<(usize, Vec3, f32, u32)> = targets.iter().enumerate().filter(|(_, t)| burning(t)).map(|(i, t)| (i, t.centre, t.radius, t.link)).collect();
    let mut count = lit.len();
    for (_, centre, radius, link) in lit {
        if link >= LONGEST_CHAIN {
            continue;
        }
        for t in targets.iter_mut() {
            if count >= MOST_BURNING {
                return;
            }
            if burning(t) || now < t.burnt_until || !FLAMMABLE.contains(&rules.classes[t.class].as_str()) {
                continue;
            }
            // Near enough to catch: within a metre of touching.
            if t.centre.distance(centre) > t.radius + radius + 1.0 {
                continue;
            }
            if roll() > chance {
                continue;
            }
            let landing = write_stroke(rules, burning_word, t, Stroke::default(), now);
            if let Landing::Took { .. } = landing {
                // What catches from a neighbour burns for less time than a written word.
                if let Some(w) = t.words.iter_mut().rev().find(|w| w.spelling == rules.words[burning_word].spelling) {
                    w.fades_at = Some(now + CAUGHT_HOLD);
                }
                for e in t.effects.iter_mut().filter(|e| e.by == rules.words[burning_word].spelling) {
                    e.until = Some(now + CAUGHT_HOLD);
                }
                t.link = link + 1;
                count += 1;
                events.push(CastEvent { player: u32::MAX, word: rules.words[burning_word].spelling.clone(), target: t.id, at: t.centre, landing, both: false, charge: 0.0, typo: false, compound: None, link: link + 1 });
            }
        }
    }
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
