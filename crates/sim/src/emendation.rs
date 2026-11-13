//! Emendation: words written on things. A target's Inscription holds at most one word of
//! each class, three words at most. A word written on it replaces the word of its class,
//! or joins the others; what that does is the word's sheet's, compiled into [`WordRules`].
//!
//! A word is written from a hand: a tap writes it at once, a hold charges it, which
//! reaches tougher targets, lasts longer, and costs more Ink, and both hands holding the
//! same word write it together, stronger still. The word flies from the hand and lands.
//! Written words fade after a while, back to what was there before.

use glam::Vec3;

/// Ink a player holds at most, and how fast it comes back, a second.
pub const INK_MAX: f32 = 100.0;
pub const INK_REFILL: f32 = 2.5;
/// A charge is full after this long.
pub const CHARGE_FULL: f32 = 1.2;
/// Words can be written on things this far away at most.
pub const CAST_REACH: f32 = 40.0;
/// How long a written word holds, in seconds of the world's time, before it fades: a tap,
/// and more as the charge grows; both hands hold it longer still.
pub const HOLD_TAP: f32 = 45.0;
pub const HOLD_CHARGED: f32 = 120.0;
pub const BOTH_HANDS_HOLD: f32 = 1.5;
/// A typo holds only this long.
pub const TYPO_HOLD: f32 = 12.0;
/// A compound's effect holds this long.
pub const COMPOUND_HOLD: f32 = 20.0;
/// Typing the word in hand while it charges makes it this much stronger.
pub const TYPED_BONUS: f32 = 1.25;
/// Both hands together: how much stronger, and how much dearer.
pub const BOTH_HANDS_STRENGTH: f32 = 1.5;
pub const BOTH_HANDS_COST: f32 = 1.6;

/// How hard a target is to change. `Sealed` never changes, except by the story.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Resistance {
    Plain = 0,
    Tough = 1,
    Hard = 2,
    Sealed = 3,
}

impl Resistance {
    /// The charge, 0 to 1, a cast needs to reach it.
    pub fn charge_needed(self) -> Option<f32> {
        match self {
            Resistance::Plain => Some(0.0),
            Resistance::Tough => Some(0.5),
            Resistance::Hard => Some(1.0),
            Resistance::Sealed => None,
        }
    }

    pub fn from_index(i: u8) -> Self {
        match i {
            0 => Resistance::Plain,
            1 => Resistance::Tough,
            2 => Resistance::Hard,
            _ => Resistance::Sealed,
        }
    }
}

/// What a word does to one class of target.
#[derive(Clone, Debug, PartialEq)]
pub enum RuleOutcome {
    /// The effect's name, its strength, and the preview's string.
    Effect {
        effect: String,
        strength: f32,
        preview: u64,
    },
    /// The line it is refused with.
    Refusal(u64),
}

#[derive(Clone, Debug, PartialEq)]
pub struct WordRule {
    pub spelling: String,
    pub class: String,
    pub ink: f32,
    pub flight_speed: f32,
    /// One for each target class.
    pub outcomes: Vec<RuleOutcome>,
    pub weapon: RuleOutcome,
    pub typo: String,
    pub typo_outcomes: Vec<RuleOutcome>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CompoundRule {
    pub name: String,
    pub words: [usize; 2],
    pub outcomes: Vec<RuleOutcome>,
}

/// The words as the rules know them, from the word table.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct WordRules {
    pub classes: Vec<String>,
    pub words: Vec<WordRule>,
    pub compounds: Vec<CompoundRule>,
}

impl WordRules {
    pub fn word(&self, spelling: &str) -> Option<usize> {
        self.words.iter().position(|w| w.spelling == spelling)
    }

    pub fn class(&self, key: &str) -> Option<usize> {
        self.classes.iter().position(|c| c == key)
    }
}

/// One word in an Inscription: its spelling and class, and, if it was written and will
/// fade, when.
#[derive(Clone, Debug, PartialEq)]
pub struct InscribedWord {
    pub spelling: String,
    pub class: String,
    pub fades_at: Option<f64>,
    /// What it replaced, which comes back when it fades.
    pub replaced: Option<(String, String)>,
}

/// An effect a written word has on its target, while it holds.
#[derive(Clone, Debug, PartialEq)]
pub struct ActiveEffect {
    pub effect: String,
    pub strength: f32,
    /// The word that made it.
    pub by: String,
    pub until: Option<f64>,
}

/// A thing with an Inscription.
#[derive(Clone, Debug, PartialEq)]
pub struct Target {
    pub id: u64,
    /// Its target class, an index into the rules' classes.
    pub class: usize,
    pub centre: Vec3,
    /// Where it was placed, for a loose thing that has since moved.
    pub authored_centre: Vec3,
    pub radius: f32,
    pub resistance: Resistance,
    pub words: Vec<InscribedWord>,
    pub effects: Vec<ActiveEffect>,
    /// Staggered or unaware: it resists one tier less.
    pub off_guard: bool,
    /// How many links of a chain of reactions led to its last change: 0 by hand.
    pub link: u32,
    /// Burnt: it cannot catch fire from a neighbour again until this time.
    pub burnt_until: f64,
    /// Since when it has been alight, if it is.
    pub alight_since: Option<f64>,
}

impl Target {
    pub fn new(
        id: u64,
        class: usize,
        centre: Vec3,
        radius: f32,
        resistance: Resistance,
        words: &[(String, String)],
    ) -> Self {
        Self {
            id,
            class,
            centre,
            authored_centre: centre,
            radius,
            resistance,
            words: words
                .iter()
                .map(|(s, c)| InscribedWord {
                    spelling: s.clone(),
                    class: c.clone(),
                    fades_at: None,
                    replaced: None,
                })
                .collect(),
            effects: Vec::new(),
            off_guard: false,
            link: 0,
            burnt_until: 0.0,
            alight_since: None,
        }
    }

    pub fn has(&self, effect: &str) -> bool {
        self.effects.iter().any(|e| e.effect == effect)
    }

    /// How hard it is now: one tier less while it is off its guard, and never below Plain.
    pub fn resists(&self) -> Resistance {
        if self.off_guard && self.resistance != Resistance::Sealed {
            Resistance::from_index((self.resistance as u8).saturating_sub(1))
        } else {
            self.resistance
        }
    }
}

/// The Ink a word costs on a target: dearer on larger and tougher things, on a charge,
/// and from both hands.
pub fn ink_cost(rule: &WordRule, target: &Target, charge: f32, both: bool) -> f32 {
    let size = (0.6 + target.radius * 0.4).clamp(0.6, 3.0);
    let tier = 1.0 + 0.5 * (target.resistance as u8).min(2) as f32;
    rule.ink * size * tier * (1.0 + charge) * if both { BOTH_HANDS_COST } else { 1.0 }
}

/// What writing a word on a target would do: shown at the crosshair before the cast.
#[derive(Clone, Debug, PartialEq)]
pub enum Preview {
    /// It would work: the preview's string.
    Result(u64),
    /// The target refuses: its line.
    Refusal(u64),
    /// It would work with this much charge, 0 to 1.
    NeedsCharge(f32),
    /// It can never be changed.
    Sealed,
}

pub fn preview(
    rules: &WordRules,
    word: usize,
    target: &Target,
    charge: f32,
    both: bool,
) -> Preview {
    let rule = &rules.words[word];
    match &rule.outcomes[target.class] {
        RuleOutcome::Refusal(line) => Preview::Refusal(*line),
        RuleOutcome::Effect { preview, .. } => match target.resists().charge_needed() {
            None => Preview::Sealed,
            Some(needed) => {
                // Both hands reach one tier further.
                let reach = if both { needed - 0.5 } else { needed };
                if charge + 1.0e-4 >= reach {
                    Preview::Result(*preview)
                } else {
                    Preview::NeedsCharge(reach)
                }
            }
        },
    }
}

/// What happened when a word landed.
#[derive(Clone, Debug, PartialEq)]
pub enum Landing {
    /// The word took: the effect it had.
    Took {
        effect: String,
        strength: f32,
    },
    Refused(u64),
    /// Not enough charge to move it.
    Resisted,
    Sealed,
}

/// How a word is written: its charge, whether by both hands, typed, or as its typo.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Stroke {
    pub charge: f32,
    pub both: bool,
    /// The word was typed while it charged.
    pub typed: bool,
    /// It came out as its typo.
    pub typo: bool,
}

/// Writes `word` on `target`: changes its Inscription and starts its effect, or not.
pub fn write(
    rules: &WordRules,
    word: usize,
    target: &mut Target,
    charge: f32,
    both: bool,
    now: f64,
) -> Landing {
    write_stroke(
        rules,
        word,
        target,
        Stroke {
            charge,
            both,
            ..Default::default()
        },
        now,
    )
}

/// As `write`, with everything about how it was written.
pub fn write_stroke(
    rules: &WordRules,
    word: usize,
    target: &mut Target,
    stroke: Stroke,
    now: f64,
) -> Landing {
    let rule = &rules.words[word];
    let Stroke {
        charge,
        both,
        typed,
        typo,
    } = stroke;
    let outcomes = if typo {
        &rule.typo_outcomes
    } else {
        &rule.outcomes
    };
    let (effect, strength) = match &outcomes[target.class] {
        RuleOutcome::Refusal(line) => return Landing::Refused(*line),
        RuleOutcome::Effect {
            effect, strength, ..
        } => (effect.clone(), *strength),
    };
    // A typo obeys resistance as its word does.
    match preview(rules, word, target, charge, both) {
        Preview::Sealed => return Landing::Sealed,
        Preview::NeedsCharge(_) => return Landing::Resisted,
        Preview::Refusal(_) if !typo => return Landing::Resisted,
        _ => {}
    }
    if target.resists() == Resistance::Sealed {
        return Landing::Sealed;
    }
    // Mending lasts: a thing made whole stays whole until something breaks it again.
    let lasts = effect == "mend";
    let hold = if typo {
        TYPO_HOLD
    } else {
        (HOLD_TAP + (HOLD_CHARGED - HOLD_TAP) * charge.clamp(0.0, 1.0))
            * if both { BOTH_HANDS_HOLD } else { 1.0 }
    };
    let until = now + hold as f64;
    let strength = strength
        * (1.0 + 0.5 * charge)
        * if both { BOTH_HANDS_STRENGTH } else { 1.0 }
        * if typed { TYPED_BONUS } else { 1.0 };
    let spelling = if typo {
        rule.typo.clone()
    } else {
        rule.spelling.clone()
    };
    // The word of its class goes, and this one takes its place; or it joins the others.
    let replaced = target
        .words
        .iter()
        .position(|w| w.class == rule.class)
        .map(|i| target.words.remove(i));
    if replaced.is_none() && target.words.len() >= 3 {
        // No room: the oldest written word gives way.
        if let Some(i) = target.words.iter().position(|w| w.fades_at.is_some()) {
            let gone = target.words.remove(i);
            target.effects.retain(|e| e.by != gone.spelling);
        } else {
            return Landing::Resisted;
        }
    }
    // A word replaced takes its effect with it.
    if let Some(r) = &replaced {
        target.effects.retain(|e| e.by != r.spelling);
    }
    let original = replaced.map(|r| r.replaced.unwrap_or((r.spelling, r.class)));
    let (fades_at, replaced) = if lasts {
        (None, None)
    } else {
        (Some(until), original)
    };
    target.words.push(InscribedWord {
        spelling: spelling.clone(),
        class: rule.class.clone(),
        fades_at,
        replaced,
    });
    target.effects.push(ActiveEffect {
        effect: effect.clone(),
        strength,
        by: spelling,
        until: if lasts { Some(now + 3.0) } else { Some(until) },
    });
    Landing::Took { effect, strength }
}

/// A compound the word just written completes on its target, if any: the two words of a
/// compound standing on one thing at once. It fires once, now. Returns the compound and
/// what it did.
pub fn complete_compound(
    rules: &WordRules,
    word: usize,
    target: &mut Target,
    now: f64,
) -> Option<(usize, Landing)> {
    let others: Vec<usize> = target
        .words
        .iter()
        .filter_map(|w| rules.word(&w.spelling))
        .filter(|&w| w != word)
        .collect();
    let (index, c) = rules.compounds.iter().enumerate().find(|(_, c)| {
        (c.words[0] == word && others.contains(&c.words[1]))
            || (c.words[1] == word && others.contains(&c.words[0]))
    })?;
    // Compounds obey resistance and never touch Sealed words.
    if target.resists() == Resistance::Sealed {
        return Some((index, Landing::Sealed));
    }
    let landing = match &c.outcomes[target.class] {
        RuleOutcome::Refusal(line) => Landing::Refused(*line),
        RuleOutcome::Effect {
            effect, strength, ..
        } => {
            target.effects.push(ActiveEffect {
                effect: effect.clone(),
                strength: *strength,
                by: c.name.clone(),
                until: Some(now + COMPOUND_HOLD as f64),
            });
            Landing::Took {
                effect: effect.clone(),
                strength: *strength,
            }
        }
    };
    Some((index, landing))
}

/// Lets written words fade whose time is up, bringing back what they replaced.
pub fn fade(target: &mut Target, now: f64) -> bool {
    let mut changed = false;
    let mut i = 0;
    while i < target.words.len() {
        if target.words[i].fades_at.is_some_and(|t| now >= t) {
            let gone = target.words.remove(i);
            target.effects.retain(|e| e.by != gone.spelling);
            if let Some((spelling, class)) = gone.replaced {
                target.words.insert(
                    i,
                    InscribedWord {
                        spelling,
                        class,
                        fades_at: None,
                        replaced: None,
                    },
                );
                i += 1;
            }
            changed = true;
        } else {
            i += 1;
        }
    }
    target.effects.retain(|e| e.until.is_none_or(|t| now < t));
    changed
}

/// A word on its way from a hand to a target.
#[derive(Clone, Debug, PartialEq)]
pub struct Flight {
    pub player: u32,
    pub word: usize,
    pub target: u64,
    pub from: Vec3,
    pub at: Vec3,
    pub speed: f32,
    pub charge: f32,
    pub both: bool,
    /// Typed as it charged.
    pub typed: bool,
    /// It will come out as its typo.
    pub typo: bool,
}

/// Where a ray from `from` along `dir` first comes within `radius` of `centre`, if it
/// does within `reach`.
pub fn ray_sphere(from: Vec3, dir: Vec3, centre: Vec3, radius: f32, reach: f32) -> Option<f32> {
    let to = centre - from;
    let along = to.dot(dir);
    let off = to.length_squared() - along * along;
    if off > radius * radius {
        return None;
    }
    let t = along - (radius * radius - off).sqrt();
    (t >= -radius && t <= reach).then_some(t.max(0.0))
}

/// The target the gaze is on: the nearest whose sphere it meets within reach.
pub fn aim(eye: Vec3, gaze: Vec3, targets: &[Target]) -> Option<usize> {
    targets
        .iter()
        .enumerate()
        .filter_map(|(i, t)| ray_sphere(eye, gaze, t.centre, t.radius, CAST_REACH).map(|d| (d, i)))
        .min_by(|a, b| a.0.total_cmp(&b.0))
        .map(|(_, i)| i)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rules() -> WordRules {
        let e = |effect: &str| RuleOutcome::Effect {
            effect: effect.into(),
            strength: 1.0,
            preview: 100,
        };
        WordRules {
            classes: vec!["door".into(), "stone".into()],
            words: vec![
                WordRule {
                    spelling: "OPEN".into(),
                    class: "Access".into(),
                    ink: 8.0,
                    flight_speed: 14.0,
                    outcomes: vec![e("open"), RuleOutcome::Refusal(7)],
                    weapon: RuleOutcome::Refusal(8),
                    typo: "OVEN".into(),
                    typo_outcomes: vec![e("heat"), e("heat")],
                },
                WordRule {
                    spelling: "BURNING".into(),
                    class: "Temperature".into(),
                    ink: 10.0,
                    flight_speed: 15.0,
                    outcomes: vec![e("ignite"), e("heat")],
                    weapon: e("ignite"),
                    typo: "TURNING".into(),
                    typo_outcomes: vec![e("turn"), RuleOutcome::Refusal(9)],
                },
            ],
            compounds: vec![],
        }
    }

    fn door(resistance: Resistance) -> Target {
        Target::new(
            1,
            0,
            Vec3::new(0.0, 1.0, -5.0),
            1.2,
            resistance,
            &[("LOCKED".into(), "Access".into())],
        )
    }

    #[test]
    fn a_word_replaces_its_class_and_fades_back() {
        let r = rules();
        let mut t = door(Resistance::Plain);
        assert_eq!(
            write(&r, 0, &mut t, 0.0, false, 0.0),
            Landing::Took {
                effect: "open".into(),
                strength: 1.0
            }
        );
        assert_eq!(t.words.len(), 1);
        assert_eq!(t.words[0].spelling, "OPEN");
        assert!(t.has("open"));
        // A word of another class joins it.
        write(&r, 1, &mut t, 0.0, false, 0.0);
        assert_eq!(
            t.words
                .iter()
                .map(|w| w.spelling.as_str())
                .collect::<Vec<_>>(),
            ["OPEN", "BURNING"]
        );
        // Time passes: both fade, and LOCKED comes back.
        assert!(fade(&mut t, HOLD_TAP as f64 + 1.0));
        assert_eq!(
            t.words
                .iter()
                .map(|w| w.spelling.as_str())
                .collect::<Vec<_>>(),
            ["LOCKED"]
        );
        assert!(t.effects.is_empty());
    }

    #[test]
    fn refusals_resistance_and_seals() {
        let r = rules();
        let mut stone = Target::new(2, 1, Vec3::ZERO, 1.0, Resistance::Plain, &[]);
        assert_eq!(
            write(&r, 0, &mut stone, 0.0, false, 0.0),
            Landing::Refused(7)
        );
        let mut tough = door(Resistance::Tough);
        assert_eq!(
            preview(&r, 0, &tough, 0.0, false),
            Preview::NeedsCharge(0.5)
        );
        assert_eq!(write(&r, 0, &mut tough, 0.2, false, 0.0), Landing::Resisted);
        assert!(matches!(
            write(&r, 0, &mut tough, 0.6, false, 0.0),
            Landing::Took { .. }
        ));
        // Both hands reach a tier further; a target off its guard resists a tier less.
        let mut hard = door(Resistance::Hard);
        assert_eq!(preview(&r, 0, &hard, 0.5, true), Preview::Result(100));
        hard.off_guard = true;
        assert_eq!(preview(&r, 0, &hard, 0.5, false), Preview::Result(100));
        let mut sealed = door(Resistance::Sealed);
        assert_eq!(preview(&r, 0, &sealed, 1.0, true), Preview::Sealed);
        assert_eq!(write(&r, 0, &mut sealed, 1.0, true, 0.0), Landing::Sealed);
    }

    #[test]
    fn a_charge_lasts_longer_and_costs_more() {
        let r = rules();
        let mut a = door(Resistance::Plain);
        let mut b = door(Resistance::Plain);
        write(&r, 0, &mut a, 0.0, false, 0.0);
        write(&r, 0, &mut b, 1.0, false, 0.0);
        assert!(b.words[0].fades_at > a.words[0].fades_at);
        let t = door(Resistance::Plain);
        assert!(ink_cost(&r.words[0], &t, 1.0, false) > ink_cost(&r.words[0], &t, 0.0, false));
        assert!(ink_cost(&r.words[0], &t, 0.0, true) > ink_cost(&r.words[0], &t, 0.0, false));
    }

    #[test]
    fn the_gaze_finds_the_nearest_target() {
        let near = Target::new(1, 0, Vec3::new(0.0, 0.0, -5.0), 1.0, Resistance::Plain, &[]);
        let far = Target::new(
            2,
            0,
            Vec3::new(0.0, 0.0, -15.0),
            1.0,
            Resistance::Plain,
            &[],
        );
        let ts = [far, near];
        assert_eq!(aim(Vec3::ZERO, -Vec3::Z, &ts), Some(1));
        assert_eq!(aim(Vec3::ZERO, Vec3::Z, &ts), None);
    }
}
