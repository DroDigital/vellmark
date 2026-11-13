/// Keys the game can bind. Named by position on the keyboard, not by the letter
/// printed on it, so that the defaults sit under the same fingers on every layout.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum Key {
    W, A, S, D, Q, E, R, F, C, X, Z, V, T, G, B, I, M, P,
    Num1, Num2, Num3, Num4, Num5, Num6, Num7,
    Space, LeftShift, LeftCtrl, LeftAlt, Tab, Escape, Enter, Backspace, Grave,
    Up, Down, Left, Right,
    F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum MouseButton {
    Left,
    Right,
    Middle,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub enum PadButton {
    South, East, West, North,
    LeftShoulder, RightShoulder,
    LeftStick, RightStick,
    Start, Back,
    DPadUp, DPadDown, DPadLeft, DPadRight,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum PadAxis {
    LeftX,
    LeftY,
    RightX,
    RightY,
    LeftTrigger,
    RightTrigger,
}

/// What a device reports, before it means anything.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum RawInput {
    Key { key: Key, down: bool },
    MouseMove { dx: f32, dy: f32 },
    MouseButton { button: MouseButton, down: bool },
    MouseWheel { dy: f32 },
    /// `pad` identifies the controller for as long as it stays plugged in.
    PadButton { pad: u32, button: PadButton, down: bool },
    /// Sticks are -1 to 1, with down and right positive. Triggers are 0 to 1.
    PadAxis { pad: u32, axis: PadAxis, value: f32 },
    /// A typed character, while text entry is on.
    Text(char),
    PadAdded { pad: u32 },
    PadRemoved { pad: u32 },
}

/// Everything the game can be asked to do with a button.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
#[repr(u8)]
pub enum Action {
    /// The main hand: tap to strike, hold for a heavy attack.
    MainHand,
    /// The off hand: tap or hold, a word or a shield.
    OffHand,
    Sidestep,
    Jump,
    Sprint,
    Sneak,
    Interact,
    /// Hold to read what is in front of you.
    Read,
    Inventory,
    Pause,
    NextWord,
    PreviousWord,
    PhotoMode,
    /// Development builds only.
    DevConsole,
    DevFreeCamera,
    DevProfiler,
    DevGlyphLayer,
    DevTuning,
    DevLess,
    DevMore,
    DevNext,
    DevLamps,
    DevLook,
    DevBreak,
}

const ACTION_COUNT: usize = Action::DevBreak as usize + 1;

/// The device a player last touched. Button prompts follow it.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DeviceKind {
    #[default]
    KeyboardMouse,
    Pad,
}

/// Which raw inputs mean which actions. One table for each player.
#[derive(Clone, Debug)]
pub struct Bindings {
    pub keys: Vec<(Key, Action)>,
    pub mouse: Vec<(MouseButton, Action)>,
    pub pad: Vec<(PadButton, Action)>,
    /// The triggers are the two hands on a controller.
    pub left_trigger: Action,
    pub right_trigger: Action,
    pub move_keys: [Key; 4], // forward, back, left, right
    pub stick_dead_zone: f32,
    pub trigger_threshold: f32,
    /// Radians of turn for one count of mouse movement.
    pub mouse_sensitivity: f32,
    /// Radians a second at full stick.
    pub stick_turn_rate: f32,
    pub invert_y: bool,
}

impl Default for Bindings {
    fn default() -> Self {
        use Action::*;
        Self {
            keys: vec![
                (Key::Space, Jump),
                (Key::LeftShift, Sprint),
                (Key::LeftCtrl, Sneak),
                (Key::LeftAlt, Sidestep),
                (Key::E, Interact),
                (Key::R, Read),
                (Key::Tab, Inventory),
                (Key::Escape, Pause),
                (Key::Q, PreviousWord),
                (Key::F, NextWord),
                (Key::P, PhotoMode),
                (Key::Grave, DevConsole),
                (Key::F1, DevProfiler),
                (Key::F2, DevFreeCamera),
                (Key::F3, DevGlyphLayer),
                (Key::F4, DevTuning),
                (Key::Left, DevLess),
                (Key::Right, DevMore),
                (Key::Down, DevNext),
                (Key::F5, DevLamps),
                (Key::F6, DevLook),
                (Key::F7, DevBreak),
            ],
            mouse: vec![(MouseButton::Left, MainHand), (MouseButton::Right, OffHand)],
            pad: vec![
                (PadButton::South, Jump),
                (PadButton::East, Sidestep),
                (PadButton::West, Interact),
                (PadButton::North, Read),
                (PadButton::LeftStick, Sprint),
                (PadButton::RightStick, Sneak),
                (PadButton::LeftShoulder, PreviousWord),
                (PadButton::RightShoulder, NextWord),
                (PadButton::Start, Pause),
                (PadButton::Back, Inventory),
                (PadButton::DPadUp, PhotoMode),
            ],
            left_trigger: OffHand,
            right_trigger: MainHand,
            move_keys: [Key::W, Key::S, Key::A, Key::D],
            stick_dead_zone: 0.18,
            trigger_threshold: 0.35,
            mouse_sensitivity: 0.0022,
            stick_turn_rate: 3.2,
            invert_y: false,
        }
    }
}

/// The state of one player's actions for one frame.
#[derive(Clone, Debug, Default)]
pub struct ActionState {
    held: u32,
    pressed: u32,
    released: u32,
    move_keys: [bool; 4],
    stick_move: (f32, f32),
    stick_look: (f32, f32),
    mouse_look: (f32, f32),
    triggers: (bool, bool),
    pub last_device: DeviceKind,
    /// The controller this player holds, if any. Input from other controllers is
    /// ignored, so a second controller never moves the first player.
    pub pad: Option<u32>,
    /// Set when this player's controller was unplugged. Only this player is asked to
    /// reconnect.
    pub pad_lost: bool,
}

fn bit(action: Action) -> u32 {
    debug_assert!((action as usize) < ACTION_COUNT && ACTION_COUNT <= 32);
    1 << action as u32
}

fn dead_zone(v: f32, zone: f32) -> f32 {
    if v.abs() <= zone { 0.0 } else { (v.abs() - zone) / (1.0 - zone) * v.signum() }
}

impl ActionState {
    /// Call at the start of each frame, before feeding the frame's input.
    pub fn begin_frame(&mut self) {
        self.pressed = 0;
        self.released = 0;
        self.mouse_look = (0.0, 0.0);
    }

    fn set(&mut self, action: Action, down: bool) {
        let b = bit(action);
        if down && self.held & b == 0 {
            self.held |= b;
            self.pressed |= b;
        } else if !down && self.held & b != 0 {
            self.held &= !b;
            self.released |= b;
        }
    }

    pub fn feed(&mut self, bindings: &Bindings, input: RawInput) {
        match input {
            RawInput::Key { key, down } => {
                self.last_device = DeviceKind::KeyboardMouse;
                if let Some(i) = bindings.move_keys.iter().position(|k| *k == key) {
                    self.move_keys[i] = down;
                }
                for (_, action) in bindings.keys.iter().filter(|(k, _)| *k == key) {
                    self.set(*action, down);
                }
            }
            RawInput::MouseMove { dx, dy } => {
                self.last_device = DeviceKind::KeyboardMouse;
                self.mouse_look.0 += dx;
                self.mouse_look.1 += dy;
            }
            RawInput::MouseButton { button, down } => {
                self.last_device = DeviceKind::KeyboardMouse;
                for (_, action) in bindings.mouse.iter().filter(|(b, _)| *b == button) {
                    self.set(*action, down);
                }
            }
            RawInput::MouseWheel { dy } => {
                if dy != 0.0 {
                    let action = if dy > 0.0 { Action::PreviousWord } else { Action::NextWord };
                    self.pressed |= bit(action);
                }
            }
            RawInput::Text(_) => {}
            RawInput::PadAdded { pad } => {
                if self.pad.is_none() {
                    self.pad = Some(pad);
                    self.pad_lost = false;
                }
            }
            RawInput::PadRemoved { pad } => {
                if self.pad == Some(pad) {
                    self.pad = None;
                    self.pad_lost = true;
                    self.stick_move = (0.0, 0.0);
                    self.stick_look = (0.0, 0.0);
                    for (_, action) in &bindings.pad {
                        self.set(*action, false);
                    }
                    self.set(bindings.left_trigger, false);
                    self.set(bindings.right_trigger, false);
                    self.triggers = (false, false);
                }
            }
            RawInput::PadButton { pad, button, down } => {
                if self.pad != Some(pad) {
                    return;
                }
                self.last_device = DeviceKind::Pad;
                for (_, action) in bindings.pad.iter().filter(|(b, _)| *b == button) {
                    self.set(*action, down);
                }
            }
            RawInput::PadAxis { pad, axis, value } => {
                if self.pad != Some(pad) {
                    return;
                }
                let zone = bindings.stick_dead_zone;
                match axis {
                    PadAxis::LeftX => self.stick_move.0 = dead_zone(value, zone),
                    PadAxis::LeftY => self.stick_move.1 = dead_zone(value, zone),
                    PadAxis::RightX => self.stick_look.0 = dead_zone(value, zone),
                    PadAxis::RightY => self.stick_look.1 = dead_zone(value, zone),
                    PadAxis::LeftTrigger => {
                        let down = value > bindings.trigger_threshold;
                        if down != self.triggers.0 {
                            self.triggers.0 = down;
                            self.set(bindings.left_trigger, down);
                        }
                    }
                    PadAxis::RightTrigger => {
                        let down = value > bindings.trigger_threshold;
                        if down != self.triggers.1 {
                            self.triggers.1 = down;
                            self.set(bindings.right_trigger, down);
                        }
                    }
                }
                if value.abs() > 0.5 {
                    self.last_device = DeviceKind::Pad;
                }
            }
        }
    }

    pub fn held(&self, action: Action) -> bool {
        self.held & bit(action) != 0
    }

    /// True only in the frame the action went down.
    pub fn pressed(&self, action: Action) -> bool {
        self.pressed & bit(action) != 0
    }

    pub fn released(&self, action: Action) -> bool {
        self.released & bit(action) != 0
    }

    /// Strafe and advance, each -1 to 1. Right and forward are positive.
    pub fn movement(&self) -> (f32, f32) {
        let k = &self.move_keys;
        let keys = (
            (k[3] as i32 - k[2] as i32) as f32,
            (k[0] as i32 - k[1] as i32) as f32,
        );
        // A stick's Y is positive downwards.
        let x = (keys.0 + self.stick_move.0).clamp(-1.0, 1.0);
        let y = (keys.1 - self.stick_move.1).clamp(-1.0, 1.0);
        (x, y)
    }

    /// Turn and tilt for a frame of `dt` seconds, in radians. Right and up are positive.
    pub fn look(&self, bindings: &Bindings, dt: f32) -> (f32, f32) {
        let flip = if bindings.invert_y { -1.0 } else { 1.0 };
        let yaw = self.mouse_look.0 * bindings.mouse_sensitivity + self.stick_look.0 * bindings.stick_turn_rate * dt;
        let pitch = -self.mouse_look.1 * bindings.mouse_sensitivity - self.stick_look.1 * bindings.stick_turn_rate * dt;
        (yaw, pitch * flip)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pressed_lasts_one_frame() {
        let b = Bindings::default();
        let mut s = ActionState::default();
        s.begin_frame();
        s.feed(&b, RawInput::Key { key: Key::Space, down: true });
        assert!(s.pressed(Action::Jump) && s.held(Action::Jump));
        s.begin_frame();
        assert!(!s.pressed(Action::Jump) && s.held(Action::Jump));
        s.feed(&b, RawInput::Key { key: Key::Space, down: false });
        assert!(s.released(Action::Jump) && !s.held(Action::Jump));
    }

    #[test]
    fn keys_and_stick_both_move() {
        let b = Bindings::default();
        let mut s = ActionState::default();
        s.feed(&b, RawInput::Key { key: Key::W, down: true });
        s.feed(&b, RawInput::Key { key: Key::D, down: true });
        assert_eq!(s.movement(), (1.0, 1.0));
        s.feed(&b, RawInput::Key { key: Key::W, down: false });
        s.feed(&b, RawInput::Key { key: Key::D, down: false });
        s.feed(&b, RawInput::PadAdded { pad: 4 });
        s.feed(&b, RawInput::PadAxis { pad: 4, axis: PadAxis::LeftY, value: -1.0 });
        assert_eq!(s.movement(), (0.0, 1.0));
        s.feed(&b, RawInput::PadAxis { pad: 4, axis: PadAxis::LeftY, value: 0.1 });
        assert_eq!(s.movement(), (0.0, 0.0));
    }

    #[test]
    fn each_hand_has_its_own_button_on_both_devices() {
        let b = Bindings::default();
        let mut s = ActionState::default();
        s.feed(&b, RawInput::MouseButton { button: MouseButton::Left, down: true });
        s.feed(&b, RawInput::MouseButton { button: MouseButton::Right, down: true });
        assert!(s.held(Action::MainHand) && s.held(Action::OffHand));
        assert_eq!(s.last_device, DeviceKind::KeyboardMouse);

        let mut s = ActionState::default();
        s.feed(&b, RawInput::PadAdded { pad: 1 });
        s.feed(&b, RawInput::PadAxis { pad: 1, axis: PadAxis::RightTrigger, value: 0.9 });
        assert!(s.pressed(Action::MainHand) && !s.held(Action::OffHand));
        assert_eq!(s.last_device, DeviceKind::Pad);
        s.feed(&b, RawInput::PadAxis { pad: 1, axis: PadAxis::RightTrigger, value: 0.0 });
        assert!(s.released(Action::MainHand));
    }

    #[test]
    fn another_players_controller_is_ignored() {
        let b = Bindings::default();
        let mut s = ActionState::default();
        s.feed(&b, RawInput::PadAdded { pad: 1 });
        s.feed(&b, RawInput::PadAdded { pad: 2 });
        s.feed(&b, RawInput::PadButton { pad: 2, button: PadButton::South, down: true });
        assert!(!s.held(Action::Jump));
        s.feed(&b, RawInput::PadButton { pad: 1, button: PadButton::South, down: true });
        assert!(s.held(Action::Jump));
    }

    #[test]
    fn unplugging_releases_everything_and_asks_only_that_player() {
        let b = Bindings::default();
        let mut s = ActionState::default();
        s.feed(&b, RawInput::PadAdded { pad: 1 });
        s.feed(&b, RawInput::PadButton { pad: 1, button: PadButton::South, down: true });
        s.feed(&b, RawInput::PadAxis { pad: 1, axis: PadAxis::LeftX, value: 1.0 });
        s.feed(&b, RawInput::PadRemoved { pad: 9 });
        assert!(!s.pad_lost && s.held(Action::Jump));
        s.feed(&b, RawInput::PadRemoved { pad: 1 });
        assert!(s.pad_lost && !s.held(Action::Jump));
        assert_eq!(s.movement(), (0.0, 0.0));
        s.feed(&b, RawInput::PadAdded { pad: 3 });
        assert!(!s.pad_lost && s.pad == Some(3));
    }

    #[test]
    fn mouse_look_is_per_frame_and_up_is_positive() {
        let b = Bindings::default();
        let mut s = ActionState::default();
        s.begin_frame();
        s.feed(&b, RawInput::MouseMove { dx: 100.0, dy: -50.0 });
        let (yaw, pitch) = s.look(&b, 1.0 / 60.0);
        assert!(yaw > 0.0 && pitch > 0.0);
        s.begin_frame();
        assert_eq!(s.look(&b, 1.0 / 60.0), (0.0, 0.0));
    }
}
