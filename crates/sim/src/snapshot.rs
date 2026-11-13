use crate::world::PlayerId;
use glam::{Quat, Vec3};

/// One player as drawn.
#[derive(Clone, Debug, PartialEq)]
pub struct PlayerSnap {
    pub id: PlayerId,
    pub feet: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    pub grounded: bool,
    pub crouching: bool,
    pub swimming: bool,
    /// Attacks pass through it now: the first moments of a sidestep.
    pub untouchable: bool,
    /// What it is reading, if anything.
    pub reading: crate::reading::ReadingSnap,
    /// The text it would read if it held Read now, by its ID.
    pub in_focus: Option<u64>,
    pub ink: f32,
    /// What each hand holds: main, then off.
    pub holding: [crate::casting::Holding; 2],
    /// Each hand's charge, 0 to 1, and both hands' together.
    pub charge: [f32; 3],
    /// What the crosshair shows for each hand's word, if it holds one and something is
    /// in the gaze: the target, and what the word would do.
    pub preview: [Option<(u64, crate::emendation::Preview)>; 2],
}

/// A thing with an Inscription, as drawn.
#[derive(Clone, Debug, PartialEq)]
pub struct TargetSnap {
    pub id: u64,
    pub centre: Vec3,
    pub radius: f32,
    /// Its words, in order.
    pub words: Vec<String>,
    /// The effects of the written words on it now.
    pub effects: Vec<String>,
}

/// A word on its way.
#[derive(Clone, Debug, PartialEq)]
pub struct FlightSnap {
    pub word: String,
    pub from: Vec3,
    pub at: Vec3,
    pub target: u64,
    pub charge: f32,
    /// It will come out as its word's typo, and goes wrong halfway.
    pub typo: String,
}

/// One camera. There is one for each player, and the renderer draws the whole list.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ViewSnap {
    pub player: PlayerId,
    pub eye: Vec3,
    pub yaw: f32,
    pub pitch: f32,
}

impl ViewSnap {
    pub fn rotation(&self) -> Quat {
        // Yaw turns right (clockwise seen from above), pitch looks up. -Z is forward.
        Quat::from_rotation_y(-self.yaw) * Quat::from_rotation_x(self.pitch)
    }
    pub fn forward(&self) -> Vec3 {
        self.rotation() * Vec3::NEG_Z
    }
}

/// One loose letter as drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct LetterSnap {
    pub position: Vec3,
    pub rotation: Quat,
    pub material: u32,
    pub seed: u32,
}

/// The state of the world after a step, fixed for ever. Shared by reference between the
/// renderer, the bots, and replays.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub step: u64,
    /// Hours since midnight, 0 to 24.
    pub time_of_day: f32,
    pub players: Vec<PlayerSnap>,
    pub views: Vec<ViewSnap>,
    /// Every loose letter. Shared with the last snapshot while none of them is moving.
    pub letters: std::sync::Arc<Vec<LetterSnap>>,
    /// The step at which the loose letters last changed.
    pub letters_step: u64,
    /// The permanent IDs of the things that have been broken, which are no longer drawn.
    pub broken: std::sync::Arc<Vec<u64>>,
    /// How hot or cold things are, by permanent ID, -1 to 1. Things not listed are as
    /// the weather leaves them.
    pub heat: std::sync::Arc<Vec<(u64, f32)>>,
    /// The weather of this step.
    pub weather: crate::Weather,
    /// Every thing with an Inscription. Shared with the last snapshot while none changed.
    pub targets: std::sync::Arc<Vec<TargetSnap>>,
    /// Words in flight.
    pub flights: Vec<FlightSnap>,
    /// Words that landed this step.
    pub landings: Vec<crate::casting::CastEvent>,
}

impl Snapshot {
    /// The view to draw between two snapshots. `alpha` 0 is `self`, 1 is `next`.
    pub fn lerp_view(&self, next: &Snapshot, index: usize, alpha: f32) -> ViewSnap {
        let a = self.views[index];
        let Some(b) = next.views.get(index).copied() else { return a };
        ViewSnap {
            player: b.player,
            eye: a.eye.lerp(b.eye, alpha),
            yaw: a.yaw + shortest_angle(a.yaw, b.yaw) * alpha,
            pitch: a.pitch + (b.pitch - a.pitch) * alpha,
        }
    }
}

fn shortest_angle(from: f32, to: f32) -> f32 {
    let tau = std::f32::consts::TAU;
    let d = (to - from).rem_euclid(tau);
    if d > std::f32::consts::PI { d - tau } else { d }
}
