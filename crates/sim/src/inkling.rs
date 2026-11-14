//! The Inkling, in grey box: a formless mass of characters, fast, that splits when
//! struck. It wanders until it sees or hears a player, closes, gathers itself where it
//! can be seen doing so, lunges, and is open while it recovers. Words reach it: `COLD`
//! stops it splitting and slows it, `LIT` drives it back, `CALM` stops it fighting,
//! `BURNING` burns it. It may be found annotated with one word, which gives it that
//! word's power until a charged word of the same class replaces it.

use glam::Vec3;

use crate::melee::{marks, Creature};

/// Its kind's usual size: radius and height in metres, and health.
pub const RADIUS: f32 = 0.42;
pub const HEIGHT: f32 = 0.9;
pub const HEALTH: f32 = 70.0;
pub const POISE: f32 = 40.0;
/// How far it sees, in a cone of this half-width in radians, and how near it hears.
pub const SIGHT: f32 = 14.0;
pub const SIGHT_ARC: f32 = 1.05;
pub const HEARING: f32 = 4.0;
/// A sneaking player is heard only this near.
pub const HEARING_SNEAKING: f32 = 1.4;
/// Metres a second, closing; a little less wandering.
pub const SPEED: f32 = 5.2;
/// It strikes from this far, measured from its edge.
pub const REACH: f32 = 1.4;
/// Seconds it gathers itself before the lunge, the lunge, and the opening after.
pub const TELEGRAPH: f32 = 0.6;
pub const LUNGE: f32 = 0.22;
pub const RECOVER: f32 = 0.75;
/// A blow's damage, before its size.
pub const DAMAGE: f32 = 12.0;
/// The most Inklings that may stand at once from splitting.
pub const MOST: usize = 7;
/// A blow must do this much to split it.
pub const SPLITTING_BLOW: f32 = 6.0;
/// A part smaller than this does not split again.
pub const SMALLEST: f32 = 0.42;
/// The three annotations of the Showcase and what answers them.
pub const ANNOTATIONS: [(&str, &str); 3] = [("LOCKED", "Access"), ("BURNING", "Temperature"), ("COLD", "Temperature")];

/// A new Inkling of its usual size, with an annotation or none.
pub fn inkling(id: u64, feet: Vec3, annotation: Option<&str>) -> Creature {
    let mut c = Creature::new(id, feet, RADIUS, HEIGHT, HEALTH, POISE);
    c.class = "inkling".into();
    c.annotation = annotation.map(str::to_owned);
    c
}

#[derive(Clone, Debug, PartialEq)]
pub enum Act {
    /// Drifting about where it was left, toward a point, for a while.
    Idle { toward: Vec3, until: f32 },
    /// Closing on a player.
    Chase,
    /// Gathering itself to strike, since this long.
    Gather { t: f32 },
    /// Lunging along `dir`, since this long; it strikes once.
    Lunge { t: f32, dir: Vec3, struck: bool },
    /// Open after a lunge, for this long yet.
    Recover { t: f32 },
    /// Driven back by light, or keeping away while calm.
    Keep { away: bool },
}

/// One Inkling's mind between steps.
#[derive(Clone, Debug, PartialEq)]
pub struct Mind {
    pub id: u64,
    pub act: Act,
    pub home: Vec3,
    /// Seconds since it was made.
    pub age: f32,
}

impl Mind {
    pub fn new(id: u64, home: Vec3) -> Self {
        Self { id, act: Act::Idle { toward: home, until: 0.0 }, home, age: 0.0 }
    }
}

/// A player as an Inkling knows one.
#[derive(Clone, Copy, Debug)]
pub struct Quarry {
    pub id: u32,
    pub feet: Vec3,
    pub sneaking: bool,
    pub untouchable: bool,
}

/// A blow it lands on a player.
#[derive(Clone, Debug, PartialEq)]
pub struct Bite {
    pub player: u32,
    pub damage: f32,
    pub from: Vec3,
    /// Its annotation, whose power goes with the blow.
    pub annotation: Option<String>,
}

/// One step of one Inkling's mind and body. `ground` gives the height under a point;
/// `open` says whether it may move straight from one point to another. Rolls come from
/// `roll`, 0 to 1.
#[allow(clippy::too_many_arguments)]
pub fn think(
    c: &mut Creature,
    mind: &mut Mind,
    quarry: &[Quarry],
    ground: &dyn Fn(f32, f32) -> Option<f32>,
    open: &dyn Fn(Vec3, Vec3) -> bool,
    roll: &mut dyn FnMut() -> f32,
    dt: f32,
    bites: &mut Vec<Bite>,
) {
    mind.age += dt;
    c.telegraph = 0.0;
    if !c.alive() {
        return;
    }
    let centre = c.feet + Vec3::Y * (c.height * 0.5);
    // Whom it notices: seen in its cone and in reach of sight, or heard near enough.
    let facing = Vec3::new(c.yaw.sin(), 0.0, -c.yaw.cos());
    let nearest = quarry
        .iter()
        .map(|q| (q, Vec3::new(q.feet.x - c.feet.x, 0.0, q.feet.z - c.feet.z)))
        .min_by(|a, b| a.1.length_squared().total_cmp(&b.1.length_squared()));
    if let Some((q, to)) = nearest {
        let d = to.length();
        let seen = d < SIGHT && (d < 0.01 || to.normalize().dot(facing) > SIGHT_ARC.cos()) && open(centre, q.feet + Vec3::Y * 1.5);
        let heard = d < if q.sneaking { HEARING_SNEAKING } else { HEARING };
        if seen || heard {
            c.aware = true;
        }
    }
    if c.staggered > 0.0 {
        if matches!(mind.act, Act::Gather { .. } | Act::Lunge { .. }) {
            mind.act = Act::Recover { t: RECOVER };
        }
        return;
    }
    let slowed = c.marks & marks::SLOWED != 0;
    let pace = if slowed { 0.45 } else { 1.0 } * (1.0 + (1.0 - c.size).max(0.0) * 0.6);
    let calm = c.marks & marks::PACIFIED != 0;
    let dazzled = c.marks & marks::DAZZLED != 0;

    // Words come first: light drives it back, calm keeps it from fighting.
    if dazzled || calm {
        mind.act = Act::Keep { away: dazzled };
    } else if matches!(mind.act, Act::Keep { .. }) {
        mind.act = Act::Chase;
    }
    if !c.aware && !matches!(mind.act, Act::Idle { .. } | Act::Keep { .. }) {
        mind.act = Act::Idle { toward: mind.home, until: 0.0 };
    }
    if c.aware && matches!(mind.act, Act::Idle { .. }) {
        mind.act = Act::Chase;
    }

    let mut step = Vec3::ZERO;
    let target = nearest.map(|(q, to)| (*q, to));
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
                step = to.normalize() * SPEED * 0.25 * pace * dt;
            }
        }
        Act::Keep { away } => {
            if let Some((_, to)) = target {
                let d = to.length();
                if *away {
                    step = -to.normalize_or_zero() * SPEED * 0.8 * pace * dt;
                } else if d < 3.0 {
                    step = -to.normalize_or_zero() * SPEED * 0.2 * pace * dt;
                }
                if d > 0.01 {
                    turn_to(c, to, dt * 6.0);
                }
            }
        }
        Act::Chase => {
            if let Some((_, to)) = target {
                let d = to.length() - c.radius;
                turn_to(c, to, dt * 10.0);
                if d > REACH * 0.85 {
                    // It weaves as it comes.
                    let side = Vec3::new(-to.z, 0.0, to.x).normalize_or_zero() * (mind.age * 3.1 + c.id as f32).sin() * 0.35;
                    step = (to.normalize_or_zero() + side).normalize_or_zero() * SPEED * pace * dt;
                } else {
                    mind.act = Act::Gather { t: 0.0 };
                }
            }
        }
        Act::Gather { t } => {
            *t += dt;
            let length = TELEGRAPH / pace.min(1.0);
            c.telegraph = (*t / length).min(1.0);
            if let Some((_, to)) = target {
                turn_to(c, to, dt * 4.0);
            }
            if *t >= length {
                let dir = Vec3::new(c.yaw.sin(), 0.0, -c.yaw.cos());
                mind.act = Act::Lunge { t: 0.0, dir, struck: false };
            }
        }
        Act::Lunge { t, dir, struck } => {
            *t += dt;
            c.telegraph = 1.0;
            step = *dir * (REACH + 0.4) / LUNGE * dt;
            if !*struck {
                for q in quarry {
                    let to = Vec3::new(q.feet.x - c.feet.x, 0.0, q.feet.z - c.feet.z);
                    if to.length() < c.radius + 0.45 && to.normalize_or_zero().dot(*dir) > 0.2 {
                        *struck = true;
                        if !q.untouchable {
                            bites.push(Bite { player: q.id, damage: DAMAGE * (0.5 + 0.5 * c.size), from: centre, annotation: c.annotation.clone() });
                        }
                        break;
                    }
                }
            }
            if *t >= LUNGE {
                mind.act = Act::Recover { t: RECOVER };
            }
        }
        Act::Recover { t } => {
            *t -= dt;
            if *t <= 0.0 {
                mind.act = Act::Chase;
            }
        }
    }
    // Move, on the ground, never through what is still.
    if step != Vec3::ZERO {
        let to = c.feet + step;
        let knee = Vec3::Y * 0.35;
        if open(c.feet + knee, to + knee + step.normalize_or_zero() * c.radius) {
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
    if let Some(h) = ground(c.feet.x, c.feet.z) {
        c.feet.y = h;
    }
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

/// A struck Inkling splits: a smaller part breaks off it, unless it is slowed by cold,
/// too small, or there are already as many as there may be. Returns the part.
pub fn split(c: &mut Creature, id: u64, standing: usize, side: Vec3) -> Option<Creature> {
    if !c.alive() || c.marks & marks::SLOWED != 0 || c.size < SMALLEST || standing >= MOST {
        return None;
    }
    let mut part = inkling(id, c.feet + side * (c.radius + 0.35), None);
    part.size = c.size * 0.62;
    part.radius = RADIUS * part.size;
    part.height = HEIGHT * part.size;
    part.health = c.health * 0.4;
    part.max_health = part.health;
    part.aware = true;
    c.health *= 0.6;
    c.size *= 0.8;
    c.radius = RADIUS * c.size;
    c.height = HEIGHT * c.size;
    Some(part)
}
