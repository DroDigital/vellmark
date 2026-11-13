//! Button prompts, drawn in letters: `[SPACE]`, `[LMB]`, `[A]`, `[CROSS]`. A prompt is a
//! string table key; the words are in `assets/text/<language>/prompts.ron`, never in
//! code. Prompts follow the device a player touched last and the make of their pad.

use crate::actions::{Action, Bindings, DeviceKind, Key, MouseButton, PadButton};

/// How a controller's maker names its face buttons.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum PadStyle {
    /// A, B, X, Y: Xbox, the Steam Deck, and most others.
    #[default]
    Xbox,
    /// Cross, circle, square, triangle.
    PlayStation,
    /// B, A, Y, X, the other way about.
    Nintendo,
}

/// The string table key of the prompt for an action, on the device given, if the
/// action has a button there.
pub fn prompt_key(
    action: Action,
    device: DeviceKind,
    style: PadStyle,
    bindings: &Bindings,
) -> Option<String> {
    match device {
        DeviceKind::KeyboardMouse => {
            if let Some(b) = bindings.mouse_for(action) {
                return Some(format!("prompt.mouse.{}", mouse_name(b)));
            }
            bindings
                .key_for(action)
                .map(|k| format!("prompt.key.{}", key_name(k)))
        }
        DeviceKind::Pad => {
            if bindings.right_trigger == action {
                return Some(format!("prompt.pad.{}.right_trigger", style_name(style)));
            }
            if bindings.left_trigger == action {
                return Some(format!("prompt.pad.{}.left_trigger", style_name(style)));
            }
            bindings
                .pad_for(action)
                .map(|b| format!("prompt.pad.{}.{}", style_name(style), pad_name(b)))
        }
    }
}

pub fn style_name(style: PadStyle) -> &'static str {
    match style {
        PadStyle::Xbox => "xbox",
        PadStyle::PlayStation => "playstation",
        PadStyle::Nintendo => "nintendo",
    }
}

pub fn mouse_name(b: MouseButton) -> &'static str {
    match b {
        MouseButton::Left => "left",
        MouseButton::Right => "right",
        MouseButton::Middle => "middle",
    }
}

pub fn pad_name(b: PadButton) -> &'static str {
    match b {
        PadButton::South => "south",
        PadButton::East => "east",
        PadButton::West => "west",
        PadButton::North => "north",
        PadButton::LeftShoulder => "left_shoulder",
        PadButton::RightShoulder => "right_shoulder",
        PadButton::LeftStick => "left_stick",
        PadButton::RightStick => "right_stick",
        PadButton::Start => "start",
        PadButton::Back => "back",
        PadButton::DPadUp => "dpad_up",
        PadButton::DPadDown => "dpad_down",
        PadButton::DPadLeft => "dpad_left",
        PadButton::DPadRight => "dpad_right",
    }
}

/// Every key's name in the string table: its variant, lower case.
pub fn key_name(k: Key) -> String {
    format!("{k:?}").to_ascii_lowercase()
}

/// Every key the game can bind, for checking that each has a prompt.
pub const ALL_KEYS: &[Key] = &[
    Key::W,
    Key::A,
    Key::S,
    Key::D,
    Key::Q,
    Key::E,
    Key::R,
    Key::F,
    Key::C,
    Key::X,
    Key::Z,
    Key::V,
    Key::T,
    Key::G,
    Key::B,
    Key::I,
    Key::M,
    Key::P,
    Key::Num1,
    Key::Num2,
    Key::Num3,
    Key::Num4,
    Key::Num5,
    Key::Num6,
    Key::Num7,
    Key::Space,
    Key::LeftShift,
    Key::LeftCtrl,
    Key::LeftAlt,
    Key::Tab,
    Key::Escape,
    Key::Enter,
    Key::Backspace,
    Key::Grave,
    Key::Up,
    Key::Down,
    Key::Left,
    Key::Right,
    Key::F1,
    Key::F2,
    Key::F3,
    Key::F4,
    Key::F5,
    Key::F6,
    Key::F7,
    Key::F8,
    Key::F9,
    Key::F10,
    Key::F11,
    Key::F12,
];

pub const ALL_PAD_BUTTONS: &[PadButton] = &[
    PadButton::South,
    PadButton::East,
    PadButton::West,
    PadButton::North,
    PadButton::LeftShoulder,
    PadButton::RightShoulder,
    PadButton::LeftStick,
    PadButton::RightStick,
    PadButton::Start,
    PadButton::Back,
    PadButton::DPadUp,
    PadButton::DPadDown,
    PadButton::DPadLeft,
    PadButton::DPadRight,
];

/// The actions a player may move to another button.
pub const PLAYER_ACTIONS: &[Action] = &[
    Action::MainHand,
    Action::OffHand,
    Action::Sidestep,
    Action::Jump,
    Action::Sprint,
    Action::Sneak,
    Action::Interact,
    Action::Read,
    Action::Inventory,
    Action::Pause,
    Action::NextWord,
    Action::PreviousWord,
    Action::PhotoMode,
];

/// An action by its name, as `{:?}` writes it, in any case: `jump`, `MainHand`.
pub fn action_named(name: &str) -> Option<Action> {
    PLAYER_ACTIONS
        .iter()
        .copied()
        .find(|a| format!("{a:?}").eq_ignore_ascii_case(name))
}

pub fn key_named(name: &str) -> Option<Key> {
    ALL_KEYS
        .iter()
        .copied()
        .find(|&k| key_name(k).eq_ignore_ascii_case(name))
}

pub fn pad_button_named(name: &str) -> Option<PadButton> {
    ALL_PAD_BUTTONS
        .iter()
        .copied()
        .find(|&b| pad_name(b).eq_ignore_ascii_case(name))
}

pub fn mouse_button_named(name: &str) -> Option<MouseButton> {
    [MouseButton::Left, MouseButton::Right, MouseButton::Middle]
        .into_iter()
        .find(|&b| format!("mouse_{}", mouse_name(b)).eq_ignore_ascii_case(name))
}

/// Every prompt key there can be, for checking the string table has them all.
pub fn all_prompt_keys() -> Vec<String> {
    let mut keys: Vec<String> = ALL_KEYS
        .iter()
        .map(|&k| format!("prompt.key.{}", key_name(k)))
        .collect();
    keys.extend(
        [MouseButton::Left, MouseButton::Right, MouseButton::Middle]
            .iter()
            .map(|&b| format!("prompt.mouse.{}", mouse_name(b))),
    );
    for style in [PadStyle::Xbox, PadStyle::PlayStation, PadStyle::Nintendo] {
        keys.extend(
            ALL_PAD_BUTTONS
                .iter()
                .map(|&b| format!("prompt.pad.{}.{}", style_name(style), pad_name(b))),
        );
        keys.push(format!("prompt.pad.{}.left_trigger", style_name(style)));
        keys.push(format!("prompt.pad.{}.right_trigger", style_name(style)));
    }
    keys
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn prompts_follow_the_device_and_the_bindings() {
        let b = Bindings::default();
        assert_eq!(
            prompt_key(Action::Jump, DeviceKind::KeyboardMouse, PadStyle::Xbox, &b).unwrap(),
            "prompt.key.space"
        );
        assert_eq!(
            prompt_key(
                Action::MainHand,
                DeviceKind::KeyboardMouse,
                PadStyle::Xbox,
                &b
            )
            .unwrap(),
            "prompt.mouse.left"
        );
        assert_eq!(
            prompt_key(Action::MainHand, DeviceKind::Pad, PadStyle::Xbox, &b).unwrap(),
            "prompt.pad.xbox.right_trigger"
        );
        assert_eq!(
            prompt_key(Action::Jump, DeviceKind::Pad, PadStyle::PlayStation, &b).unwrap(),
            "prompt.pad.playstation.south"
        );
        let mut s = crate::InputSettings::default();
        s.remap_key(Action::Jump, Key::C);
        assert_eq!(
            prompt_key(
                Action::Jump,
                DeviceKind::KeyboardMouse,
                PadStyle::Xbox,
                &s.bindings()
            )
            .unwrap(),
            "prompt.key.c"
        );
    }
}
