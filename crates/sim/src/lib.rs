//! Game rules. No dependency on SDL or the GPU: `sim` runs headless so that bots play the
//! real game.
//!
//! The simulation advances in fixed steps. After each step it publishes an immutable
//! [`Snapshot`], which is all that drawing, bots, and replays ever read.

mod clock;
pub mod debris;
pub mod weather;

/// The ground the players walk on: its height at a point, if there is ground there.
/// The world crate answers this from the terrain; the rules only ask.
pub trait Ground: Send + Sync + std::fmt::Debug {
    fn height(&self, x: f32, z: f32) -> Option<f32>;
}
mod hash;
mod input;
mod snapshot;
mod world;

pub use clock::{FixedClock, StepPlan, STEP_HZ, STEP_SECONDS};
pub use hash::Fnv64;
pub use input::{buttons, PlayerInput};
pub use snapshot::{LetterSnap, PlayerSnap, Snapshot, ViewSnap};
pub use weather::Weather;
pub use world::{BreakableThing, PlayerId, World, WorldConfig, EYE_HEIGHT};
