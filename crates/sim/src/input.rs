/// What one player asks for during one simulation step. This is the whole interface
/// between the device layer (or a bot, or a replay) and the rules.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct PlayerInput {
    /// Strafe, -1 left to 1 right.
    pub move_x: f32,
    /// Advance, -1 back to 1 forward.
    pub move_y: f32,
    /// Turn this step, in radians. Positive turns right.
    pub look_yaw: f32,
    /// Tilt this step, in radians. Positive looks up.
    pub look_pitch: f32,
    /// Bits from [`buttons`].
    pub buttons: u32,
}

/// Held buttons, as bits of [`PlayerInput::buttons`].
pub mod buttons {
    pub const SPRINT: u32 = 1 << 0;
    pub const JUMP: u32 = 1 << 1;
    pub const SNEAK: u32 = 1 << 2;
    pub const INTERACT: u32 = 1 << 3;
    pub const READ: u32 = 1 << 4;
    pub const ATTACK_MAIN: u32 = 1 << 5;
    pub const ATTACK_OFF: u32 = 1 << 6;
    pub const SIDESTEP: u32 = 1 << 7;
}

impl PlayerInput {
    pub fn held(&self, bit: u32) -> bool {
        self.buttons & bit != 0
    }
}
