//! The first-person body: walking, running, jumping, crouching, mantling up ledges,
//! swimming, and the sidestep, against the world's still shapes on `rapier3d`.
//!
//! The still world (terrain as heightfields, the scene's solid boxes, tree trunks) is
//! built once into a rapier world that never changes, shared between copies of the
//! `World`. A player is a capsule moved by rapier's character controller each fixed
//! step; everything else about the body is its own small state here, so that stepping
//! stays deterministic.

use glam::{Vec2, Vec3};
use rapier3d::control::{CharacterAutostep, CharacterLength, KinematicCharacterController};
use rapier3d::math::{Pose, Vector};
use rapier3d::prelude::{
    Array2, ColliderBuilder, HeightField, HeightFieldCellStatus, PhysicsWorld, QueryFilter, Ray,
    SharedShape,
};

use crate::input::{PlayerInput, buttons};

// The body's measures, in metres.
pub const RADIUS: f32 = 0.32;
pub const STANDING_HEIGHT: f32 = 1.78;
pub const CROUCHED_HEIGHT: f32 = 1.15;
pub const EYE_STANDING: f32 = 1.68;
pub const EYE_CROUCHED: f32 = 1.05;

// How it moves, in metres and seconds.
pub const JOG: f32 = 5.0;
pub const SPRINT: f32 = 7.5;
pub const CROUCH_SPEED: f32 = 2.4;
pub const SWIM: f32 = 2.6;
pub const SWIM_SPRINT: f32 = 3.8;
const GROUND_ACCEL: f32 = 45.0;
const AIR_ACCEL: f32 = 9.0;
const WATER_ACCEL: f32 = 8.0;
pub const GRAVITY: f32 = 20.0;
/// Rises about a metre.
pub const JUMP_SPEED: f32 = 6.4;
const TERMINAL: f32 = 40.0;
/// A jump pressed this long before landing still happens, and one pressed this long after
/// walking off an edge too.
const JUMP_GRACE: f32 = 0.12;

/// The sidestep: about two metres, quickly, and attacks pass through its first moments.
pub const SIDESTEP_DISTANCE: f32 = 2.0;
pub const SIDESTEP_TIME: f32 = 0.22;
pub const SIDESTEP_UNTOUCHABLE: f32 = 0.12;
const SIDESTEP_REST: f32 = 0.18;

/// Ledges this high above the feet can be mantled, at most.
pub const MANTLE_HEIGHT: f32 = 1.9;
const MANTLE_MIN: f32 = 0.45;
const MANTLE_TIME: f32 = 0.42;

/// Deeper than this, the body swims: about chest deep.
pub const SWIM_DEPTH: f32 = 1.25;
/// While swimming, the eye rides this far above the water.
const SWIM_EYE_ABOVE: f32 = 0.18;

const MAX_SLOPE: f32 = 50.0 * std::f32::consts::PI / 180.0;

/// A square of terrain heights for the physics: `samples` by `samples`, x-major, with
/// `spacing` metres between them, starting at `origin` (the smallest x and z).
#[derive(Clone, Debug)]
pub struct HeightPatch {
    pub origin: Vec2,
    pub spacing: f32,
    pub samples: usize,
    pub heights: Vec<f32>,
}

/// An upright trunk: where its foot is, its radius, and its height.
#[derive(Clone, Copy, Debug)]
pub struct Trunk {
    pub foot: Vec3,
    pub radius: f32,
    pub height: f32,
}

/// The world's still shapes, built once.
pub struct Statics {
    world: PhysicsWorld,
    /// The colliders of things that can break, by the thing's permanent ID.
    things: Vec<(u64, Vec<rapier3d::prelude::ColliderHandle>)>,
}

impl std::fmt::Debug for Statics {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Statics({} colliders)", self.world.colliders.len())
    }
}

fn v(p: Vec3) -> Vector {
    Vector::new(p.x, p.y, p.z)
}

fn g(p: Vector) -> Vec3 {
    Vec3::new(p.x, p.y, p.z)
}

/// Adds the still things to a physics world: the ground's patches, boxes, and trunks.
pub(crate) fn add_still(
    world: &mut PhysicsWorld,
    patches: &[HeightPatch],
    solids: &[crate::debris::Solid],
    trunks: &[Trunk],
) {
    for p in patches {
        let n = p.samples;
        // Rapier's heights run down the rows along z and across the columns along x,
        // column by column; ours run along x, row by row.
        let mut data = vec![0.0; n * n];
        for z in 0..n {
            for x in 0..n {
                data[z + x * n] = p.heights[x + z * n];
            }
        }
        let side = p.spacing * (n - 1) as f32;
        let centre = Vec3::new(p.origin.x + side * 0.5, 0.0, p.origin.y + side * 0.5);
        // Each cell is split from its least corner to its greatest, as the terrain is
        // drawn and as `TerrainTile::height_at` reads it.
        let mut field = HeightField::new(Array2::new(n, n, data), Vector::new(side, 1.0, side));
        for i in 0..n - 1 {
            for j in 0..n - 1 {
                field.set_cell_status(i, j, HeightFieldCellStatus::ZIGZAG_SUBDIVISION);
            }
        }
        world.insert_collider(
            ColliderBuilder::new(SharedShape::new(field)).translation(v(centre)),
            None,
        );
    }
    for s in solids {
        let half = (s.max - s.min) * 0.5;
        if half.min_element() <= 0.0 {
            continue;
        }
        world.insert_collider(
            ColliderBuilder::cuboid(half.x, half.y, half.z).translation(v(s.min + half)),
            None,
        );
    }
    for t in trunks {
        let half = (t.height * 0.5 - t.radius).max(0.05);
        world.insert_collider(
            ColliderBuilder::capsule_y(half, t.radius)
                .translation(v(t.foot + Vec3::Y * (t.height * 0.5))),
            None,
        );
    }
}

impl Statics {
    pub fn new(patches: &[HeightPatch], solids: &[crate::debris::Solid], trunks: &[Trunk]) -> Self {
        let mut world = PhysicsWorld::new();
        add_still(&mut world, patches, solids, trunks);
        // Once, so that the queries know where everything is.
        world.step();
        Self {
            world,
            things: Vec::new(),
        }
    }

    /// Sets the boxes a breakable thing stands in now: none while it is broken, some
    /// while it is being mended, all once it is whole.
    pub fn set_thing(&mut self, id: u64, solids: &[crate::debris::Solid]) {
        if let Some(at) = self.things.iter().position(|(t, _)| *t == id) {
            let (_, handles) = self.things.swap_remove(at);
            for h in handles {
                self.world.remove_collider(h);
            }
        }
        let handles: Vec<_> = solids
            .iter()
            .filter(|s| (s.max - s.min).min_element() > 0.0)
            .map(|s| {
                let half = (s.max - s.min) * 0.5;
                self.world.insert_collider(
                    ColliderBuilder::cuboid(half.x, half.y, half.z).translation(v(s.min + half)),
                    None,
                )
            })
            .collect();
        if !handles.is_empty() {
            self.things.push((id, handles));
        }
        self.world.step();
    }

    /// The first still thing a ray meets, within `reach`: how far along, and the normal.
    pub fn ray(&self, from: Vec3, dir: Vec3, reach: f32) -> Option<(f32, Vec3)> {
        let ray = Ray::new(v(from), v(dir));
        self.world
            .cast_ray_and_get_normal(&ray, reach, true, QueryFilter::default())
            .map(|(_, hit)| (hit.time_of_impact, g(hit.normal)))
    }

    /// Whether a capsule of the body's radius and this height, standing on `feet`, would
    /// overlap anything.
    pub fn blocked(&self, feet: Vec3, height: f32) -> bool {
        let half = height * 0.5 - RADIUS;
        let shape = SharedShape::capsule_y(half, RADIUS - 0.02);
        let pose = Pose::from_translation(v(feet + Vec3::Y * (height * 0.5 + 0.03)));
        self.world
            .intersect_shape(pose, shape.as_ref(), QueryFilter::default())
            .next()
            .is_some()
    }
}

/// What the world around the body is like where it stands.
pub trait Surroundings {
    /// The height of the ground, if any.
    fn ground(&self, x: f32, z: f32) -> Option<f32>;
    /// The height of the water's surface, if there is water.
    fn water(&self, x: f32, z: f32) -> Option<f32>;
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Sidestep {
    dir: Vec3,
    t: f32,
}

#[derive(Clone, Copy, Debug, PartialEq)]
struct Mantle {
    from: Vec3,
    to: Vec3,
    t: f32,
}

/// One body's state between steps.
#[derive(Clone, Debug, PartialEq)]
pub struct Body {
    pub feet: Vec3,
    pub velocity: Vec3,
    pub grounded: bool,
    pub crouching: bool,
    pub swimming: bool,
    /// The eye's height above the feet now, eased between standing and crouching.
    pub eye: f32,
    since_ground: f32,
    jump_wanted: f32,
    sidestep: Option<Sidestep>,
    rest: f32,
    mantle: Option<Mantle>,
    last_buttons: u32,
}

impl Body {
    pub fn new(feet: Vec3) -> Self {
        Self {
            feet,
            velocity: Vec3::ZERO,
            grounded: true,
            crouching: false,
            swimming: false,
            eye: EYE_STANDING,
            since_ground: 0.0,
            jump_wanted: f32::MAX,
            sidestep: None,
            rest: 0.0,
            mantle: None,
            last_buttons: 0,
        }
    }

    /// Whether attacks pass through it now: the first moments of a sidestep.
    pub fn untouchable(&self) -> bool {
        self.sidestep.is_some_and(|s| s.t < SIDESTEP_UNTOUCHABLE)
    }

    pub fn sidestepping(&self) -> bool {
        self.sidestep.is_some()
    }

    pub fn mantling(&self) -> bool {
        self.mantle.is_some()
    }

    fn height(&self) -> f32 {
        if self.crouching {
            CROUCHED_HEIGHT
        } else {
            STANDING_HEIGHT
        }
    }

    /// The bits of `state_hash` that make a body what it is.
    pub fn hash(&self, h: &mut crate::Fnv64) {
        for x in [
            self.feet.x,
            self.feet.y,
            self.feet.z,
            self.velocity.x,
            self.velocity.y,
            self.velocity.z,
            self.eye,
            self.since_ground,
            self.jump_wanted,
            self.rest,
        ] {
            h.f32(x);
        }
        h.u32(self.grounded as u32 | (self.crouching as u32) << 1 | (self.swimming as u32) << 2);
        h.u32(self.last_buttons);
        if let Some(s) = self.sidestep {
            h.f32(s.t);
            h.f32(s.dir.x);
            h.f32(s.dir.z);
        }
        if let Some(m) = self.mantle {
            h.f32(m.t);
            h.f32(m.to.x);
            h.f32(m.to.y);
            h.f32(m.to.z);
        }
    }

    /// One fixed step of `dt` seconds of the world's time: a full step, or less while
    /// time is slowed. `yaw` is the body's facing.
    pub fn step(
        &mut self,
        input: &PlayerInput,
        yaw: f32,
        statics: Option<&Statics>,
        around: &dyn Surroundings,
        dt: f32,
    ) {
        let pressed = input.buttons & !self.last_buttons;
        self.last_buttons = input.buttons;
        let (sin, cos) = yaw.sin_cos();
        // Forward is -Z at yaw 0, and yaw turns right.
        let forward = Vec3::new(sin, 0.0, -cos);
        let right = Vec3::new(cos, 0.0, sin);
        let mut wish = Vec2::new(input.move_x, input.move_y);
        if wish.length_squared() > 1.0 {
            wish = wish.normalize();
        }
        let wish_dir = right * wish.x + forward * wish.y;

        if pressed & buttons::JUMP != 0 {
            self.jump_wanted = 0.0;
        } else {
            self.jump_wanted += dt;
        }
        self.rest = (self.rest - dt).max(0.0);

        // A mantle under way carries the body up and over, and nothing else moves it.
        if let Some(mut m) = self.mantle {
            m.t += dt / MANTLE_TIME;
            let t = m.t.min(1.0);
            // Up first, then over.
            let up = (t / 0.6).min(1.0);
            let over = ((t - 0.35) / 0.65).clamp(0.0, 1.0);
            let ease = |x: f32| x * x * (3.0 - 2.0 * x);
            self.feet = Vec3::new(
                m.from.x + (m.to.x - m.from.x) * ease(over),
                m.from.y + (m.to.y - m.from.y) * ease(up),
                m.from.z + (m.to.z - m.from.z) * ease(over),
            );
            self.mantle = if m.t >= 1.0 { None } else { Some(m) };
            if self.mantle.is_none() {
                self.velocity = wish_dir * JOG * 0.5;
                self.grounded = true;
            }
            self.ease_eye(dt);
            return;
        }

        // Water: how deep is it here?
        let water = around.water(self.feet.x, self.feet.z);
        let depth = water.map_or(0.0, |w| w - self.feet.y);
        let was_swimming = self.swimming;
        self.swimming = depth > SWIM_DEPTH || (was_swimming && depth > SWIM_DEPTH - 0.25);

        // Crouching, unless there is no room to stand up again.
        let want_crouch = input.held(buttons::SNEAK) && !self.swimming;
        if want_crouch {
            self.crouching = true;
        } else if self.crouching {
            let room = statics.is_none_or(|s| !s.blocked(self.feet, STANDING_HEIGHT));
            if room {
                self.crouching = false;
            }
        }

        // The sidestep: a quick step in the direction of movement, or back when still.
        if pressed & buttons::SIDESTEP != 0
            && self.sidestep.is_none()
            && self.rest == 0.0
            && !self.swimming
        {
            let dir = if wish_dir.length_squared() > 0.01 {
                wish_dir.normalize()
            } else {
                -forward
            };
            self.sidestep = Some(Sidestep { dir, t: 0.0 });
        }

        let speed = if self.swimming {
            if input.held(buttons::SPRINT) {
                SWIM_SPRINT
            } else {
                SWIM
            }
        } else if self.crouching {
            CROUCH_SPEED
        } else if input.held(buttons::SPRINT) {
            SPRINT
        } else {
            JOG
        };
        // Wading slows the legs.
        let wading = if self.swimming {
            1.0
        } else {
            1.0 - (depth / SWIM_DEPTH).clamp(0.0, 1.0) * 0.45
        };
        let target = wish_dir * speed * wading;
        let accel = if self.swimming {
            WATER_ACCEL
        } else if self.grounded {
            GROUND_ACCEL
        } else {
            AIR_ACCEL
        };
        let flat = Vec2::new(self.velocity.x, self.velocity.z);
        let to = Vec2::new(target.x, target.z);
        let change = to - flat;
        let step = accel * dt;
        let flat = if change.length() <= step {
            to
        } else {
            flat + change.normalize() * step
        };
        self.velocity.x = flat.x;
        self.velocity.z = flat.y;

        let mut desired;
        if let Some(mut s) = self.sidestep {
            // Fast out, easing in: most of the distance in the first half.
            let t0 = s.t / SIDESTEP_TIME;
            s.t += dt;
            let t1 = (s.t / SIDESTEP_TIME).min(1.0);
            let ease = |x: f32| 1.0 - (1.0 - x) * (1.0 - x);
            desired = s.dir * SIDESTEP_DISTANCE * (ease(t1) - ease(t0));
            self.sidestep = if s.t >= SIDESTEP_TIME { None } else { Some(s) };
            if self.sidestep.is_none() {
                self.rest = SIDESTEP_REST;
            }
            self.velocity.y = (self.velocity.y - GRAVITY * dt).max(-TERMINAL);
            desired.y += self.velocity.y * dt;
        } else if self.swimming {
            // Swimming: the body floats with its eye just above the water, and jump and
            // crouch rise and dive.
            let surface = water.unwrap_or(self.feet.y);
            let float_feet = surface + SWIM_EYE_ABOVE - EYE_STANDING;
            let mut vy = (float_feet - self.feet.y) * 3.0;
            if input.held(buttons::JUMP) {
                vy = vy.max(1.5);
            }
            if input.held(buttons::SNEAK) {
                vy = -1.8;
            }
            self.velocity.y = vy;
            desired = Vec3::new(self.velocity.x, self.velocity.y, self.velocity.z) * dt;
        } else {
            // On land: jumps, with a little grace either side of the edge.
            if self.grounded {
                self.since_ground = 0.0;
            } else {
                self.since_ground += dt;
            }
            if self.jump_wanted <= JUMP_GRACE && self.since_ground <= JUMP_GRACE && !self.crouching
            {
                self.velocity.y = JUMP_SPEED;
                self.jump_wanted = f32::MAX;
                self.since_ground = f32::MAX;
                self.grounded = false;
            }
            self.velocity.y = (self.velocity.y - GRAVITY * dt).max(-TERMINAL);
            desired = self.velocity * dt;
        }

        let Some(statics) = statics else {
            // No still world: the feet follow the ground's height, as before physics.
            self.feet += Vec3::new(desired.x, 0.0, desired.z);
            if let Some(h) = around.ground(self.feet.x, self.feet.z) {
                self.feet.y = h;
            }
            self.velocity.y = 0.0;
            self.grounded = true;
            self.ease_eye(dt);
            return;
        };

        // A ledge ahead, while pushing at it in the air or jumping at it: mantle.
        if !self.swimming
            && wish.y > 0.5
            && (!self.grounded || pressed & buttons::JUMP != 0)
            && self.sidestep.is_none()
        {
            if let Some(top) = self.ledge(statics, forward) {
                self.mantle = Some(Mantle {
                    from: self.feet,
                    to: top,
                    t: 0.0,
                });
                self.velocity = Vec3::ZERO;
                self.ease_eye(dt);
                return;
            }
        }

        let height = self.height();
        let half = height * 0.5 - RADIUS;
        let shape = SharedShape::capsule_y(half, RADIUS);
        let pose = Pose::from_translation(v(self.feet + Vec3::Y * (height * 0.5)));
        let controller = KinematicCharacterController {
            up: Vector::Y,
            offset: CharacterLength::Absolute(0.02),
            slide: true,
            autostep: Some(CharacterAutostep {
                max_height: CharacterLength::Absolute(0.38),
                min_width: CharacterLength::Absolute(0.2),
                include_dynamic_bodies: false,
            }),
            max_slope_climb_angle: MAX_SLOPE,
            min_slope_slide_angle: MAX_SLOPE,
            snap_to_ground: if self.velocity.y <= 0.0 && !self.swimming {
                Some(CharacterLength::Absolute(0.3))
            } else {
                None
            },
            normal_nudge_factor: 1.0e-4,
        };
        let moved = controller.move_shape(
            dt,
            &statics.world.query_pipeline(),
            shape.as_ref(),
            &pose,
            v(desired),
            |_| {},
        );
        let moved_by = g(moved.translation);
        self.feet += moved_by;
        self.grounded = moved.grounded && !self.swimming;
        if self.grounded && self.velocity.y < 0.0 {
            self.velocity.y = 0.0;
        }
        // A head striking a ceiling stops rising.
        if self.velocity.y > 0.0 && moved_by.y < desired.y * 0.5 {
            self.velocity.y = 0.0;
        }
        // Never below the terrain, whatever happens.
        if let Some(h) = around.ground(self.feet.x, self.feet.z) {
            if self.feet.y < h - 0.5 {
                self.feet.y = h;
                self.velocity.y = 0.0;
            }
        }
        self.ease_eye(dt);
    }

    fn ease_eye(&mut self, dt: f32) {
        let target = if self.crouching {
            EYE_CROUCHED
        } else {
            EYE_STANDING
        };
        self.eye += (target - self.eye) * (1.0 - (-dt * 14.0).exp());
    }

    /// The top of a ledge ahead that the body could climb onto, if there is one: a
    /// surface within reach of the hands, with room above it to stand.
    fn ledge(&self, statics: &Statics, forward: Vec3) -> Option<Vec3> {
        // Something must be in the way at chest height.
        let chest = self.feet + Vec3::Y * 1.0;
        statics.ray(chest, forward, RADIUS + 0.45)?;
        // Look down onto it from above the highest reach.
        let over = self.feet + forward * (RADIUS + 0.4) + Vec3::Y * (MANTLE_HEIGHT + 0.2);
        let (t, normal) = statics.ray(over, -Vec3::Y, MANTLE_HEIGHT + 0.2)?;
        let top = over - Vec3::Y * t;
        let rise = top.y - self.feet.y;
        if !(MANTLE_MIN..=MANTLE_HEIGHT).contains(&rise) || normal.y < 0.7 {
            return None;
        }
        // Room to stand, or at least to crouch, on top.
        let landing = top + Vec3::Y * 0.02;
        if statics.blocked(landing, CROUCHED_HEIGHT) {
            return None;
        }
        Some(landing)
    }
}
