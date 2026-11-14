//! A menu page: a title and a list of entries, each a label and its value (a choice, a
//! switch, a slider drawn in strokes, a button for an action), on a ruled panel from the
//! interface kit. Menus read their own fixed keys as well as the player's bindings, so
//! that no remapping can lock a player out of the menu that undoes it.

use vellmark_content::{sid, Face, GlyphAtlas, StringId, StringTable};
use vellmark_platform::{ActionState, Key, PadButton, RawInput};

use crate::kit::{self, Focus, NavRepeat};
use crate::{Canvas, Part, Shown, TextStyle};

/// What an entry shows after its label.
#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    /// Opens a page or does a thing.
    Button,
    Switch(bool),
    /// A value between `min` and `max`, shown as strokes and as a whole number scaled by
    /// `shown` (100 for a percentage).
    Slider { value: f32, min: f32, max: f32, shown: f32 },
    /// One of several, by string key.
    Choice(StringId),
    /// A button prompt's key, or none while it waits for a press.
    Binding(Option<String>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    pub label: StringId,
    pub value: Value,
}

/// A frame's menu input: one step of focus, one step of value, confirm, and back.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct MenuInput {
    pub step: i32,
    pub change: i32,
    pub confirm: bool,
    pub back: bool,
}

/// The repeats of a held direction, up and down and across.
#[derive(Clone, Copy, Debug, Default)]
pub struct MenuRepeat {
    vertical: NavRepeat,
    across: NavRepeat,
}

impl MenuInput {
    /// From this frame's raw input and the movement axes: arrows, WASD, Enter, Space, and
    /// Escape on a keyboard; the pad's D-pad, its left stick, South to confirm, and East
    /// to go back.
    pub fn read(raw: &[RawInput], actions: &ActionState, repeat: &mut MenuRepeat, dt: f32) -> Self {
        let mut m = MenuInput::default();
        for r in raw {
            match *r {
                RawInput::Key { key, down: true } => match key {
                    Key::Up => m.step = -1,
                    Key::Down => m.step = 1,
                    Key::Left => m.change = -1,
                    Key::Right => m.change = 1,
                    Key::Enter | Key::Space | Key::E => m.confirm = true,
                    Key::Escape | Key::Backspace => m.back = true,
                    _ => {}
                },
                RawInput::PadButton { button, down: true, .. } => match button {
                    PadButton::DPadUp => m.step = -1,
                    PadButton::DPadDown => m.step = 1,
                    PadButton::DPadLeft => m.change = -1,
                    PadButton::DPadRight => m.change = 1,
                    PadButton::South => m.confirm = true,
                    PadButton::East | PadButton::Start => m.back = true,
                    _ => {}
                },
                _ => {}
            }
        }
        let (x, y) = actions.movement();
        let v = repeat.vertical.step(y, dt);
        let a = -repeat.across.step(x, dt);
        if m.step == 0 {
            m.step = v;
        }
        if m.change == 0 {
            m.change = a;
        }
        m
    }
}

pub struct MenuView<'a> {
    pub atlas: &'a GlyphAtlas,
    pub strings: &'a StringTable,
    pub size: (u32, u32),
    pub density: f32,
}

const ROW: f32 = 19.0;

impl MenuView<'_> {
    fn line(&self) -> f32 {
        let style = TextStyle { face: Face::Large, size: (ROW * self.density.max(1.0)).round(), colour: kit::INK };
        Canvas::line_height(self.atlas, &style) * 1.15
    }

    /// The panel for a page of `rows` entries, and the list inside it.
    fn layout(&self, rows: usize) -> ([f32; 4], [f32; 4]) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let d = self.density.max(1.0);
        let pad = 34.0 * d;
        let pw = (620.0 * d).min(w * 0.9);
        let ph = (rows as f32 * self.line() + pad * 2.0 + 90.0 * d).min(h * 0.9);
        let panel = [(w - pw) * 0.5, (h - ph) * 0.5, pw, ph];
        (panel, [panel[0] + pad, panel[1] + pad + 46.0 * d, pw - pad * 2.0, ph - pad * 2.0 - 80.0 * d])
    }

    /// The rows a page can show at once.
    pub fn rows(&self, entries: usize, _atlas: &GlyphAtlas) -> usize {
        let (_, list) = self.layout(entries);
        ((list[3] / self.line()).floor() as usize).max(1)
    }

    /// Draws a page over a dimmed view. `hint` is a line along its foot.
    pub fn draw(&self, canvas: &mut Canvas, title: StringId, entries: &[Entry], focus: Focus, hint: Option<Shown<'_>>) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let d = self.density.max(1.0);
        canvas.rect([0.0, 0.0, w, h], [0.0, 0.0, 0.0, 0.5]);
        let (panel, list) = self.layout(entries.len());
        kit::panel(canvas, self.atlas, self.strings, panel, d);
        let pad = 34.0 * d;
        let heading = TextStyle { face: Face::Large, size: (26.0 * d).round(), colour: kit::INK };
        let title = Shown::from_table(self.strings, title);
        let tw = Canvas::measure(self.atlas, &heading, &title);
        canvas.text(self.atlas, panel[0] + (panel[2] - tw) * 0.5, panel[1] + pad + 20.0 * d, &heading, &title);
        let values: Vec<Shown<'_>> = entries.iter().map(|e| self.value(&e.value)).collect();
        let rows: Vec<kit::Row<'_>> = entries.iter().map(|e| kit::Row { text: Shown::from_table(self.strings, e.label), marked: false, colour: kit::INK }).collect();
        kit::list(canvas, self.atlas, self.strings, list, d, &rows, focus, ROW);
        // Each value, right-aligned on its row.
        let style = TextStyle { face: Face::Large, size: (ROW * d).round(), colour: kit::INK };
        let line = Canvas::line_height(self.atlas, &style) * 1.15;
        let shown = ((list[3] / line).floor() as usize).max(1);
        for (k, value) in values.iter().enumerate().skip(focus.scroll).take(shown) {
            let top = list[1] + (k - focus.scroll) as f32 * line + line * 0.8;
            let colour = if k == focus.at { kit::GOLD } else { kit::INK };
            let vw = Canvas::measure(self.atlas, &style, value);
            canvas.text(self.atlas, list[0] + list[2] - vw, top, &TextStyle { colour, ..style }, value);
        }
        if let Some(hint) = hint {
            let small = TextStyle { face: Face::Small, size: (15.0 * d).round(), colour: kit::FAINT };
            let hw = Canvas::measure(self.atlas, &small, &hint);
            canvas.text(self.atlas, panel[0] + (panel[2] - hw) * 0.5, panel[1] + panel[3] - pad, &small, &hint);
        }
    }

    fn value(&self, value: &Value) -> Shown<'_> {
        match value {
            Value::Button => Shown::from_table(self.strings, sid!("ui.menu.open")),
            Value::Switch(on) => Shown::from_table(self.strings, if *on { sid!("ui.menu.on") } else { sid!("ui.menu.off") }),
            Value::Choice(id) => Shown::from_table(self.strings, *id),
            Value::Binding(Some(key)) => Shown::from_table(self.strings, StringId::from_key(key)),
            Value::Binding(None) => Shown::from_table(self.strings, sid!("ui.menu.press")),
            Value::Slider { value, min, max, shown } => {
                const STROKES: usize = 12;
                let t = ((value - min) / (max - min)).clamp(0.0, 1.0);
                let full = (t * STROKES as f32).round() as usize;
                let stroke = Shown::from_table(self.strings, sid!("ui.slider.full"));
                let empty = Shown::from_table(self.strings, sid!("ui.slider.empty"));
                Shown::filled(
                    self.strings,
                    sid!("ui.slider.bar"),
                    &[("full", Part::Repeated(stroke, full)), ("empty", Part::Repeated(empty, STROKES - full)), ("n", Part::Number((value * shown).round() as u32))],
                )
            }
        }
    }
}
