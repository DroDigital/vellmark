//! The OS layer: a window, events, and devices, through SDL3. SDL is used only for OS
//! access; nothing here draws.
//!
//! Game code never reads a raw button. Devices produce [`RawInput`], the bindings turn
//! it into [`Action`]s, and everything above reads an [`ActionState`].

mod actions;
mod device;
pub mod prompts;
mod sdl;
mod settings;

pub use actions::{
    Action, ActionState, Bindings, DeviceKind, Key, MouseButton, PadAxis, PadButton, RawInput,
};
pub use device::{InputSource, SimulatedDevice};
pub use sdl::{AudioOut, Platform, PlatformEvent, Window, WindowOptions, message_box};
pub use settings::InputSettings;
