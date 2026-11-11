//! Loose letters. When a thing breaks, it comes apart into what it was made of: tens
//! of thousands of characters, each a small body that falls, strikes the world,
//! bounces, slides, and comes to rest on the floor.
//!
//! Each letter is simulated on its own, against the ground and the scene's boxes.
//! Letters do not strike one another: there are too many, and a spill of them reads
//! right without it. The simulation is deterministic, like the rest of the rules, so
//! that a broken thing can later be made whole along the paths its letters fell.

use glam::{Quat, Vec3};

/// The most loose letters alive at once. Older ones at rest are swept first.
pub const MAX_LETTERS: usize = 60_000;
/// Half the side of a letter, in metres. Letters are the size of the 1.28 cm cells.
pub const LETTER_HALF: f32 = 0.0064;

const GRAVITY: f32 = -9.81;
/// How much of its speed a letter keeps along the normal when it strikes something.
const BOUNCE: f32 = 0.32;
/// How much of its speed along a surface it keeps on striking it.
const SLIDE: f32 = 0.62;
/// Below this speed, on something it can lie on, a letter comes to rest.
const REST_SPEED: f32 = 0.22;

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
}

#[derive(Clone, Debug, Default)]
pub struct Debris {
    pub letters: Vec<Letter>,
    solids: Vec<Solid>,
    awake: usize,
}

impl Debris {
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
        let solids = &self.solids;
        let mut awake = 0;
        for letter in self.letters.iter_mut().filter(|l| !l.resting) {
            letter.velocity.y += GRAVITY * dt;
            // A little drag, so that a burst does not carry letters across the valley.
            letter.velocity *= 1.0 - 0.35 * dt;
            letter.position += letter.velocity * dt;
            letter.rotation = (Quat::from_axis_angle(letter.spin_axis, letter.spin * dt) * letter.rotation).normalize();

            let mut on_top = false;
            // The ground.
            if letter.position.y < LETTER_HALF {
                letter.position.y = LETTER_HALF;
                strike(letter, Vec3::Y);
                on_top = true;
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
                    on_top = true;
                }
            }

            if on_top && letter.velocity.length_squared() < REST_SPEED * REST_SPEED {
                // It lies down flat, face up, turned whichever way it happened to land.
                let turn = (letter.seed % 6283) as f32 / 1000.0;
                letter.rotation = Quat::from_rotation_y(turn) * Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2);
                // Flat on what it landed on, a hair above it, each at its own height so
                // that letters lying across one another keep a steady order.
                letter.position.y += -LETTER_HALF + 0.0006 + (letter.seed % 8) as f32 * 0.0002;
                letter.velocity = Vec3::ZERO;
                letter.spin = 0.0;
                letter.resting = true;
            } else {
                awake += 1;
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
/// `to_world` places the box's own space in the world. `random` gives the rules'
/// deterministic random numbers. Returns how many letters were made.
pub fn shatter(
    debris: &mut Debris,
    min: Vec3,
    max: Vec3,
    to_world: glam::Affine3A,
    material: u32,
    burst: Vec3,
    random: &mut dyn FnMut() -> u64,
) -> usize {
    let spacing = LETTER_HALF * 2.0 * 1.25;
    let count = |extent: f32| ((extent / spacing).floor() as usize).max(1);
    let (nx, ny, nz) = (count(max.x - min.x), count(max.y - min.y), count(max.z - min.z));
    let mut made = 0;
    for iy in 0..ny {
        for ix in 0..nx {
            for iz in 0..nz {
                let r = random();
                // Seven cells in ten hold a letter: a thing is not solid print.
                if r % 10 >= 7 {
                    continue;
                }
                let unit = |shift: u32| ((r >> shift) & 0x3ff) as f32 / 1023.0;
                let local = Vec3::new(
                    min.x + (ix as f32 + 0.5) * (max.x - min.x) / nx as f32,
                    min.y + (iy as f32 + 0.5) * (max.y - min.y) / ny as f32,
                    min.z + (iz as f32 + 0.5) * (max.z - min.z) / nz as f32,
                );
                let position = to_world.transform_point3(local);
                // Outwards from the blow, harder the nearer it was, and never quite the same.
                let away = position - burst;
                let distance = away.length().max(0.05);
                let push = away / distance * (2.6 / (0.4 + distance)) * (0.5 + unit(10));
                let scatter = Vec3::new(unit(20) - 0.5, unit(30) * 0.8, unit(40) - 0.5) * 1.2;
                let axis = Vec3::new(unit(14) - 0.5, unit(24) - 0.5, unit(34) - 0.5).normalize_or(Vec3::Y);
                debris.push(Letter {
                    position,
                    velocity: push + scatter,
                    rotation: Quat::IDENTITY,
                    spin: 4.0 + unit(44) * 14.0,
                    spin_axis: axis,
                    material,
                    seed: (r >> 32) as u32,
                    resting: false,
                    home: position,
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
        assert!(debris.letters.iter().any(|l| (l.position.y - 0.9).abs() < 3e-3));
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
        };
        for _ in 0..MAX_LETTERS + 10 {
            debris.push(letter);
        }
        assert_eq!(debris.letters.len(), MAX_LETTERS);
    }
}
