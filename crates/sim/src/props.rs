//! Loose props: crates, barrels, bottles, and anything else a scene marks with a mass.
//! Each is a rigid body in its own `rapier3d` world, standing on the same still things
//! the bodies of players stand on. Players pick them up, carry them before them, and
//! throw them; a breakable one jolted hard enough falls to its letters.
//!
//! Bodies at rest sleep and cost nothing. The world is stepped at the rules' fixed step,
//! with the same inputs in the same order, so it is deterministic on one machine.

use std::sync::Arc;

use glam::{Affine3A, Quat, Vec3};
use rapier3d::math::{Pose, Rotation, Vector};
use rapier3d::prelude::{
    ColliderBuilder, PhysicsWorld, QueryFilter, Ray, RigidBodyBuilder, RigidBodyHandle,
};

use crate::character::{HeightPatch, Trunk};
use crate::debris::Solid;

/// The heaviest thing a player can pick up, in kilograms.
pub const CARRY_MASS: f32 = 40.0;
/// How far off a thing can be picked up, from the eye, in metres.
pub const REACH: f32 = 2.6;
/// Where a carried thing is held: this far before the eye.
pub const HOLD_AT: f32 = 1.3;
/// A carried thing more than this far from where it is held is let go: it caught on
/// something.
pub const LOSE_HOLD: f32 = 1.4;
/// How fast a thing of five kilograms leaves the hands, in metres a second. Lighter
/// things go faster and heavier ones slower.
pub const THROW_SPEED: f32 = 11.0;
/// A change of speed in one step, in metres a second, that breaks a breakable thing.
pub const BREAK_JOLT: f32 = 7.5;

/// A thing that is not fixed in place, as the scene places it.
#[derive(Clone, Debug, PartialEq)]
pub struct LooseThing {
    pub id: u64,
    pub to_world: Affine3A,
    /// Its boxes in its own space.
    pub boxes: Vec<(Vec3, Vec3)>,
    /// In kilograms.
    pub mass: f32,
}

/// The still things props stand on and strike.
#[derive(Clone, Debug, Default)]
pub struct Still {
    pub patches: Vec<HeightPatch>,
    pub solids: Vec<Solid>,
    pub trunks: Vec<Trunk>,
}

struct Body {
    id: u64,
    handle: RigidBodyHandle,
    mass: f32,
    velocity: Vec3,
}

pub struct Props {
    world: PhysicsWorld,
    bodies: Vec<Body>,
    things: Vec<LooseThing>,
    still: Arc<Still>,
    dt: f32,
}

impl std::fmt::Debug for Props {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Props({} bodies)", self.bodies.len())
    }
}

/// A copy stands where the original stands and moves as it moves. Contacts are found
/// afresh, so the copy's next steps may differ from the original's in the last bits.
impl Clone for Props {
    fn clone(&self) -> Self {
        let mut things = self.things.clone();
        things.retain(|t| self.bodies.iter().any(|b| b.id == t.id));
        for t in &mut things {
            if let Some(p) = self.pose(t.id) {
                t.to_world = p;
            }
        }
        let mut copy = Self::new(self.still.clone(), things, self.dt);
        for b in &self.bodies {
            let body = &self.world.bodies[b.handle];
            let (v, w) = (body.linvel(), body.angvel());
            if let Some(c) = copy.bodies.iter_mut().find(|c| c.id == b.id) {
                c.velocity = b.velocity;
                let target = &mut copy.world.bodies[c.handle];
                target.set_linvel(v, false);
                target.set_angvel(w, false);
            }
        }
        copy
    }
}

fn v(p: Vec3) -> Vector {
    Vector::new(p.x, p.y, p.z)
}

fn g(p: Vector) -> Vec3 {
    Vec3::new(p.x, p.y, p.z)
}

fn pose_of(t: &Affine3A) -> Pose {
    let (_, r, p) = t.to_scale_rotation_translation();
    Pose::from_parts(v(p), Rotation::from_xyzw(r.x, r.y, r.z, r.w))
}

fn affine_of(p: &Pose) -> Affine3A {
    let r = p.rotation;
    Affine3A::from_rotation_translation(Quat::from_xyzw(r.x, r.y, r.z, r.w), g(p.translation))
}

impl Props {
    /// `dt` is the rules' fixed step.
    pub fn new(still: Arc<Still>, things: Vec<LooseThing>, dt: f32) -> Self {
        let mut world = PhysicsWorld::new();
        world.integration_parameters.dt = dt;
        crate::character::add_still(&mut world, &still.patches, &still.solids, &still.trunks);
        let mut bodies = Vec::with_capacity(things.len());
        for t in &things {
            let volume: f32 = t
                .boxes
                .iter()
                .map(|(min, max)| (*max - *min).max(Vec3::splat(0.01)))
                .map(|s| s.x * s.y * s.z)
                .sum();
            let density = t.mass / volume.max(1e-4);
            let handle = world.insert_body(
                RigidBodyBuilder::dynamic()
                    .pose(pose_of(&t.to_world))
                    .linear_damping(0.05)
                    .angular_damping(0.3)
                    .ccd_enabled(true)
                    .user_data(t.id as u128),
            );
            for (min, max) in &t.boxes {
                let half = ((*max - *min) * 0.5).max(Vec3::splat(0.005));
                let centre = (*min + *max) * 0.5;
                world.insert_collider(
                    ColliderBuilder::cuboid(half.x, half.y, half.z)
                        .translation(v(centre))
                        .density(density)
                        .friction(0.7)
                        .restitution(0.15),
                    Some(handle),
                );
            }
            bodies.push(Body {
                id: t.id,
                handle,
                mass: t.mass,
                velocity: Vec3::ZERO,
            });
        }
        Self {
            world,
            bodies,
            things,
            still,
            dt,
        }
    }

    /// The step's length: shorter while time is slowed.
    pub fn set_dt(&mut self, dt: f32) {
        self.dt = dt;
        self.world.integration_parameters.dt = dt;
    }

    pub fn is_empty(&self) -> bool {
        self.bodies.is_empty()
    }

    /// One fixed step. Returns the things jolted hard enough to break, with where they
    /// were struck, leaving out `spared` (things in hand).
    pub fn step(&mut self, spared: &[u64]) -> Vec<(u64, Vec3)> {
        self.world.step();
        let mut jolted = Vec::new();
        for b in &mut self.bodies {
            let body = &self.world.bodies[b.handle];
            let now = g(body.linvel());
            if !spared.contains(&b.id) && (now - b.velocity).length() > BREAK_JOLT {
                jolted.push((
                    b.id,
                    g(body.translation()) - (now - b.velocity).normalize_or_zero() * 0.2,
                ));
            }
            b.velocity = now;
        }
        jolted
    }

    /// Where a thing stands now, as its own space placed in the world.
    pub fn pose(&self, id: u64) -> Option<Affine3A> {
        self.bodies
            .iter()
            .find(|b| b.id == id)
            .map(|b| affine_of(self.world.bodies[b.handle].position()))
    }

    /// Every loose thing's place now.
    pub fn poses(&self) -> Vec<(u64, Affine3A)> {
        self.bodies
            .iter()
            .map(|b| (b.id, affine_of(self.world.bodies[b.handle].position())))
            .collect()
    }

    /// The loose thing a ray from the eye meets first within reach, if one can be picked
    /// up.
    pub fn pick(&self, eye: Vec3, gaze: Vec3) -> Option<u64> {
        let ray = Ray::new(v(eye), v(gaze));
        let (collider, _) = self
            .world
            .cast_ray(&ray, REACH, true, QueryFilter::only_dynamic())?;
        let parent = self.world.colliders[collider].parent()?;
        self.bodies
            .iter()
            .find(|b| b.handle == parent && b.mass <= CARRY_MASS)
            .map(|b| b.id)
    }

    /// Draws a carried thing toward where it is held, and steadies it. Returns false if
    /// it has caught on something and is let go.
    pub fn carry(&mut self, id: u64, at: Vec3) -> bool {
        let Some(b) = self.bodies.iter().find(|b| b.id == id) else {
            return false;
        };
        let body = &mut self.world.bodies[b.handle];
        let centre = g(body.center_of_mass());
        let off = at - centre;
        if off.length() > LOSE_HOLD {
            return false;
        }
        let want = (off / self.dt * 0.35).clamp_length_max(14.0);
        body.set_linvel(v(want), true);
        let w = body.angvel();
        body.set_angvel(w * 0.8, true);
        true
    }

    /// Sends a carried thing off along `dir`, as fast as its weight allows.
    pub fn throw(&mut self, id: u64, dir: Vec3) {
        let Some(b) = self.bodies.iter().find(|b| b.id == id) else {
            return;
        };
        let speed = THROW_SPEED * (5.0 / b.mass.max(0.1)).sqrt().clamp(0.3, 1.4);
        let body = &mut self.world.bodies[b.handle];
        body.set_linvel(v(dir.normalize_or_zero() * speed), true);
    }

    /// The thing is gone from the world: it broke.
    pub fn remove(&mut self, id: u64) {
        if let Some(at) = self.bodies.iter().position(|b| b.id == id) {
            let b = self.bodies.swap_remove(at);
            self.world.remove_body_with_colliders(b.handle, true);
        }
    }

    /// Lets every body fall and come to rest, as though it had stood there a while.
    pub fn settle(&mut self) {
        for _ in 0..900 {
            self.step(&[]);
            if self.awake() == 0 {
                break;
            }
        }
    }

    /// Bodies awake now.
    pub fn awake(&self) -> usize {
        self.bodies
            .iter()
            .filter(|b| !self.world.bodies[b.handle].is_sleeping())
            .count()
    }
}
