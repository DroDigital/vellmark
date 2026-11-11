use crate::clock::STEP_SECONDS;
use crate::hash::Fnv64;
use crate::input::{buttons, PlayerInput};
use crate::debris::{shatter, Debris, Solid};
use crate::snapshot::{LetterSnap, PlayerSnap, Snapshot, ViewSnap};
use glam::{Vec2, Vec3};
use std::sync::Arc;

/// Height of the eye above the feet, in metres.
pub const EYE_HEIGHT: f32 = 1.68;

const WALK_SPEED: f32 = 4.2;
const SPRINT_SPEED: f32 = 7.0;
const MAX_PITCH: f32 = 1.48;

/// Identifies a player for the whole session. Never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlayerId(pub u32);

#[derive(Clone, Debug)]
pub struct WorldConfig {
    pub seed: u64,
    /// Hours since midnight at step 0.
    pub start_time_of_day: f32,
    /// In-game hours that pass in one real second. 0 freezes the sun.
    pub hours_per_second: f32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self { seed: 1, start_time_of_day: 10.0, hours_per_second: 24.0 / (48.0 * 60.0) }
    }
}

/// A thing that can be broken into its letters: its permanent ID, where it stands, and
/// the boxes it is made of in its own space, each with its material.
#[derive(Clone, Debug)]
pub struct BreakableThing {
    pub id: u64,
    pub to_world: glam::Affine3A,
    pub boxes: Vec<(Vec3, Vec3, u32)>,
}

#[derive(Clone, Debug)]
struct Player {
    id: PlayerId,
    feet: Vec3,
    yaw: f32,
    pitch: f32,
}

/// The whole mutable state of the game. Stepping it is deterministic: the same
/// configuration and the same inputs give the same state, bit for bit, on every run.
#[derive(Clone, Debug)]
pub struct World {
    config: WorldConfig,
    step: u64,
    rng: u64,
    time_of_day: f32,
    players: Vec<Player>,
    next_player: u32,
    debris: Debris,
    breakables: Vec<BreakableThing>,
    broken: Arc<Vec<u64>>,
    letters: Arc<Vec<LetterSnap>>,
    letters_step: u64,
    heat: Arc<Vec<(u64, f32)>>,
}

impl World {
    pub fn new(config: WorldConfig) -> Self {
        let rng = config.seed ^ 0x9e37_79b9_7f4a_7c15;
        let time_of_day = config.start_time_of_day.rem_euclid(24.0);
        Self {
            config,
            step: 0,
            rng,
            time_of_day,
            players: Vec::new(),
            next_player: 0,
            debris: Debris::default(),
            breakables: Vec::new(),
            broken: Arc::default(),
            letters: Arc::default(),
            letters_step: 0,
            heat: Arc::default(),
        }
    }

    /// Adds a player and their view. The list may hold any number, though the Showcase
    /// draws one.
    pub fn add_player(&mut self, feet: Vec3, yaw: f32) -> PlayerId {
        let id = PlayerId(self.next_player);
        self.next_player += 1;
        self.players.push(Player { id, feet, yaw, pitch: 0.0 });
        id
    }

    /// Gives the world the scene's solid boxes and the things in it that can break.
    pub fn set_scene(&mut self, solids: Vec<Solid>, breakables: Vec<BreakableThing>) {
        self.debris.set_solids(solids);
        self.breakables = breakables;
    }

    /// Breaks a thing into its letters, thrown outwards from `blow`. Returns how many
    /// letters it made, or `None` if there is no such thing or it is already broken.
    pub fn break_thing(&mut self, id: u64, blow: Vec3) -> Option<usize> {
        self.break_thing_with(id, blow, 1.0)
    }

    /// Breaks a thing with a blow of the given strength: 1 throws its letters across
    /// the yard, and near 0 it falls in on itself.
    pub fn break_thing_with(&mut self, id: u64, blow: Vec3, strength: f32) -> Option<usize> {
        if self.broken.contains(&id) {
            return None;
        }
        let thing = self.breakables.iter().find(|b| b.id == id)?.clone();
        let mut made = 0;
        for (min, max, material) in &thing.boxes {
            let mut rng = self.rng;
            let mut random = || {
                rng = rng.wrapping_add(0x9e37_79b9_7f4a_7c15);
                let mut z = rng;
                z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                z ^ (z >> 31)
            };
            made += shatter(&mut self.debris, *min, *max, thing.to_world, *material, blow, strength, &mut random);
            self.rng = rng;
        }
        let mut broken = (*self.broken).clone();
        broken.push(id);
        self.broken = Arc::new(broken);
        self.refresh_letters();
        Some(made)
    }

    /// Sets how hot or cold a thing is, -1 (frozen) to 1 (as hot as it gets). 0 lets it
    /// be. The words of S05 will set this; until then the development tools do.
    pub fn set_heat(&mut self, id: u64, heat: f32) {
        let heat = heat.clamp(-1.0, 1.0);
        let mut all: Vec<(u64, f32)> = self.heat.iter().copied().filter(|(i, _)| *i != id).collect();
        if heat != 0.0 {
            all.push((id, heat));
        }
        self.heat = Arc::new(all);
    }

    pub fn loose_letters(&self) -> usize {
        self.debris.letters.len()
    }

    pub fn letters_moving(&self) -> usize {
        self.debris.awake()
    }

    fn refresh_letters(&mut self) {
        self.letters_step = self.step + 1;
        self.letters = Arc::new(
            self.debris
                .letters
                .iter()
                .map(|l| LetterSnap { position: l.position, rotation: l.rotation, material: l.material, seed: l.seed })
                .collect(),
        );
    }

    pub fn remove_player(&mut self, id: PlayerId) {
        self.players.retain(|p| p.id != id);
    }

    pub fn step_index(&self) -> u64 {
        self.step
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    /// Deterministic random numbers for the rules. SplitMix64.
    pub fn next_random(&mut self) -> u64 {
        self.rng = self.rng.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Advances one fixed step. `inputs[i]` belongs to the i-th player in the order they
    /// were added. A missing input is treated as no input.
    pub fn step(&mut self, inputs: &[PlayerInput]) {
        for (i, p) in self.players.iter_mut().enumerate() {
            let input = inputs.get(i).copied().unwrap_or_default();
            p.yaw = (p.yaw + input.look_yaw).rem_euclid(std::f32::consts::TAU);
            p.pitch = (p.pitch + input.look_pitch).clamp(-MAX_PITCH, MAX_PITCH);

            let mut wish = Vec2::new(input.move_x, input.move_y);
            if wish.length_squared() > 1.0 {
                wish = wish.normalize();
            }
            let speed = if input.held(buttons::SPRINT) { SPRINT_SPEED } else { WALK_SPEED };
            let (sin, cos) = p.yaw.sin_cos();
            // Forward is -Z at yaw 0, and yaw turns right.
            let forward = Vec3::new(sin, 0.0, -cos);
            let right = Vec3::new(cos, 0.0, sin);
            p.feet += (right * wish.x + forward * wish.y) * speed * STEP_SECONDS;
        }
        if self.debris.awake() > 0 {
            self.debris.step(STEP_SECONDS);
            self.refresh_letters();
        }
        self.time_of_day =
            (self.time_of_day + self.config.hours_per_second * STEP_SECONDS).rem_euclid(24.0);
        self.step += 1;
    }

    /// The immutable picture of this step.
    pub fn snapshot(&self) -> Arc<Snapshot> {
        let players: Vec<PlayerSnap> = self
            .players
            .iter()
            .map(|p| PlayerSnap { id: p.id, feet: p.feet, yaw: p.yaw, pitch: p.pitch })
            .collect();
        let views = self
            .players
            .iter()
            .map(|p| ViewSnap {
                player: p.id,
                eye: p.feet + Vec3::Y * EYE_HEIGHT,
                yaw: p.yaw,
                pitch: p.pitch,
            })
            .collect();
        Arc::new(Snapshot {
            step: self.step,
            time_of_day: self.time_of_day,
            players,
            views,
            letters: self.letters.clone(),
            letters_step: self.letters_step,
            broken: self.broken.clone(),
            heat: self.heat.clone(),
        })
    }

    /// A hash of every bit of state that the rules can read. Two worlds with equal hashes
    /// will behave identically from here on.
    pub fn state_hash(&self) -> u64 {
        let mut h = Fnv64::new();
        h.u64(self.step);
        h.u64(self.rng);
        h.f32(self.time_of_day);
        h.u32(self.next_player);
        h.u32(self.players.len() as u32);
        for p in &self.players {
            h.u32(p.id.0);
            h.f32(p.feet.x);
            h.f32(p.feet.y);
            h.f32(p.feet.z);
            h.f32(p.yaw);
            h.f32(p.pitch);
        }
        h.u32(self.debris.letters.len() as u32);
        for l in &self.debris.letters {
            for v in l.position.to_array().into_iter().chain(l.velocity.to_array()) {
                h.f32(v);
            }
        }
        h.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scripted_input(world: &mut World, step: u64, player: usize) -> PlayerInput {
        let r = world.next_random();
        let t = step as f32 * 0.013 + player as f32;
        PlayerInput {
            move_x: t.sin(),
            move_y: (t * 0.7).cos(),
            look_yaw: ((r & 0xff) as f32 / 255.0 - 0.5) * 0.05,
            look_pitch: (((r >> 8) & 0xff) as f32 / 255.0 - 0.5) * 0.02,
            buttons: if step % 90 < 30 { buttons::SPRINT } else { 0 },
        }
    }

    fn run(steps: u64) -> (u64, Vec<u64>) {
        let mut w = World::new(WorldConfig { seed: 7, ..Default::default() });
        w.add_player(Vec3::ZERO, 0.0);
        w.add_player(Vec3::new(3.0, 0.0, 1.0), 1.0);
        let mut trail = Vec::new();
        for s in 0..steps {
            let inputs = [scripted_input(&mut w, s, 0), scripted_input(&mut w, s, 1)];
            w.step(&inputs);
            if s % 100 == 99 {
                trail.push(w.state_hash());
            }
        }
        (w.state_hash(), trail)
    }

    #[test]
    fn a_thousand_steps_twice_are_identical() {
        let a = run(1000);
        let b = run(1000);
        assert_eq!(a, b);
        assert_eq!(a.1.len(), 10);
    }

    #[test]
    fn different_inputs_differ() {
        assert_ne!(run(1000).0, run(999).0);
    }

    #[test]
    fn a_view_for_each_player() {
        let mut w = World::new(WorldConfig::default());
        w.add_player(Vec3::ZERO, 0.0);
        w.add_player(Vec3::X, 0.0);
        let s = w.snapshot();
        assert_eq!(s.views.len(), 2);
        assert_eq!(s.views[1].eye, Vec3::new(1.0, EYE_HEIGHT, 0.0));
    }

    #[test]
    fn forward_is_minus_z_and_yaw_turns_right() {
        let mut w = World::new(WorldConfig::default());
        w.add_player(Vec3::ZERO, 0.0);
        w.step(&[PlayerInput { move_y: 1.0, ..Default::default() }]);
        let s = w.snapshot();
        assert!(s.players[0].feet.z < 0.0);
        assert!((s.views[0].forward() - Vec3::NEG_Z).length() < 1e-5);

        let mut w = World::new(WorldConfig::default());
        w.add_player(Vec3::ZERO, std::f32::consts::FRAC_PI_2);
        w.step(&[PlayerInput { move_y: 1.0, ..Default::default() }]);
        let s = w.snapshot();
        assert!(s.players[0].feet.x > 0.0);
        assert!((s.views[0].forward() - Vec3::X).length() < 1e-5);
    }

    #[test]
    fn snapshot_is_unchanged_by_later_steps() {
        let mut w = World::new(WorldConfig::default());
        w.add_player(Vec3::ZERO, 0.0);
        let before = w.snapshot();
        let copy = (*before).clone();
        w.step(&[PlayerInput { move_y: 1.0, ..Default::default() }]);
        assert_eq!(*before, copy);
        assert_ne!(*w.snapshot(), copy);
    }
}
