//! The pause menu and the settings: what each page holds, and what a press on it does.
//! Pages are drawn by `vellmark_ui::menu`; this decides their entries from the settings
//! and changes the settings as the player asks.

use vellmark_content::{sid, StringId};
use vellmark_platform::prompts::{self, PadStyle};
use vellmark_platform::{Action, Bindings, DeviceKind, Key, PadButton, RawInput, Settings};
use crate::kit::Focus;
use crate::menu::{Entry, MenuInput, MenuRepeat, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Page {
    Pause,
    Settings,
    Controls,
    Buttons,
    Display,
    Sound,
    Access,
}

/// What a press asked of the game.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Outcome {
    None,
    Resume,
    Save,
    Load,
    Quit,
    /// Photo mode, from the pause menu.
    Photo,
    /// The credits.
    Credits,
    /// Back to the beginning: the world as it was at the start, and the title.
    StartAgain,
    /// The settings changed: apply them and write them.
    Changed,
}

/// The actions a player may move to another button, in the order the page lists them.
pub const REMAPPABLE: [Action; 16] = [
    Action::MainHand,
    Action::OffHand,
    Action::Interact,
    Action::Read,
    Action::Jump,
    Action::Sprint,
    Action::Sneak,
    Action::Sidestep,
    Action::PreviousWord,
    Action::NextWord,
    Action::Inventory,
    Action::Favourites,
    Action::MoveForward,
    Action::MoveBack,
    Action::MoveLeft,
    Action::MoveRight,
];

fn action_label(action: Action) -> StringId {
    match action {
        Action::MainHand => sid!("ui.action.main_hand"),
        Action::OffHand => sid!("ui.action.off_hand"),
        Action::Interact => sid!("ui.action.interact"),
        Action::Read => sid!("ui.action.read"),
        Action::Jump => sid!("ui.action.jump"),
        Action::Sprint => sid!("ui.action.sprint"),
        Action::Sneak => sid!("ui.action.sneak"),
        Action::Sidestep => sid!("ui.action.sidestep"),
        Action::PreviousWord => sid!("ui.action.previous_word"),
        Action::NextWord => sid!("ui.action.next_word"),
        Action::MoveForward => sid!("ui.action.move_forward"),
        Action::MoveBack => sid!("ui.action.move_back"),
        Action::MoveLeft => sid!("ui.action.move_left"),
        Action::MoveRight => sid!("ui.action.move_right"),
        Action::Favourites => sid!("ui.action.favourites"),
        _ => sid!("ui.action.inventory"),
    }
}

const PRESETS: [&str; 3] = ["low", "medium", "high"];

pub struct Menu {
    stack: Vec<(Page, Focus)>,
    /// The action waiting for a button to be pressed for it.
    capturing: Option<Action>,
    pub repeat: MenuRepeat,
}

impl Menu {
    pub fn open() -> Self {
        Self { stack: vec![(Page::Pause, Focus::default())], capturing: None, repeat: MenuRepeat::default() }
    }

    pub fn page(&self) -> Page {
        self.stack.last().map_or(Page::Pause, |p| p.0)
    }

    pub fn focus(&self) -> Focus {
        self.stack.last().map_or_else(Focus::default, |p| p.1)
    }

    pub fn capturing(&self) -> bool {
        self.capturing.is_some()
    }

    pub fn title(&self) -> StringId {
        match self.page() {
            Page::Pause => sid!("ui.menu.paused"),
            Page::Settings => sid!("ui.menu.settings"),
            Page::Controls => sid!("ui.menu.controls"),
            Page::Buttons => sid!("ui.menu.buttons"),
            Page::Display => sid!("ui.menu.display"),
            Page::Sound => sid!("ui.menu.sound"),
            Page::Access => sid!("ui.menu.access"),
        }
    }

    /// The current page's entries, as the settings now stand.
    pub fn entries(&self, s: &Settings, bindings: &Bindings, device: DeviceKind, style: PadStyle) -> Vec<Entry> {
        let e = |label: StringId, value: Value| Entry { label, value };
        let pct = |value: f32, min: f32, max: f32| Value::Slider { value, min, max, shown: 100.0 };
        match self.page() {
            Page::Pause => vec![
                e(sid!("ui.menu.resume"), Value::Button),
                e(sid!("ui.menu.settings"), Value::Button),
                e(sid!("ui.menu.photo"), Value::Button),
                e(sid!("ui.menu.save"), Value::Button),
                e(sid!("ui.menu.load"), Value::Button),
                e(sid!("ui.menu.credits"), Value::Button),
                e(sid!("ui.menu.start_again"), Value::Button),
                e(sid!("ui.menu.quit"), Value::Button),
            ],
            Page::Settings => vec![
                e(sid!("ui.menu.controls"), Value::Button),
                e(sid!("ui.menu.display"), Value::Button),
                e(sid!("ui.menu.sound"), Value::Button),
                e(sid!("ui.menu.access"), Value::Button),
            ],
            Page::Controls => vec![
                e(sid!("ui.setting.mouse"), pct(s.input.mouse_sensitivity, 0.2, 4.0)),
                e(sid!("ui.setting.stick"), pct(s.input.stick_sensitivity, 0.2, 4.0)),
                e(sid!("ui.setting.invert"), Value::Switch(s.input.invert_y)),
                e(sid!("ui.setting.aim_assist"), pct(s.input.aim_assist, 0.0, 1.0)),
                e(sid!("ui.menu.buttons"), Value::Button),
                e(sid!("ui.setting.reset_buttons"), Value::Button),
            ],
            Page::Buttons => REMAPPABLE
                .iter()
                .map(|&a| {
                    let key = if self.capturing == Some(a) { None } else { prompts::prompt_key(a, device, style, bindings) };
                    e(action_label(a), Value::Binding(key))
                })
                .collect(),
            Page::Display => {
                let preset = s.preset.as_deref().unwrap_or("medium");
                vec![e(sid!("ui.setting.quality"), Value::Choice(StringId::from_key(&format!("ui.preset.{preset}")))), e(sid!("ui.setting.interface_scale"), pct(s.interface_scale(), 0.75, 1.5))]
            }
            Page::Sound => vec![
                e(sid!("ui.setting.master"), pct(s.master_volume, 0.0, 1.0)),
                e(sid!("ui.setting.music"), pct(s.music_volume, 0.0, 1.0)),
                e(sid!("ui.setting.effects"), pct(s.effects_volume, 0.0, 1.0)),
                e(sid!("ui.setting.ambience"), pct(s.ambience_volume, 0.0, 1.0)),
            ],
            Page::Access => vec![
                e(sid!("ui.setting.reduced_motion"), Value::Switch(s.reduced_motion)),
                e(sid!("ui.setting.reduced_screen_response"), Value::Switch(s.reduced_screen_response)),
                e(sid!("ui.setting.fewer_flashes"), Value::Switch(s.fewer_flashes)),
                e(sid!("ui.setting.captions"), Value::Switch(s.captions)),
                e(sid!("ui.setting.close_reading_slows"), Value::Switch(s.close_reading_slows)),
                e(sid!("ui.setting.creature_states"), Value::Switch(s.creature_states)),
                e(sid!("ui.setting.high_contrast"), Value::Switch(s.high_contrast)),
                e(sid!("ui.setting.letter_size"), pct(s.letter_size, 1.0, 1.5)),
                e(sid!("ui.setting.read_toggle"), Value::Switch(s.read_toggle)),
                e(sid!("ui.setting.safe_colours"), Value::Switch(s.safe_colours)),
            ],
        }
    }

    /// The prompts along the foot of a page, as string keys: confirm, then back.
    pub fn hint_keys(device: DeviceKind, style: PadStyle) -> (String, String) {
        match device {
            DeviceKind::KeyboardMouse => ("prompt.key.enter".into(), "prompt.key.escape".into()),
            DeviceKind::Pad => {
                let s = prompts::style_name(style);
                (format!("prompt.pad.{s}.south"), format!("prompt.pad.{s}.east"))
            }
        }
    }

    /// A button pressed for the action waiting for one. Escape or Start gives up.
    pub fn capture(&mut self, raw: &[RawInput], s: &mut Settings) -> Outcome {
        let Some(action) = self.capturing else { return Outcome::None };
        for r in raw {
            match *r {
                RawInput::Key { key: Key::Escape, down: true } | RawInput::PadButton { button: PadButton::Start, down: true, .. } => {
                    self.capturing = None;
                    return Outcome::None;
                }
                RawInput::Key { key, down: true } => {
                    s.input.remap_key(action, key);
                    self.capturing = None;
                    return Outcome::Changed;
                }
                // Movement is moved only to keys: a stick moves as it is.
                RawInput::MouseButton { .. } | RawInput::PadButton { .. } if matches!(action, Action::MoveForward | Action::MoveBack | Action::MoveLeft | Action::MoveRight) => {}
                RawInput::MouseButton { button, down: true } => {
                    s.input.remap_mouse(action, button);
                    self.capturing = None;
                    return Outcome::Changed;
                }
                RawInput::PadButton { button, down: true, .. } => {
                    s.input.remap_pad(action, button);
                    self.capturing = None;
                    return Outcome::Changed;
                }
                _ => {}
            }
        }
        Outcome::None
    }

    /// Acts on a frame's menu input. `rows` is how many rows the page shows at once.
    pub fn update(&mut self, input: MenuInput, s: &mut Settings, len: usize, rows: usize) -> Outcome {
        if self.capturing.is_some() {
            return Outcome::None;
        }
        let page = self.page();
        let Some((_, focus)) = self.stack.last_mut() else { return Outcome::Resume };
        if let Some(k) = input.pick.filter(|k| *k < len) {
            focus.at = k;
        }
        focus.step(input.step, len, rows);
        let at = focus.at;
        if input.back {
            self.stack.pop();
            return if self.stack.is_empty() { Outcome::Resume } else { Outcome::None };
        }
        let open = |stack: &mut Vec<(Page, Focus)>, p: Page| stack.push((p, Focus::default()));
        let change = input.change as f32;
        let nudge = |v: &mut f32, step: f32, min: f32, max: f32| {
            let was = *v;
            *v = ((*v + change * step) / step).round() * step;
            *v = v.clamp(min, max);
            *v != was
        };
        let changed = match (page, at) {
            (Page::Pause, k) if input.confirm => {
                return match k {
                    0 => Outcome::Resume,
                    1 => {
                        open(&mut self.stack, Page::Settings);
                        Outcome::None
                    }
                    2 => Outcome::Photo,
                    3 => Outcome::Save,
                    4 => Outcome::Load,
                    5 => Outcome::Credits,
                    6 => Outcome::StartAgain,
                    _ => Outcome::Quit,
                };
            }
            (Page::Settings, k) if input.confirm => {
                open(&mut self.stack, [Page::Controls, Page::Display, Page::Sound, Page::Access][k.min(3)]);
                false
            }
            (Page::Controls, 0) => nudge(&mut s.input.mouse_sensitivity, 0.1, 0.2, 4.0),
            (Page::Controls, 1) => nudge(&mut s.input.stick_sensitivity, 0.1, 0.2, 4.0),
            (Page::Controls, 2) if input.confirm || input.change != 0 => {
                s.input.invert_y = !s.input.invert_y;
                true
            }
            (Page::Controls, 3) => nudge(&mut s.input.aim_assist, 0.1, 0.0, 1.0),
            (Page::Controls, 4) if input.confirm => {
                open(&mut self.stack, Page::Buttons);
                false
            }
            (Page::Controls, 5) if input.confirm => {
                s.input.keys.clear();
                s.input.mouse.clear();
                s.input.pad.clear();
                true
            }
            (Page::Buttons, k) if input.confirm => {
                self.capturing = REMAPPABLE.get(k).copied();
                false
            }
            (Page::Display, 0) if input.confirm || input.change != 0 => {
                let now = PRESETS.iter().position(|p| Some(*p) == s.preset.as_deref()).unwrap_or(1) as i32;
                let by = if input.change != 0 { input.change } else { 1 };
                s.preset = Some(PRESETS[(now + by).rem_euclid(3) as usize].into());
                true
            }
            (Page::Display, 1) => {
                let mut v = s.interface_scale();
                let changed = nudge(&mut v, 0.05, 0.75, 1.5);
                s.interface_scale = v;
                changed
            }
            (Page::Sound, 0) => nudge(&mut s.master_volume, 0.1, 0.0, 1.0),
            (Page::Sound, 1) => nudge(&mut s.music_volume, 0.1, 0.0, 1.0),
            (Page::Sound, 2) => nudge(&mut s.effects_volume, 0.1, 0.0, 1.0),
            (Page::Sound, 3) => nudge(&mut s.ambience_volume, 0.1, 0.0, 1.0),
            (Page::Access, 7) => {
                let mut v = s.letter_size.clamp(1.0, 1.5);
                let changed = nudge(&mut v, 0.05, 1.0, 1.5);
                s.letter_size = v;
                changed
            }
            (Page::Access, k) if input.confirm || input.change != 0 => {
                let flag = match k {
                    0 => &mut s.reduced_motion,
                    1 => &mut s.reduced_screen_response,
                    2 => &mut s.fewer_flashes,
                    3 => &mut s.captions,
                    4 => &mut s.close_reading_slows,
                    5 => &mut s.creature_states,
                    6 => &mut s.high_contrast,
                    8 => &mut s.read_toggle,
                    _ => &mut s.safe_colours,
                };
                *flag = !*flag;
                true
            }
            _ => false,
        };
        if changed {
            Outcome::Changed
        } else {
            Outcome::None
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn press(menu: &mut Menu, s: &mut Settings, input: MenuInput) -> Outcome {
        let len = menu.entries(s, &Bindings::default(), DeviceKind::KeyboardMouse, PadStyle::Xbox).len();
        menu.update(input, s, len, 8)
    }

    const DOWN: MenuInput = MenuInput { step: 1, change: 0, confirm: false, back: false, pick: None };
    const RIGHT: MenuInput = MenuInput { step: 0, change: 1, confirm: false, back: false, pick: None };
    const OK: MenuInput = MenuInput { step: 0, change: 0, confirm: true, back: false, pick: None };
    const BACK: MenuInput = MenuInput { step: 0, change: 0, confirm: false, back: true, pick: None };

    #[test]
    fn the_pages_open_change_settings_and_close() {
        let mut s = Settings::default();
        let mut m = Menu::open();
        assert_eq!(press(&mut m, &mut s, OK), Outcome::Resume);
        press(&mut m, &mut s, DOWN);
        assert_eq!(press(&mut m, &mut s, OK), Outcome::None);
        assert_eq!(m.page(), Page::Settings);
        press(&mut m, &mut s, DOWN);
        press(&mut m, &mut s, DOWN);
        press(&mut m, &mut s, OK);
        assert_eq!(m.page(), Page::Sound);
        assert_eq!(press(&mut m, &mut s, MenuInput { change: -1, ..Default::default() }), Outcome::Changed);
        assert!((s.master_volume - 0.9).abs() < 1e-5);
        assert_eq!(press(&mut m, &mut s, RIGHT), Outcome::Changed);
        assert_eq!(press(&mut m, &mut s, RIGHT), Outcome::None, "already at the top");
        press(&mut m, &mut s, BACK);
        press(&mut m, &mut s, BACK);
        assert_eq!(m.page(), Page::Pause);
        assert_eq!(press(&mut m, &mut s, BACK), Outcome::Resume);
    }

    #[test]
    fn a_button_is_moved_by_pressing_it_and_escape_gives_up() {
        let mut s = Settings::default();
        let mut m = Menu { stack: vec![(Page::Buttons, Focus::default())], capturing: None, repeat: Default::default() };
        press(&mut m, &mut s, DOWN);
        press(&mut m, &mut s, DOWN);
        press(&mut m, &mut s, OK);
        assert!(m.capturing());
        assert_eq!(m.capture(&[RawInput::Key { key: Key::Escape, down: true }], &mut s), Outcome::None);
        assert!(!m.capturing());
        press(&mut m, &mut s, OK);
        assert_eq!(m.capture(&[RawInput::Key { key: Key::G, down: true }], &mut s), Outcome::Changed);
        assert_eq!(s.input.bindings().key_for(Action::Interact), Some(Key::G));
        // E, which Interact had, went to whatever had G: nothing, so nothing lost.
        let entries = m.entries(&s, &s.input.bindings(), DeviceKind::KeyboardMouse, PadStyle::Xbox);
        assert_eq!(entries[2].value, Value::Binding(Some("prompt.key.g".into())));
    }

    #[test]
    fn the_quality_choice_cycles_through_the_presets() {
        let mut s = Settings::default();
        let mut m = Menu { stack: vec![(Page::Display, Focus::default())], capturing: None, repeat: Default::default() };
        assert_eq!(press(&mut m, &mut s, OK), Outcome::Changed);
        assert_eq!(s.preset.as_deref(), Some("high"));
        press(&mut m, &mut s, RIGHT);
        assert_eq!(s.preset.as_deref(), Some("low"));
    }
}
