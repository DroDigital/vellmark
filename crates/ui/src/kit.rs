//! The interface kit: panels framed in letters, lists with a focus that moves and
//! scrolls, and menu navigation from the same actions that move and act in the world.
//! Every screen is built from these, so that every screen reads the same.

use vellmark_content::{Face, GlyphAtlas, StringTable};

use crate::{Canvas, Shown, TextStyle};

pub const PAPER: [f32; 4] = [0.05, 0.045, 0.04, 0.9];
pub const INK: [f32; 4] = [0.9, 0.87, 0.8, 1.0];
pub const FAINT: [f32; 4] = [0.9, 0.87, 0.8, 0.45];
pub const GOLD: [f32; 4] = [0.98, 0.78, 0.30, 1.0];
pub const RULE: [f32; 4] = [0.9, 0.87, 0.8, 0.6];

/// A panel: a dark field framed with a rule of characters, as a ledger's page is ruled.
pub fn panel(canvas: &mut Canvas, atlas: &GlyphAtlas, strings: &StringTable, rect: [f32; 4], density: f32) {
    let [x, y, w, h] = rect;
    canvas.rect(rect, PAPER);
    let style = TextStyle { face: Face::Small, size: (12.0 * density).round(), colour: RULE };
    let across = Shown::from_table(strings, vellmark_content::sid!("ui.rule.across"));
    let down = Shown::from_table(strings, vellmark_content::sid!("ui.rule.down"));
    let corner = Shown::from_table(strings, vellmark_content::sid!("ui.rule.corner"));
    let step = Canvas::measure(atlas, &style, &across).max(1.0);
    let line = Canvas::line_height(atlas, &style).max(1.0);
    let mut cx = x + step;
    while cx < x + w - step * 1.5 {
        canvas.text(atlas, cx, y + line * 0.8, &style, &across);
        canvas.text(atlas, cx, y + h - line * 0.2, &style, &across);
        cx += step;
    }
    let mut cy = y + line * 1.8;
    while cy < y + h - line * 0.8 {
        canvas.text(atlas, x, cy, &style, &down);
        canvas.text(atlas, x + w - step, cy, &style, &down);
        cy += line;
    }
    for (px, py) in [(x, y + line * 0.8), (x + w - step, y + line * 0.8), (x, y + h - line * 0.2), (x + w - step, y + h - line * 0.2)] {
        canvas.text(atlas, px, py, &style, &corner);
    }
}

/// A list's focus and how far it has scrolled.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Focus {
    pub at: usize,
    pub scroll: usize,
}

impl Focus {
    /// Moves the focus by `by`, kept within `len`, and scrolls so that it stays among the
    /// `rows` shown.
    pub fn step(&mut self, by: i32, len: usize, rows: usize) {
        if len == 0 {
            *self = Focus::default();
            return;
        }
        self.at = (self.at as i32 + by).clamp(0, len as i32 - 1) as usize;
        if self.at < self.scroll {
            self.scroll = self.at;
        }
        if self.at >= self.scroll + rows {
            self.scroll = self.at + 1 - rows;
        }
    }
}

/// One row of a list: its text, and whether it is marked (held, worn, chosen).
pub struct Row<'a> {
    pub text: Shown<'a>,
    pub marked: bool,
    pub colour: [f32; 4],
}

/// Draws a list in a panel's inside, with the focused row drawn bright behind a mark.
/// Returns the number of rows that fit.
#[allow(clippy::too_many_arguments)]
pub fn list(canvas: &mut Canvas, atlas: &GlyphAtlas, strings: &StringTable, rect: [f32; 4], density: f32, rows: &[Row<'_>], focus: Focus, size: f32) -> usize {
    let [x, y, w, h] = rect;
    let style = TextStyle { face: Face::Large, size: (size * density).round(), colour: INK };
    let line = Canvas::line_height(atlas, &style) * 1.15;
    let shown = ((h / line).floor() as usize).max(1);
    let pointer = Shown::from_table(strings, vellmark_content::sid!("ui.focus"));
    let held = Shown::from_table(strings, vellmark_content::sid!("ui.held"));
    let indent = Canvas::measure(atlas, &style, &pointer) * 1.5;
    for (k, row) in rows.iter().enumerate().skip(focus.scroll).take(shown) {
        let top = y + (k - focus.scroll) as f32 * line + line * 0.8;
        let focused = k == focus.at;
        if focused {
            canvas.rect([x, top - line * 0.8, w, line], [1.0, 1.0, 1.0, 0.06]);
            canvas.text(atlas, x, top, &TextStyle { colour: GOLD, ..style }, &pointer);
        }
        let colour = if focused { row.colour } else { [row.colour[0], row.colour[1], row.colour[2], row.colour[3] * 0.8] };
        let room = w - indent - Canvas::measure(atlas, &style, &held) * 2.0;
        let (fitted, text) = fit(atlas, strings, TextStyle { colour, ..style }, &row.text, room, 13.0 * density);
        canvas.text(atlas, x + indent, top, &fitted, text.as_ref().unwrap_or(&row.text));
        if row.marked {
            let mw = Canvas::measure(atlas, &style, &held);
            canvas.text(atlas, x + w - mw, top, &TextStyle { colour: GOLD, ..style }, &held);
        }
    }
    shown
}

/// A line made to fit `room`: set smaller, down to `least`, and if it still will not fit,
/// clipped and ended with the table's ellipsis.
pub fn fit<'a>(atlas: &GlyphAtlas, strings: &'a StringTable, style: TextStyle, text: &Shown<'_>, room: f32, least: f32) -> (TextStyle, Option<Shown<'static>>) {
    let mut style = style;
    while Canvas::measure(atlas, &style, text) > room && style.size > least {
        style.size = (style.size - 1.0).max(least);
    }
    if Canvas::measure(atlas, &style, text) <= room {
        return (style, None);
    }
    let tail = Shown::from_table(strings, vellmark_content::sid!("ui.ellipsis"));
    let mut chars = text.as_str().chars().count();
    loop {
        chars = chars.saturating_sub(1);
        let clipped = text.clipped(chars, &tail);
        if chars == 0 || Canvas::measure(atlas, &style, &clipped) <= room {
            return (style, Some(clipped));
        }
    }
}

/// What a menu was asked to do this frame.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Nav {
    /// Up is -1, down 1.
    pub step: i32,
    pub confirm: bool,
    pub back: bool,
}

/// Menu navigation from the movement axis, with a repeat while it is held.
#[derive(Clone, Copy, Debug, Default)]
pub struct NavRepeat {
    held: i32,
    wait: f32,
}

impl NavRepeat {
    /// `axis` is the forward movement axis, up positive.
    pub fn step(&mut self, axis: f32, dt: f32) -> i32 {
        let dir = if axis > 0.5 { -1 } else if axis < -0.5 { 1 } else { 0 };
        if dir == 0 {
            self.held = 0;
            return 0;
        }
        if dir != self.held {
            self.held = dir;
            self.wait = 0.35;
            return dir;
        }
        self.wait -= dt;
        if self.wait <= 0.0 {
            self.wait = 0.09;
            return dir;
        }
        0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_focus_stays_in_the_list_and_on_screen() {
        let mut f = Focus::default();
        f.step(-1, 10, 4);
        assert_eq!(f, Focus { at: 0, scroll: 0 });
        for _ in 0..6 {
            f.step(1, 10, 4);
        }
        assert_eq!(f, Focus { at: 6, scroll: 3 });
        f.step(20, 10, 4);
        assert_eq!(f, Focus { at: 9, scroll: 6 });
        f.step(-8, 10, 4);
        assert_eq!(f, Focus { at: 1, scroll: 1 });
    }

    #[test]
    fn a_held_direction_steps_once_then_repeats() {
        let mut n = NavRepeat::default();
        let steps: Vec<i32> = (0..60).map(|_| n.step(-1.0, 1.0 / 60.0)).collect();
        assert_eq!(steps[0], 1);
        let count = steps.iter().filter(|s| **s != 0).count();
        // One at once, then after 0.35 s one every 0.09 s: eight in a second.
        assert!((7..=9).contains(&count), "{count}");
        assert_eq!(n.step(0.0, 0.016), 0);
    }
}
