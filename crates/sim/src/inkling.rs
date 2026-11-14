//! The Inkling, in grey box: a formless mass of characters, fast, that splits when
//! struck. Its mind is the one every creature shares (`crate::ai`), with its numbers:
//! it lunges, and is open while it recovers; `COLD` stops it splitting and slows it,
//! `LIT` drives it back, `CALM` stops it fighting, `BURNING` burns it. It may be found
//! annotated with one word, which gives it that word's power until a charged word of the
//! same class replaces it.

use glam::Vec3;

use crate::melee::{marks, Creature};

/// Its kind's usual size: radius and height in metres, and health.
pub const RADIUS: f32 = 0.42;
pub const HEIGHT: f32 = 0.9;
pub const HEALTH: f32 = 120.0;
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
pub const ANNOTATIONS: [&str; 4] = ["LOCKED", "DARK", "BURNING", "COLD"];

/// The Inkling's numbers for the mind every creature shares (`crate::ai`).
pub const SPECIES: crate::ai::Species = crate::ai::Species {
    key: "inkling",
    speed: SPEED,
    reach: REACH,
    telegraph: TELEGRAPH,
    strike: LUNGE,
    lunge: REACH + 0.4,
    recover: RECOVER,
    damage: DAMAGE,
    sight: SIGHT,
    sight_arc: SIGHT_ARC,
    hearing: HEARING,
    hearing_crouched: HEARING_SNEAKING,
    memory: 6.0,
    pack: true,
    retreats_below: 0.0,
    calls: 10.0,
    fears_light: true,
    weaves: true,
    annotations: &ANNOTATIONS,
};

/// A new Inkling of its usual size, with an annotation or none.
pub fn inkling(id: u64, feet: Vec3, annotation: Option<&str>) -> Creature {
    let mut c = Creature::new(id, feet, RADIUS, HEIGHT, HEALTH, POISE);
    c.class = "inkling".into();
    c.annotation = annotation.map(str::to_owned);
    c.elite = annotation.is_some();
    c
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
    part.drops = false;
    c.health *= 0.6;
    c.size *= 0.8;
    c.radius = RADIUS * c.size;
    c.height = HEIGHT * c.size;
    Some(part)
}
