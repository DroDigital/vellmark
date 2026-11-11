//! Game rules. No dependency on SDL or the GPU: `sim` runs headless so that bots play the
//! real game.
//!
//! The simulation advances in fixed steps. After each step it publishes an immutable
//! [`Snapshot`], which is all that drawing, bots, and replays ever read.

mod clock;
pub mod debris;
mod hash;
mod input;
mod snapshot;
mod world;

pub use clock::{FixedClock, StepPlan, STEP_HZ, STEP_SECONDS};
pub use hash::Fnv64;
pub use input::{buttons, PlayerInput};
pub use snapshot::{LetterSnap, PlayerSnap, Snapshot, ViewSnap};
pub use world::{BreakableThing, PlayerId, World, WorldConfig, EYE_HEIGHT};
