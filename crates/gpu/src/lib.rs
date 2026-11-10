//! The GPU layer: one `wgpu` device, per-pass timers, and the registry through which
//! every pipeline is made. Nothing here knows what is drawn.

mod device;
mod pipelines;
mod readback;
mod timers;

pub use device::{Gpu, GpuError, GpuOptions, WindowHandles};
pub use pipelines::{PipelinePhase, PipelineRegistry};
pub use readback::read_texture_rgba8;
pub use timers::{PassTimers, PassTiming};
pub use wgpu;
