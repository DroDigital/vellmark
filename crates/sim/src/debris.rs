//! Loose letters. When a thing breaks, it comes apart into what it was made of: tens
//! of thousands of characters, each a small body that falls, strikes the world,
//! bounces, slides, and comes to rest on the floor.
//!
//! Each letter in flight is simulated on its own, against the ground, the scene's
//! boxes, and the letters already at rest. Letters at rest lie on one another and build
//! heaps: a heap is kept as the height of its top in each cell of a fine map, a letter
//! landing on it lies on that top, and a letter that would make the heap steeper than
//! letters can stand slides down to the foot of the slope first. Two letters in flight
//! never strike each other; in a spill, nothing would show it.
//!
//! The simulation is deterministic, like the rest of the rules, so that a broken thing
//! can later be made whole along the paths its letters fell.

use std::collections::HashMap;

use glam::{Quat, Vec3};

/// How long a letter takes to come home when its thing is made whole.
pub const MEND_SECONDS: f32 = 1.6;
/// Seconds a metre: how fast a thing being mended knits outward from the word.
pub const MEND_SPREAD: f32 = 0.11;
/// The most loose letters alive at once. Older ones at rest are swept first.
pub const MAX_LETTERS: usize = 60_000;
/// The most letters one thing falls into, however big it is: a door is about this.
pub const THING_LETTERS: usize = 24_000;

/// The share of a thing's cells that hold a letter when it breaks, so that the biggest
/// fall into no more than [`THING_LETTERS`].
pub fn keep_for(boxes: impl Iterator<Item = (Vec3, Vec3)>) -> f32 {
    let spacing = LETTER_HALF * 2.0 * 1.25;
    let cells: f32 = boxes.map(|(min, max)| ((max - min) / spacing).floor().max(Vec3::ONE)).map(|n| n.x * n.y * n.z).sum();
    (THING_LETTERS as f32 / cells.max(1.0)).min(0.7)
}
/// Half the side of a letter, in metres. Letters are the size of the 1.28 cm cells.
pub const LETTER_HALF: f32 = 0.0064;

const GRAVITY: f32 = -9.81;
/// How much of its speed a letter keeps along the normal when it strikes something.
const BOUNCE: f32 = 0.32;
/// How much of its speed along a surface it keeps on striking it.
const SLIDE: f32 = 0.62;
/// Below this speed, on something it can lie on, a letter comes to rest.
const REST_SPEED: f32 = 0.22;
/// The side of a cell of the heap map: one letter.
const PILE_CELL: f32 = LETTER_HALF * 2.0;
/// How much a letter adds to the heap it lies on.
pub const LETTER_THICK: f32 = 0.003;
/// The most one cell of a heap may stand above its neighbour: about 35 degrees, which
/// is how steep a heap of flat flakes stands.
const REPOSE: f32 = PILE_CELL * 0.7;
/// The furthest a letter slides down a heap, in cells, before it lies where it is.
const MOST_SLIDE: usize = 64;

/// One heap in one cell of the map: what it stands on, and its top.
#[derive(Clone, Copy, Debug, PartialEq)]
struct Pile {
    base: f32,
    top: f32,
}

/// An axis-aligned box in the world that letters land on and bounce from.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Solid {
    pub min: Vec3,
    pub max: Vec3,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Letter {
    pub position: Vec3,
    pub velocity: Vec3,
    pub rotation: Quat,
    /// Radians a second about `spin_axis`.
    pub spin: f32,
    pub spin_axis: Vec3,
    /// The material it broke from, which gives it its colour and its alphabet.
    pub material: u32,
    /// Chooses its character, for good.
    pub seed: u32,
    pub resting: bool,
    /// Where it stood in the thing before the thing broke.
    pub home: Vec3,
    /// The thing it broke from, by its permanent ID.
    pub owner: u64,
    /// Which of the thing's boxes it was part of.
    pub piece: u32,
    /// On its way home, being made whole: where it set out from, and how far along it
    /// is in seconds, below 0 while it waits its turn.
    pub returning: Option<(Vec3, f32)>,
    /// The height of what the heap it lies in stands on, once it is at rest.
    pub floor: f32,
}

#[derive(Clone, Debug, Default)]
pub struct Debris {
    pub letters: Vec<Letter>,
    solids: Vec<Solid>,
    awake: usize,
    /// The heaps, by cell. A cell can hold more than one: under a bench and on it.
    piles: HashMap<(i32, i32), Vec<Pile>>,
    /// Letters at rest were swept, and the heaps must be measured again.
    piles_stale: bool,
    /// The ground under them, if there is any other than the level at 0.
    ground: Option<std::sync::Arc<dyn crate::Ground>>,
}

fn cell_of(p: Vec3) -> (i32, i32) {
    ((p.x / PILE_CELL).floor() as i32, (p.z / PILE_CELL).floor() as i32)
}

impl Debris {
    /// The heap in a cell standing on the surface at `base`, if there is one.
    fn pile(&self, cell: (i32, i32), base: f32) -> Option<Pile> {
        self.piles.get(&cell)?.iter().copied().find(|p| (p.base - base).abs() < 0.01)
    }

    /// The heap in a cell that a letter at height `y` would land on: the highest one
    /// whose foot is below it.
    fn pile_under(&self, cell: (i32, i32), y: f32) -> Option<Pile> {
        self.piles.get(&cell)?.iter().copied().filter(|p| p.base <= y + 0.005).max_by(|a, b| a.base.total_cmp(&b.base))
    }

    fn set_pile(&mut self, cell: (i32, i32), pile: Pile) {
        let piles = self.piles.entry(cell).or_default();
        match piles.iter_mut().find(|p| (p.base - pile.base).abs() < 0.01) {
            Some(p) => p.top = p.top.max(pile.top),
            None => piles.push(pile),
        }
    }

    /// The highest surface at the middle of a cell that is no higher than `y`, or `None`
    /// if something solid stands there at that height.
    fn surface(&self, cell: (i32, i32), y: f32) -> Option<f32> {
        let x = (cell.0 as f32 + 0.5) * PILE_CELL;
        let z = (cell.1 as f32 + 0.5) * PILE_CELL;
        let mut best = self.ground_at(x, z);
        for s in &self.solids {
            if x < s.min.x - LETTER_HALF || x > s.max.x + LETTER_HALF || z < s.min.z - LETTER_HALF || z > s.max.z + LETTER_HALF {
                continue;
            }
            if s.max.y <= y + 1.0e-3 {
                best = best.max(s.max.y);
            } else if s.min.y < y + LETTER_HALF {
                return None;
            }
        }
        Some(best)
    }

    /// Lays a letter that has stopped on the surface or heap at `top` to rest: it
    /// slides down the heap while the heap beside it is lower by more than letters can
    /// stand, then lies flat on top.
    fn lay(&mut self, i: usize, base: f32, top: f32) {
        let mut position = self.letters[i].position;
        let mut cell = cell_of(position);
        let (mut base, mut top) = (base, top);
        for _ in 0..MOST_SLIDE {
            let mut lowest: Option<((i32, i32), f32, f32)> = None;
            for (dx, dz) in [(-1, 0), (1, 0), (0, -1), (0, 1), (-1, -1), (1, -1), (-1, 1), (1, 1)] {
                let next = (cell.0 + dx, cell.1 + dz);
                let Some(surface) = self.surface(next, top) else { continue };
                let next_top = self.pile(next, surface).map_or(surface, |p| p.top);
                // Corner cells are further away, so they must be lower still.
                let reach = if dx != 0 && dz != 0 { REPOSE * std::f32::consts::SQRT_2 } else { REPOSE };
                if next_top < top - reach && lowest.is_none_or(|l| next_top < l.2) {
                    lowest = Some((next, surface, next_top));
                }
            }
            let Some((next, next_base, next_top)) = lowest else { break };
            position.x += (next.0 - cell.0) as f32 * PILE_CELL;
            position.z += (next.1 - cell.1) as f32 * PILE_CELL;
            (cell, base, top) = (next, next_base, next_top);
        }
        let letter = &mut self.letters[i];
        // Flat, face up, turned whichever way it happened to land, and tipped a little,
        // as nothing lies quite flat on a heap of other things.
        let turn = (letter.seed % 6283) as f32 / 1000.0;
        let tip = ((letter.seed >> 13) % 100) as f32 / 100.0 - 0.5;
        let tip = if top > base + 1.0e-4 { tip * 0.5 } else { tip * 0.06 };
        letter.rotation = Quat::from_rotation_y(turn) * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2 + tip);
        letter.position = Vec3::new(position.x, top + LETTER_THICK * 0.5, position.z);
        letter.velocity = Vec3::ZERO;
        letter.spin = 0.0;
        letter.resting = true;
        letter.floor = base;
        self.set_pile(cell, Pile { base, top: top + LETTER_THICK });
    }

    /// Measures every heap again from the letters at rest in it.
    fn remeasure(&mut self) {
        self.piles.clear();
        let resting: Vec<(Vec3, f32)> = self.letters.iter().filter(|l| l.resting).map(|l| (l.position, l.floor)).collect();
        for (p, floor) in resting {
            self.set_pile(cell_of(p), Pile { base: floor, top: p.y + LETTER_THICK * 0.5 });
        }
        self.piles_stale = false;
    }
}

impl Debris {
    /// Makes a broken thing whole: each of its letters sets out for home, the nearest to
    /// home first, along an arc back up the way it fell, its spin unwinding.
    pub fn mend(&mut self, owner: u64, from: Vec3) -> usize {
        let mut n = 0;
        for l in self.letters.iter_mut().filter(|l| l.owner == owner && l.returning.is_none()) {
            let away = l.position.distance(l.home);
            // Further letters set out first, so that neighbours arrive together; and the
            // thing knits outward from where the word was written.
            let wait = -(0.6 - (away * 0.12).min(0.6)) - l.home.distance(from) * MEND_SPREAD;
            l.returning = Some((l.position, wait));
            if l.resting {
                l.resting = false;
                self.awake += 1;
            }
            n += 1;
        }
        if n > 0 {
            self.piles_stale = true;
        }
        n
    }

    /// The pieces of a thing being mended whose letters are all home.
    pub fn pieces_home(&self, owner: u64) -> Vec<u32> {
        let mut out: Vec<(u32, bool)> = Vec::new();
        for l in self.letters.iter().filter(|l| l.owner == owner) {
            let home = l.returning.is_some_and(|(_, t)| t >= MEND_SECONDS);
            match out.iter_mut().find(|(p, _)| *p == l.piece) {
                Some(e) => e.1 &= home,
                None => out.push((l.piece, home)),
            }
        }
        out.into_iter().filter(|e| e.1).map(|e| e.0).collect()
    }

    /// The things whose letters have all come home since last asked. Their letters are
    /// gone: the thing stands whole again.
    pub fn take_mended(&mut self) -> Vec<u64> {
        let mut owners: Vec<u64> = self.letters.iter().filter(|l| l.returning.is_some()).map(|l| l.owner).collect();
        owners.sort_unstable();
        owners.dedup();
        let done: Vec<u64> = owners.into_iter().filter(|o| self.letters.iter().filter(|l| l.owner == *o).all(|l| l.returning.is_some_and(|(_, t)| t >= MEND_SECONDS))).collect();
        if !done.is_empty() {
            let before = self.letters.len();
            self.letters.retain(|l| !done.contains(&l.owner));
            self.awake = self.awake.saturating_sub(before - self.letters.len());
            self.piles_stale = true;
        }
        done
    }

    /// Settles every letter in flight, as if they had fallen long ago.
    pub fn settle(&mut self, dt: f32) {
        for _ in 0..1200 {
            if self.awake == 0 {
                break;
            }
            self.step(dt);
        }
    }

    /// Gives the letters ground to land on: the terrain's height.
    pub fn set_ground(&mut self, ground: std::sync::Arc<dyn crate::Ground>) {
        self.ground = Some(ground);
    }

    fn ground_at(&self, x: f32, z: f32) -> f32 {
        self.ground.as_ref().and_then(|g| g.height(x, z)).unwrap_or(0.0)
    }

    pub fn set_solids(&mut self, solids: Vec<Solid>) {
        self.solids = solids;
    }

    pub fn awake(&self) -> usize {
        self.awake
    }

    /// Adds a letter, making room among the ones at rest if there is none.
    pub fn push(&mut self, letter: Letter) {
        if self.letters.len() >= MAX_LETTERS {
            match self.letters.iter().position(|l| l.resting) {
                Some(oldest) => {
                    self.letters.swap_remove(oldest);
                    self.piles_stale = true;
                }
                None => return,
            }
        }
        self.awake += 1;
        self.letters.push(letter);
    }

    /// One fixed step.
    pub fn step(&mut self, dt: f32) {
        if self.awake == 0 {
            return;
        }
        if self.piles_stale {
            self.remeasure();
        }
        let mut awake = 0;
        for i in 0..self.letters.len() {
            if self.letters[i].resting {
                continue;
            }
            // On its way home: along an arc, easing into place, its spin unwinding.
            if let Some((from, t)) = self.letters[i].returning {
                let l = &mut self.letters[i];
                let t = t + dt;
                l.returning = Some((from, t));
                let p = (t / MEND_SECONDS).clamp(0.0, 1.0);
                let ease = p * p * (3.0 - 2.0 * p);
                let rise = from.distance(l.home) * 0.35 + 0.25;
                let mid = (from + l.home) * 0.5 + Vec3::Y * rise;
                let a = from.lerp(mid, ease);
                let b = mid.lerp(l.home, ease);
                l.position = a.lerp(b, ease);
                l.rotation = l.rotation.slerp(Quat::IDENTITY, (dt * 4.0).min(1.0));
                l.velocity = Vec3::ZERO;
                awake += 1;
                continue;
            }
            let solids = &self.solids;
            let letter = &mut self.letters[i];
            letter.velocity.y += GRAVITY * dt;
            // A little drag, so that a burst does not carry letters across the valley.
            letter.velocity *= 1.0 - 0.35 * dt;
            letter.position += letter.velocity * dt;
            letter.rotation = (Quat::from_axis_angle(letter.spin_axis, letter.spin * dt) * letter.rotation).normalize();

            // The surface it is lying on, if it is lying on one this step.
            let mut on: Option<f32> = None;
            // The ground.
            let ground = self.ground.as_ref().and_then(|g| g.height(letter.position.x, letter.position.z)).unwrap_or(0.0);
            if letter.position.y < ground + LETTER_HALF {
                letter.position.y = ground + LETTER_HALF;
                strike(letter, Vec3::Y);
                on = Some(ground);
            }
            for solid in solids {
                let (min, max) = (solid.min - LETTER_HALF, solid.max + LETTER_HALF);
                let p = letter.position;
                if p.x <= min.x || p.x >= max.x || p.y <= min.y || p.y >= max.y || p.z <= min.z || p.z >= max.z {
                    continue;
                }
                // Inside: put it back out through the nearest face.
                let exits = [
                    (p.x - min.x, Vec3::NEG_X),
                    (max.x - p.x, Vec3::X),
                    (p.y - min.y, Vec3::NEG_Y),
                    (max.y - p.y, Vec3::Y),
                    (p.z - min.z, Vec3::NEG_Z),
                    (max.z - p.z, Vec3::Z),
                ];
                // Out through the nearest face that leads into the open: never under the
                // ground, and never into the thing next door, or a letter in the corner
                // where a step meets a wall would be passed between them for ever.
                let open = |to: Vec3| {
                    to.y >= LETTER_HALF - 1.0e-4
                        && !solids.iter().any(|other| {
                            other != solid
                                && to.cmpgt(other.min - LETTER_HALF + 1.0e-4).all()
                                && to.cmplt(other.max + LETTER_HALF - 1.0e-4).all()
                        })
                };
                let (depth, normal) = exits
                    .into_iter()
                    .filter(|&(depth, n)| open(p + n * depth))
                    .min_by(|a, b| a.0.total_cmp(&b.0))
                    .unwrap_or((max.y - p.y, Vec3::Y));
                letter.position += normal * depth;
                strike(letter, normal);
                if normal == Vec3::Y {
                    on = Some(solid.max.y);
                }
            }

            // The letters already at rest: a letter that comes down onto a heap, or into
            // its side, ends on its top.
            let p = self.letters[i].position;
            if let Some(pile) = self.pile_under(cell_of(p), p.y - LETTER_HALF) {
                if p.y < pile.top + LETTER_HALF {
                    let letter = &mut self.letters[i];
                    letter.position.y = pile.top + LETTER_HALF;
                    strike(letter, Vec3::Y);
                    on = Some(pile.base);
                }
            }

            let letter = &self.letters[i];
            match on {
                Some(base) if letter.velocity.length_squared() < REST_SPEED * REST_SPEED => {
                    let top = letter.position.y - LETTER_HALF;
                    self.lay(i, base, top);
                }
                _ => awake += 1,
            }
        }
        self.awake = awake;
    }
}

/// A letter strikes a surface with this outward normal.
fn strike(letter: &mut Letter, normal: Vec3) {
    let into = letter.velocity.dot(normal);
    if into < 0.0 {
        let along = letter.velocity - normal * into;
        letter.velocity = along * SLIDE - normal * into * BOUNCE;
        // Striking something knocks most of the spin out of it.
        letter.spin *= 0.6;
    }
}

/// Fills a box with letters, a cell apart, and throws them outwards from `burst`.
/// `strength` is how hard: 1 is a blow that throws them across the yard, and near 0
/// the thing simply falls in on itself.
/// `to_world` places the box's own space in the world. `random` gives the rules'
/// deterministic random numbers. `keep` is the share of cells that hold a letter, at
/// most seven in ten. Returns how many letters were made.
pub fn shatter(
    debris: &mut Debris,
    min: Vec3,
    max: Vec3,
    to_world: glam::Affine3A,
    material: u32,
    burst: Vec3,
    strength: f32,
    owner: u64,
    piece: u32,
    keep: f32,
    random: &mut dyn FnMut() -> u64,
) -> usize {
    let spacing = LETTER_HALF * 2.0 * 1.25;
    let count = |extent: f32| ((extent / spacing).floor() as usize).max(1);
    let (nx, ny, nz) = (count(max.x - min.x), count(max.y - min.y), count(max.z - min.z));
    let cells = nx * ny * nz;
    // Seven cells in ten hold a letter: a thing is not solid print. A big thing keeps
    // fewer, scattered through it, so that it does not bury the world in letters.
    let wanted = (cells as f32 * keep.clamp(0.0, 0.7)).round() as usize;
    let sparse = wanted * 10 < cells * 7;
    let mut made = 0;
    let mut cell = 0usize;
    while if sparse { made < wanted } else { cell < cells } {
        let r = random();
        let local = if sparse {
            // On the box's faces, by their area: its print is what is seen of it.
            let q = random();
            let unit = |shift: u32| ((q >> shift) & 0xf_ffff) as f32 / 1_048_575.0;
            let size = max - min;
            let areas = Vec3::new(size.y * size.z, size.x * size.z, size.x * size.y);
            let pick = unit(40) * (areas.x + areas.y + areas.z);
            let axis = if pick < areas.x {
                0
            } else if pick < areas.x + areas.y {
                1
            } else {
                2
            };
            let mut p = Vec3::ZERO;
            p[(axis + 1) % 3] = unit(0);
            p[(axis + 2) % 3] = unit(20);
            p[axis] = if q >> 63 == 0 { 0.0 } else { 1.0 };
            let inset = Vec3::splat(LETTER_HALF).min(size * 0.5);
            (min + size * p).clamp(min + inset, max - inset)
        } else {
            let (iy, rest) = (cell / (nx * nz), cell % (nx * nz));
            let (ix, iz) = (rest / nz, rest % nz);
            cell += 1;
            if r % 10 >= 7 {
                continue;
            }
            Vec3::new(
                min.x + (ix as f32 + 0.5) * (max.x - min.x) / nx as f32,
                min.y + (iy as f32 + 0.5) * (max.y - min.y) / ny as f32,
                min.z + (iz as f32 + 0.5) * (max.z - min.z) / nz as f32,
            )
        };
        {
            {
                let unit = |shift: u32| ((r >> shift) & 0x3ff) as f32 / 1023.0;
                let position = to_world.transform_point3(local);
                // Outwards from the blow, harder the nearer it was, and never quite the same.
                let away = position - burst;
                let distance = away.length().max(0.05);
                let push = away / distance * (2.6 / (0.4 + distance)) * (0.5 + unit(10));
                let scatter = Vec3::new(unit(20) - 0.5, unit(30) * 0.8, unit(40) - 0.5) * 1.2;
                let axis = Vec3::new(unit(14) - 0.5, unit(24) - 0.5, unit(34) - 0.5).normalize_or(Vec3::Y);
                debris.push(Letter {
                    position,
                    velocity: (push + scatter) * strength,
                    rotation: Quat::IDENTITY,
                    spin: 4.0 + unit(44) * 14.0,
                    spin_axis: axis,
                    material,
                    seed: (r >> 32) as u32,
                    resting: false,
                    home: position,
                    owner,
                    piece,
                    returning: None,
                    floor: 0.0,
                });
                made += 1;
            }
        }
    }
    made
}

#[cfg(test)]
mod tests {
    use super::*;

    fn random_from(mut state: u64) -> impl FnMut() -> u64 {
        move || {
            state = state.wrapping_add(0x9e37_79b9_7f4a_7c15);
            let mut z = state;
            z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
            z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
            z ^ (z >> 31)
        }
    }

    fn broken_door() -> Debris {
        let mut debris = Debris::default();
        // A low wall for letters to strike and land on.
        debris.set_solids(vec![Solid { min: Vec3::new(-2.0, 0.0, 1.0), max: Vec3::new(2.0, 0.9, 1.4) }]);
        let mut random = random_from(11);
        let made = shatter(
            &mut debris,
            Vec3::new(0.0, 0.02, -0.04),
            Vec3::new(1.3, 2.15, 0.04),
            glam::Affine3A::from_translation(Vec3::new(-0.65, 0.0, 0.0)),
            1,
            Vec3::new(0.0, 1.0, -0.4),
            1.0,
            7,
            0,
            0.7,
            &mut random,
        );
        assert!(made > 20_000, "a door is tens of thousands of letters: {made}");
        debris
    }

    #[test]
    fn letters_fall_and_all_come_to_rest_on_something() {
        let mut debris = broken_door();
        for _ in 0..60 * 12 {
            debris.step(1.0 / 60.0);
        }
        assert_eq!(debris.awake(), 0, "every letter has come to rest within twelve seconds");
        let wall = debris.solids[0];
        for letter in &debris.letters {
            assert!(letter.resting);
            assert!(letter.position.y > 0.0, "none has fallen through the ground");
            let p = letter.position;
            let inside = p.x > wall.min.x && p.x < wall.max.x && p.y > wall.min.y && p.y < wall.max.y && p.z > wall.min.z && p.z < wall.max.z;
            assert!(!inside, "none is left inside the wall");
        }
        // They have spread across the floor, not stayed in a heap where the door stood.
        let far = debris.letters.iter().filter(|l| l.position.z.abs() > 0.5).count();
        assert!(far > debris.letters.len() / 10);
        // And some came to rest on top of the wall.
        assert!(debris.letters.iter().any(|l| l.floor == 0.9 && l.position.y > 0.9 && l.position.y < 1.0));
    }

    #[test]
    fn letters_poured_in_one_place_make_a_heap_no_steeper_than_letters_stand() {
        let mut debris = Debris::default();
        let mut random = random_from(5);
        for i in 0..6000u32 {
            let r = random();
            let unit = |shift: u32| ((r >> shift) & 0x3ff) as f32 / 1023.0 - 0.5;
            debris.push(Letter {
                position: Vec3::new(unit(0) * 0.02, 0.4 + i as f32 * 0.0004, unit(10) * 0.02),
                velocity: Vec3::ZERO,
                rotation: Quat::IDENTITY,
                spin: 3.0,
                spin_axis: Vec3::X,
                material: 0,
                seed: (r >> 32) as u32,
                resting: false,
                home: Vec3::ZERO,
                owner: 0,
                piece: 0,
                returning: None,
                floor: 0.0,
            });
        }
        for _ in 0..60 * 10 {
            debris.step(1.0 / 60.0);
        }
        assert_eq!(debris.awake(), 0);
        let highest = debris.letters.iter().map(|l| l.position.y).fold(0.0, f32::max);
        assert!(highest > 0.03, "the letters stand in a heap, not a single layer: {highest}");
        // It is a cone, not a column: it has spread wider than it is high.
        let widest = debris.letters.iter().map(|l| l.position.x.abs().max(l.position.z.abs())).fold(0.0, f32::max);
        assert!(widest > highest, "wider than high: {widest} against {highest}");
        // Nowhere is one cell of the heap higher than its neighbour by more than letters stand.
        for (&(x, z), piles) in &debris.piles {
            let top = piles[0].top;
            for (dx, dz) in [(1, 0), (0, 1), (-1, 0), (0, -1)] {
                let next = debris.piles.get(&(x + dx, z + dz)).map_or(0.0, |p| p[0].top);
                assert!(top - next <= REPOSE + LETTER_THICK * 2.0 + 1.0e-4, "a cliff in the heap: {top} beside {next}");
            }
        }
        // And every letter lies on the heap's surface, where the map says it is.
        for l in &debris.letters {
            let top = debris.piles[&cell_of(l.position)][0].top;
            assert!(l.position.y <= top + 1.0e-4);
        }
    }

    #[test]
    fn the_same_break_falls_the_same_way_twice() {
        let run = || {
            let mut debris = broken_door();
            for _ in 0..300 {
                debris.step(1.0 / 60.0);
            }
            debris.letters.iter().map(|l| (l.position.to_array().map(f32::to_bits), l.resting)).collect::<Vec<_>>()
        };
        assert!(run() == run());
    }

    #[test]
    fn a_broken_thing_is_made_whole_along_the_way_its_letters_fell() {
        let mut debris = broken_door();
        for _ in 0..60 * 6 {
            debris.step(1.0 / 60.0);
        }
        let made = debris.letters.len();
        assert_eq!(debris.mend(7, Vec3::ZERO), made);
        let mut seen_high = false;
        for step in 0..60 * 3 {
            debris.step(1.0 / 60.0);
            // On their way, letters rise above where they lay.
            seen_high |= debris.letters.iter().any(|l| l.position.y > l.home.y + 0.2);
            let done = debris.take_mended();
            if !done.is_empty() {
                assert_eq!(done, [7]);
                assert!(seen_high, "the letters arc home");
                assert!(debris.letters.is_empty());
                assert!(step as f32 / 60.0 <= MEND_SECONDS + 0.7);
                return;
            }
        }
        panic!("the door was never made whole");
    }

    #[test]
    fn the_oldest_letters_at_rest_make_room() {
        let mut debris = Debris::default();
        let letter = Letter {
            position: Vec3::Y,
            velocity: Vec3::ZERO,
            rotation: Quat::IDENTITY,
            spin: 0.0,
            spin_axis: Vec3::Y,
            material: 0,
            seed: 0,
            resting: true,
            home: Vec3::Y,
            owner: 0,
            piece: 0,
            returning: None,
            floor: 0.0,
        };
        for _ in 0..MAX_LETTERS + 10 {
            debris.push(letter);
        }
        assert_eq!(debris.letters.len(), MAX_LETTERS);
    }
}
