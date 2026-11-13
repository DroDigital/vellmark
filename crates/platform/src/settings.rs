//! The player's input settings: how fast the view turns, which way is up, how much aim
//! assist, and which buttons do what. Kept in the user's settings file, read at start
//! and written when changed. Only what differs from the defaults is remapped.

use serde::{Deserialize, Serialize};

use crate::actions::{Action, Bindings, Key, MouseButton, PadButton};

/// Multipliers on the defaults, so that a later change of default carries through.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct InputSettings {
    /// 0.2 to 4: how fast the mouse turns the view.
    pub mouse_sensitivity: f32,
    /// 0.2 to 4: how fast a full stick turns it.
    pub stick_sensitivity: f32,
    pub invert_y: bool,
    /// 0 off to 1 strong: how much a controller's aim is drawn onto targets.
    pub aim_assist: f32,
    /// Remapped buttons: each action named here is moved to the button given.
    pub keys: Vec<(Action, Key)>,
    pub mouse: Vec<(Action, MouseButton)>,
    pub pad: Vec<(Action, PadButton)>,
}

impl Default for InputSettings {
    fn default() -> Self {
        Self {
            mouse_sensitivity: 1.0,
            stick_sensitivity: 1.0,
            invert_y: false,
            aim_assist: 0.5,
            keys: Vec::new(),
            mouse: Vec::new(),
            pad: Vec::new(),
        }
    }
}

impl InputSettings {
    /// Reads the settings, or gives the defaults if there are none or they cannot be read.
    pub fn parse(text: &str) -> Result<Self, String> {
        ron::from_str(text).map_err(|e| e.to_string())
    }

    pub fn to_text(&self) -> String {
        ron::ser::to_string_pretty(self, ron::ser::PrettyConfig::default()).unwrap_or_default()
    }

    /// The bindings these settings make from the defaults.
    pub fn bindings(&self) -> Bindings {
        let mut b = Bindings::default();
        b.mouse_sensitivity *= self.mouse_sensitivity.clamp(0.2, 4.0);
        b.stick_turn_rate *= self.stick_sensitivity.clamp(0.2, 4.0);
        b.invert_y = self.invert_y;
        for &(action, key) in &self.keys {
            b.rebind_key(action, key);
        }
        for &(action, button) in &self.mouse {
            b.rebind_mouse(action, button);
        }
        for &(action, button) in &self.pad {
            b.rebind_pad(action, button);
        }
        b
    }

    /// Moves an action to a key, remembering it.
    pub fn remap_key(&mut self, action: Action, key: Key) {
        self.keys.retain(|(a, _)| *a != action);
        self.keys.push((action, key));
    }

    pub fn remap_mouse(&mut self, action: Action, button: MouseButton) {
        self.mouse.retain(|(a, _)| *a != action);
        self.mouse.push((action, button));
    }

    pub fn remap_pad(&mut self, action: Action, button: PadButton) {
        self.pad.retain(|(a, _)| *a != action);
        self.pad.push((action, button));
    }
}

impl Bindings {
    /// Puts `action` on `key`. Whatever was on that key moves to where `action` was, so
    /// that no action is left without a button.
    pub fn rebind_key(&mut self, action: Action, key: Key) {
        let old = self
            .keys
            .iter()
            .find(|(_, a)| *a == action)
            .map(|(k, _)| *k);
        let displaced = self.keys.iter().find(|(k, _)| *k == key).map(|(_, a)| *a);
        self.keys.retain(|(k, a)| *a != action && *k != key);
        self.keys.push((key, action));
        if let (Some(old), Some(other)) = (old, displaced) {
            if other != action {
                self.keys.push((old, other));
            }
        }
    }

    pub fn rebind_mouse(&mut self, action: Action, button: MouseButton) {
        let old = self
            .mouse
            .iter()
            .find(|(_, a)| *a == action)
            .map(|(b, _)| *b);
        let displaced = self
            .mouse
            .iter()
            .find(|(b, _)| *b == button)
            .map(|(_, a)| *a);
        self.mouse.retain(|(b, a)| *a != action && *b != button);
        self.mouse.push((button, action));
        if let (Some(old), Some(other)) = (old, displaced) {
            if other != action {
                self.mouse.push((old, other));
            }
        }
    }

    pub fn rebind_pad(&mut self, action: Action, button: PadButton) {
        let old = self.pad.iter().find(|(_, a)| *a == action).map(|(b, _)| *b);
        let displaced = self.pad.iter().find(|(b, _)| *b == button).map(|(_, a)| *a);
        self.pad.retain(|(b, a)| *a != action && *b != button);
        self.pad.push((button, action));
        if let (Some(old), Some(other)) = (old, displaced) {
            if other != action {
                self.pad.push((old, other));
            }
        }
    }

    /// The key an action is on, if any.
    pub fn key_for(&self, action: Action) -> Option<Key> {
        self.keys
            .iter()
            .find(|(_, a)| *a == action)
            .map(|(k, _)| *k)
    }

    pub fn mouse_for(&self, action: Action) -> Option<MouseButton> {
        self.mouse
            .iter()
            .find(|(_, a)| *a == action)
            .map(|(b, _)| *b)
    }

    pub fn pad_for(&self, action: Action) -> Option<PadButton> {
        self.pad.iter().find(|(_, a)| *a == action).map(|(b, _)| *b)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn settings_round_trip_and_unknown_fields_fall_back() {
        let mut s = InputSettings {
            mouse_sensitivity: 1.5,
            invert_y: true,
            ..Default::default()
        };
        s.remap_key(Action::Jump, Key::C);
        let back = InputSettings::parse(&s.to_text()).unwrap();
        assert_eq!(back, s);
        // A file that names only some settings keeps the defaults for the rest.
        let partial = InputSettings::parse("(invert_y: true)").unwrap();
        assert!(partial.invert_y && partial.mouse_sensitivity == 1.0);
    }

    #[test]
    fn remapping_swaps_so_nothing_is_lost() {
        let mut s = InputSettings::default();
        // Jump onto Left Ctrl, where Sneak was: Sneak moves to Space.
        s.remap_key(Action::Jump, Key::LeftCtrl);
        let b = s.bindings();
        assert_eq!(b.key_for(Action::Jump), Some(Key::LeftCtrl));
        assert_eq!(b.key_for(Action::Sneak), Some(Key::Space));
        // The sidestep onto the pad's south button, where Jump was.
        s.remap_pad(Action::Sidestep, PadButton::South);
        let b = s.bindings();
        assert_eq!(b.pad_for(Action::Sidestep), Some(PadButton::South));
        assert_eq!(b.pad_for(Action::Jump), Some(PadButton::East));
        // The hands swap mouse buttons.
        s.remap_mouse(Action::MainHand, MouseButton::Right);
        let b = s.bindings();
        assert_eq!(b.mouse_for(Action::MainHand), Some(MouseButton::Right));
        assert_eq!(b.mouse_for(Action::OffHand), Some(MouseButton::Left));
    }

    #[test]
    fn sensitivity_and_invert_reach_the_bindings() {
        let d = Bindings::default();
        let b = InputSettings {
            mouse_sensitivity: 2.0,
            stick_sensitivity: 0.5,
            invert_y: true,
            ..Default::default()
        }
        .bindings();
        assert!((b.mouse_sensitivity - d.mouse_sensitivity * 2.0).abs() < 1e-6);
        assert!((b.stick_turn_rate - d.stick_turn_rate * 0.5).abs() < 1e-6);
        assert!(b.invert_y);
    }
}
