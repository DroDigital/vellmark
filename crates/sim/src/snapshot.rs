use crate::world::PlayerId;
use glam::{Quat, Vec3};

/// One player as drawn.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct PlayerSnap {
    pub id: PlayerId,
    pub feet: Vec3,
    pub yaw: f32,
    pub pitch: f32,
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

/// The state of the world after a step, fixed for ever. Shared by reference between the
/// renderer, the bots, and replays.
#[derive(Clone, Debug, PartialEq)]
pub struct Snapshot {
    pub step: u64,
    /// Hours since midnight, 0 to 24.
    pub time_of_day: f32,
    pub players: Vec<PlayerSnap>,
    pub views: Vec<ViewSnap>,
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
