//! The minds of creatures, one kind of mind for every species, set by its numbers. A
//! creature wanders until it sees or hears a player, and remembers where it last did;
//! calls its kin when it notices; closes; waits its turn while another of its pack
//! strikes, circling to the player's side; gathers itself where it can be seen, strikes,
//! and is open while it recovers; falls back when badly hurt. Words reach it: on itself
//! (calm, light, fire, cold), on its kin (it keeps clear of one that burns), and on the
//! place (it keeps from fire, and a thing of ink from light).

use glam::Vec3;

use crate::melee::{marks, Creature};

/// What a species is like in a fight.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Species {
    pub key: &'static str,
    /// Metres a second, closing.
    pub speed: f32,
    /// It strikes from this far, measured from its edge.
    pub reach: f32,
    /// Seconds it gathers itself before a blow, how long the blow takes, how far it
    /// carries, and how long it is open after.
    pub telegraph: f32,
    pub strike: f32,
    pub lunge: f32,
    pub recover: f32,
    pub damage: f32,
    /// How far it sees, in a cone this many radians either side; how near it hears, and
    /// a crouching player.
    pub sight: f32,
    pub sight_arc: f32,
    pub hearing: f32,
    pub hearing_crouched: f32,
    /// Seconds it keeps after a player it can no longer see or hear.
    pub memory: f32,
    /// It hunts with its kind: one strikes while the others circle to the sides.
    pub pack: bool,
    /// It falls back when its health is below this share of its most, for a while.
    pub retreats_below: f32,
    /// When it notices a player, its kin within this many metres notice too. 0: it does
    /// not call.
    pub calls: f32,
    /// A thing of ink: light drives it back, and it keeps from lit lamps.
    pub fears_light: bool,
    /// It weaves from side to side as it comes.
    pub weaves: bool,
    /// The annotations it may be found with.
    pub annotations: &'static [&'static str],
}

/// One creature's mind between steps.
#[derive(Clone, Debug, PartialEq)]
pub struct Mind {
    pub id: u64,
    pub act: Act,
    pub home: Vec3,
    /// Seconds since it was made.
    pub age: f32,
    /// Where it last saw or heard a player, and how long ago.
    pub last_seen: Option<(Vec3, f32)>,
    /// It may fall back again once this is 0.
    pub retreat_cooldown: f32,
}

impl Mind {
    pub fn new(id: u64, home: Vec3) -> Self {
        Self { id, act: Act::Idle { toward: home, until: 0.0 }, home, age: 0.0, last_seen: None, retreat_cooldown: 0.0 }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Act {
    /// Drifting about where it was left, toward a point, for a while.
    Idle { toward: Vec3, until: f32 },
    /// Closing on a player, or on where one was last known to be.
    Chase,
    /// Waiting its turn to strike, circling at the player's side.
    Circle,
    /// Gathering itself to strike, since this long.
    Gather { t: f32 },
    /// Striking along `dir`, since this long; it strikes once.
    Strike { t: f32, dir: Vec3, struck: bool },
    /// Open after a blow, for this long yet.
    Recover { t: f32 },
    /// Falling back, badly hurt, for this long yet.
    Retreat { t: f32 },
    /// Driven back by light, or keeping away while calm.
    Keep { away: bool },
}

impl Act {
    /// Striking or about to: its turn in the pack.
    pub fn attacking(&self) -> bool {
        matches!(self, Act::Gather { .. } | Act::Strike { .. })
    }
}

/// A player as a creature knows one.
#[derive(Clone, Copy, Debug)]
pub struct Quarry {
    pub id: u32,
    pub feet: Vec3,
    pub crouching: bool,
    pub untouchable: bool,
}

/// What a creature knows of its surroundings this step.
pub struct Around<'a> {
    /// Its kin: where each stands, what words do to it, and whether it is striking.
    pub kin: &'a [(u64, Vec3, u32, bool)],
    /// The creature whose turn it is to strike at each player, if any. A creature that
    /// takes its turn this step claims it here, so that no other takes it too.
    pub turn: &'a std::cell::RefCell<Vec<(u32, u64)>>,
    /// Places to keep from: a fire, or for a thing of ink a lit lamp. Centre and radius.
    pub hazards: &'a [(Vec3, f32, bool)],
    /// The height of the ground under a point.
    pub ground: &'a dyn Fn(f32, f32) -> Option<f32>,
    /// Whether nothing still stands between two points.
    pub open: &'a dyn Fn(Vec3, Vec3) -> bool,
}

/// A blow a creature lands on a player.
#[derive(Clone, Debug, PartialEq)]
pub struct Bite {
    pub player: u32,
    pub damage: f32,
    pub from: Vec3,
    /// Its annotation, whose power goes with the blow.
    pub annotation: Option<String>,
}

/// What a step of thought led to, beyond moving.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Outcome {
    /// It noticed a player this step, and calls its kin within this range.
    pub called: Option<f32>,
}

/// One step of one creature's mind and body. Rolls come from `roll`, 0 to 1.
#[allow(clippy::too_many_arguments)]
pub fn think(
    s: &Species,
    c: &mut Creature,
    mind: &mut Mind,
    quarry: &[Quarry],
    around: &Around<'_>,
    roll: &mut dyn FnMut() -> f32,
    dt: f32,
    bites: &mut Vec<Bite>,
) -> Outcome {
    let mut out = Outcome::default();
    mind.age += dt;
    mind.retreat_cooldown = (mind.retreat_cooldown - dt).max(0.0);
    c.telegraph = 0.0;
    if !c.alive() {
        return out;
    }
    let centre = c.feet + Vec3::Y * (c.height * 0.5);
    // Whom it notices: seen in its cone and in reach of sight, or heard near enough.
    let facing = Vec3::new(c.yaw.sin(), 0.0, -c.yaw.cos());
    let nearest = quarry
        .iter()
        .map(|q| (*q, Vec3::new(q.feet.x - c.feet.x, 0.0, q.feet.z - c.feet.z)))
        .min_by(|a, b| a.1.length_squared().total_cmp(&b.1.length_squared()));
    let was_aware = c.aware;
    let mut sensed = false;
    if let Some((q, to)) = nearest {
        let d = to.length();
        let seen = d < s.sight && (d < 0.01 || to.normalize().dot(facing) > s.sight_arc.cos()) && (around.open)(centre, q.feet + Vec3::Y * 1.5);
        let heard = d < if q.crouching { s.hearing_crouched } else { s.hearing };
        // Once it is after a player, it keeps them in view without facing them.
        let kept = c.aware && d < s.sight && (around.open)(centre, q.feet + Vec3::Y * 1.5);
        if seen || heard || kept {
            c.aware = true;
            sensed = true;
            mind.last_seen = Some((q.feet, 0.0));
        }
    }
    if c.aware && !was_aware && s.calls > 0.0 {
        out.called = Some(s.calls);
    }
    // Memory: an unseen player is followed to where they were, then forgotten.
    if !sensed {
        if let Some((at, age)) = &mut mind.last_seen {
            *age += dt;
            if *age > s.memory {
                mind.last_seen = None;
                c.aware = false;
            } else {
                let _ = at;
            }
        }
    }
    if c.staggered > 0.0 {
        if mind.act.attacking() {
            mind.act = Act::Recover { t: s.recover };
        }
        return out;
    }
    let slowed = c.marks & marks::SLOWED != 0;
    let pace = if slowed { 0.45 } else { 1.0 } * (1.0 + (1.0 - c.size).max(0.0) * 0.6);
    let calm = c.marks & marks::PACIFIED != 0;
    let dazzled = c.marks & marks::DAZZLED != 0;
    let burning = c.marks & marks::BURNING != 0;

    // Words come first: light drives a thing of ink back, calm keeps it from fighting.
    if (dazzled && s.fears_light) || calm {
        mind.act = Act::Keep { away: dazzled };
    } else if matches!(mind.act, Act::Keep { .. }) {
        mind.act = Act::Chase;
    }
    // Badly hurt, it falls back for a while.
    if c.aware && s.retreats_below > 0.0 && c.health < c.max_health * s.retreats_below && mind.retreat_cooldown == 0.0 && !mind.act.attacking() {
        mind.act = Act::Retreat { t: 3.0 };
        mind.retreat_cooldown = 10.0;
    }
    if !c.aware && !matches!(mind.act, Act::Idle { .. } | Act::Keep { .. }) {
        mind.act = Act::Idle { toward: mind.home, until: 0.0 };
    }
    if c.aware && matches!(mind.act, Act::Idle { .. }) {
        mind.act = Act::Chase;
    }

    // The player it is after: the nearest it senses, or where it last knew them to be.
    let goal = match (nearest, mind.last_seen) {
        (Some((q, to)), _) if sensed => Some((Some(q), to)),
        (_, Some((at, _))) => Some((None, Vec3::new(at.x - c.feet.x, 0.0, at.z - c.feet.z))),
        _ => None,
    };
    let whose_turn = |player: u32| around.turn.borrow().iter().find(|(p, _)| *p == player).map(|(_, id)| *id);

    let mut step = Vec3::ZERO;
    match &mut mind.act {
        Act::Idle { toward, until } => {
            *until -= dt;
            if *until <= 0.0 {
                let a = roll() * std::f32::consts::TAU;
                let r = roll() * 3.0;
                *toward = mind.home + Vec3::new(a.cos() * r, 0.0, a.sin() * r);
                *until = 2.0 + roll() * 3.0;
            }
            let to = Vec3::new(toward.x - c.feet.x, 0.0, toward.z - c.feet.z);
            if to.length() > 0.2 {
                step = to.normalize() * s.speed * 0.25 * pace * dt;
                turn_to(c, to, dt * 3.0);
            }
        }
        Act::Keep { away } => {
            if let Some((_, to)) = goal {
                let d = to.length();
                if *away {
                    step = -to.normalize_or_zero() * s.speed * 0.8 * pace * dt;
                } else if d < 3.0 {
                    step = -to.normalize_or_zero() * s.speed * 0.2 * pace * dt;
                }
                if d > 0.01 {
                    turn_to(c, to, dt * 6.0);
                }
            }
        }
        Act::Retreat { t } => {
            *t -= dt;
            if let Some((_, to)) = goal {
                step = -to.normalize_or_zero() * s.speed * 0.7 * pace * dt;
                turn_to(c, to, dt * 6.0);
            }
            if *t <= 0.0 {
                mind.act = Act::Chase;
            }
        }
        Act::Chase | Act::Circle => {
            if let Some((q, to)) = goal {
                let d = to.length() - c.radius;
                turn_to(c, to, dt * 10.0);
                let free = q.is_some_and(|q| whose_turn(q.id).is_none_or(|id| id == c.id));
                if d > s.reach * 0.85 {
                    // Closing; weaving as it comes.
                    let side = Vec3::new(-to.z, 0.0, to.x).normalize_or_zero() * if s.weaves { (mind.age * 3.1 + c.id as f32).sin() * 0.35 } else { 0.0 };
                    step = (to.normalize_or_zero() + side).normalize_or_zero() * s.speed * pace * dt;
                    mind.act = Act::Chase;
                } else if q.is_none() {
                    // At the place it last knew of, and nobody there.
                    mind.last_seen = None;
                    c.aware = false;
                } else if free && !calm && !(dazzled && s.fears_light) {
                    mind.act = Act::Gather { t: 0.0 };
                    if let Some(q) = q {
                        let mut turn = around.turn.borrow_mut();
                        if !turn.iter().any(|(p, _)| *p == q.id) {
                            turn.push((q.id, c.id));
                        }
                    }
                } else if s.pack {
                    // Another of its pack is striking: it circles to the player's side.
                    let slot = if c.id % 2 == 0 { 1.0 } else { -1.0 };
                    let round = Vec3::new(-to.z, 0.0, to.x).normalize_or_zero() * slot;
                    let keep = to.normalize_or_zero() * (to.length() - (s.reach + 1.2));
                    step = (round * 0.8 + keep).clamp_length_max(1.0) * s.speed * 0.5 * pace * dt;
                    mind.act = Act::Circle;
                }
            }
        }
        Act::Gather { t } => {
            *t += dt;
            let length = s.telegraph / pace.min(1.0);
            c.telegraph = (*t / length).min(1.0);
            if let Some((_, to)) = goal {
                turn_to(c, to, dt * 4.0);
            }
            if *t >= length {
                let dir = Vec3::new(c.yaw.sin(), 0.0, -c.yaw.cos());
                mind.act = Act::Strike { t: 0.0, dir, struck: false };
            }
        }
        Act::Strike { t, dir, struck } => {
            *t += dt;
            c.telegraph = 1.0;
            let near = quarry.iter().any(|q| Vec3::new(q.feet.x - c.feet.x, 0.0, q.feet.z - c.feet.z).length() < c.radius + crate::character::RADIUS + 0.05);
            if !near {
                step = *dir * s.lunge / s.strike * dt;
            }
            if !*struck {
                for q in quarry {
                    let to = Vec3::new(q.feet.x - c.feet.x, 0.0, q.feet.z - c.feet.z);
                    if to.length() < c.radius + s.reach.min(0.6) && to.normalize_or_zero().dot(*dir) > 0.2 {
                        *struck = true;
                        if !q.untouchable {
                            bites.push(Bite { player: q.id, damage: s.damage * (0.5 + 0.5 * c.size), from: centre, annotation: c.annotation.clone() });
                        }
                        break;
                    }
                }
            }
            if *t >= s.strike {
                mind.act = Act::Recover { t: s.recover };
            }
        }
        Act::Recover { t } => {
            *t -= dt;
            if *t <= 0.0 {
                mind.act = Act::Chase;
            }
        }
    }
    // Burning, it thrashes: half its steps go astray.
    if burning && !matches!(mind.act, Act::Strike { .. }) && roll() < 0.5 {
        let a = roll() * std::f32::consts::TAU;
        step = Vec3::new(a.cos(), 0.0, a.sin()) * s.speed * 0.6 * dt;
    }
    // It keeps from its burning kin, from fire, and a thing of ink from light.
    let mut away = Vec3::ZERO;
    for (id, at, kin_marks, _) in around.kin {
        if *id != c.id && kin_marks & marks::BURNING != 0 {
            let off = Vec3::new(c.feet.x - at.x, 0.0, c.feet.z - at.z);
            let d = off.length();
            if d < 2.0 {
                away += off.normalize_or_zero() * (2.0 - d);
            }
        }
    }
    if away != Vec3::ZERO {
        step += away.clamp_length_max(1.0) * s.speed * 0.6 * dt;
    }
    // Into fire, or for a thing of ink into light, it does not go: whatever of its step
    // leads further in is taken away, and it is eased out.
    for (at, radius, light) in around.hazards {
        if *light && !s.fears_light {
            continue;
        }
        let off = Vec3::new(c.feet.x - at.x, 0.0, c.feet.z - at.z);
        let d = off.length();
        if d < *radius {
            let out = off.normalize_or(Vec3::X);
            let inward = step.dot(out);
            if inward < 0.0 {
                step -= out * inward;
            }
            step += out * s.speed * 0.6 * dt * (*radius - d) / radius;
        }
    }
    // Move, on the ground, never through what is still.
    if step != Vec3::ZERO {
        let to = c.feet + step;
        let knee = Vec3::Y * 0.35;
        if (around.open)(c.feet + knee, to + knee + step.normalize_or_zero() * c.radius) {
            c.feet = to;
        }
    }
    // It never passes into a player's body.
    for q in quarry {
        let off = Vec3::new(c.feet.x - q.feet.x, 0.0, c.feet.z - q.feet.z);
        let least = c.radius + crate::character::RADIUS;
        if off.length() < least {
            let dir = off.normalize_or(Vec3::new(-c.yaw.sin(), 0.0, c.yaw.cos()));
            c.feet = Vec3::new(q.feet.x, c.feet.y, q.feet.z) + dir * least;
        }
    }
    if let Some(h) = (around.ground)(c.feet.x, c.feet.z) {
        c.feet.y = h;
    }
    out
}

fn turn_to(c: &mut Creature, to: Vec3, rate: f32) {
    let want = to.x.atan2(-to.z);
    let mut d = want - c.yaw;
    while d > std::f32::consts::PI {
        d -= std::f32::consts::TAU;
    }
    while d < -std::f32::consts::PI {
        d += std::f32::consts::TAU;
    }
    c.yaw += d.clamp(-rate, rate);
}

/// The Showcase's annotations: the word, its class (a charged word of which replaces
/// it), and the word that answers it.
pub const ANNOTATIONS: [(&str, &str, &str); 4] = [("LOCKED", "Access", "OPEN"), ("DARK", "Light", "LIT"), ("BURNING", "Temperature", "COLD"), ("COLD", "Temperature", "BURNING")];

/// How near a creature annotated DARK puts out a lit lamp, and how far it can be seen.
pub const DARK_REACH: f32 = 6.0;
pub const DARK_SEEN: f32 = 5.0;

/// Every species of the Showcase, by its key.
pub fn species(key: &str) -> Option<&'static Species> {
    SPECIES.iter().find(|s| s.key == key)
}

/// The species, until each is built from its own sheet (S04-T08 to T10).
pub const SPECIES: &[Species] = &[crate::inkling::SPECIES];

#[cfg(test)]
mod tests {
    use super::*;

    const HOUND: Species = Species { key: "test", retreats_below: 0.4, fears_light: false, ..crate::inkling::SPECIES };

    fn run(s: &Species, c: &mut Creature, mind: &mut Mind, q: Quarry, hazards: &[(Vec3, f32, bool)], steps: u32) -> Vec<Bite> {
        let ground = |_: f32, _: f32| Some(0.0);
        let open = |_: Vec3, _: Vec3| true;
        let turn = std::cell::RefCell::new(Vec::new());
        let around = Around { kin: &[], turn: &turn, hazards, ground: &ground, open: &open };
        let mut n = 0u32;
        let mut roll = || {
            n = n.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
            (n >> 8) as f32 / (1u32 << 24) as f32
        };
        let mut bites = Vec::new();
        for _ in 0..steps {
            think(s, c, mind, &[q], &around, &mut roll, 1.0 / 60.0, &mut bites);
        }
        bites
    }

    fn quarry(at: Vec3) -> Quarry {
        Quarry { id: 0, feet: at, crouching: false, untouchable: false }
    }

    #[test]
    fn badly_hurt_it_falls_back_and_then_comes_again() {
        let mut c = Creature::new(1, Vec3::new(0.0, 0.0, -3.0), 0.4, 0.9, 100.0, 40.0);
        c.yaw = std::f32::consts::PI;
        c.health = 30.0;
        let mut mind = Mind::new(1, c.feet);
        run(&HOUND, &mut c, &mut mind, quarry(Vec3::ZERO), &[], 60);
        assert!(matches!(mind.act, Act::Retreat { .. }), "{:?}", mind.act);
        assert!(c.feet.z < -4.0, "it fell back: {}", c.feet);
        run(&HOUND, &mut c, &mut mind, quarry(Vec3::ZERO), &[], 180);
        assert!(!matches!(mind.act, Act::Retreat { .. }), "and came again: {:?}", mind.act);
    }

    #[test]
    fn it_keeps_from_fire_and_a_thing_of_ink_from_light() {
        // A fire between it and the player: it goes round, never into it.
        let s = crate::inkling::SPECIES;
        for (hazard, fears) in [((Vec3::new(0.0, 0.0, -3.0), 2.5, false), true), ((Vec3::new(0.0, 0.0, -3.0), 4.0, true), true)] {
            let mut c = Creature::new(1, Vec3::new(0.0, 0.0, -8.0), 0.4, 0.9, 100.0, 40.0);
            c.yaw = std::f32::consts::PI;
            let mut mind = Mind::new(1, c.feet);
            let mut nearest = f32::MAX;
            for _ in 0..240 {
                run(&s, &mut c, &mut mind, quarry(Vec3::ZERO), &[hazard], 1);
                nearest = nearest.min(c.feet.distance(hazard.0));
            }
            assert!(nearest > hazard.1 * 0.45 || !fears, "it kept from {:?}: came within {nearest}", hazard);
        }
    }

    #[test]
    fn it_follows_to_where_it_last_knew_a_player_was_then_forgets() {
        let blind = |_: Vec3, _: Vec3| false;
        let ground = |_: f32, _: f32| Some(0.0);
        let mut c = Creature::new(1, Vec3::new(0.0, 0.0, -10.0), 0.4, 0.9, 100.0, 40.0);
        c.aware = true;
        let mut mind = Mind::new(1, c.feet);
        mind.last_seen = Some((Vec3::new(5.0, 0.0, -10.0), 0.0));
        mind.act = Act::Chase;
        let turn = std::cell::RefCell::new(Vec::new());
        let mut roll = || 0.5;
        let _ = &blind;
        let mut bites = Vec::new();
        // The player is far off and out of sight; it goes to the last place it knew.
        let far = Quarry { id: 0, feet: Vec3::new(60.0, 0.0, 60.0), crouching: false, untouchable: false };
        let mut furthest = f32::MIN;
        for _ in 0..120 {
            think(&crate::inkling::SPECIES, &mut c, &mut mind, &[far], &Around { kin: &[], turn: &turn, hazards: &[], ground: &ground, open: &|a: Vec3, b: Vec3| (b - a).length() < 30.0 && b.x < 30.0 }, &mut roll, 1.0 / 60.0, &mut bites);
            furthest = furthest.max(c.feet.x);
        }
        assert!(furthest > 3.0, "it went to where the player was: {furthest}");
        for _ in 0..60 * 8 {
            think(&crate::inkling::SPECIES, &mut c, &mut mind, &[far], &Around { kin: &[], turn: &turn, hazards: &[], ground: &ground, open: &|a: Vec3, b: Vec3| (b - a).length() < 30.0 && b.x < 30.0 }, &mut roll, 1.0 / 60.0, &mut bites);
        }
        assert!(!c.aware && mind.last_seen.is_none(), "and forgot them");
    }

    #[test]
    fn it_keeps_clear_of_burning_kin() {
        let ground = |_: f32, _: f32| Some(0.0);
        let open = |_: Vec3, _: Vec3| true;
        let turn = std::cell::RefCell::new(Vec::new());
        let kin = [(2u64, Vec3::new(0.4, 0.0, 0.0), marks::BURNING, false)];
        let around = Around { kin: &kin, turn: &turn, hazards: &[], ground: &ground, open: &open };
        let mut c = Creature::new(1, Vec3::ZERO, 0.4, 0.9, 100.0, 40.0);
        let mut mind = Mind::new(1, c.feet);
        let mut roll = || 0.5;
        let mut bites = Vec::new();
        let far = Quarry { id: 0, feet: Vec3::new(0.0, 0.0, 200.0), crouching: false, untouchable: false };
        for _ in 0..90 {
            think(&crate::inkling::SPECIES, &mut c, &mut mind, &[far], &around, &mut roll, 1.0 / 60.0, &mut bites);
        }
        assert!(c.feet.distance(kin[0].1) > 1.6, "it moved clear of its burning kin: {}", c.feet);
    }
}
