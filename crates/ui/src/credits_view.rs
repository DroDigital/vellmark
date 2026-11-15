//! The credits: who made the game, the typefaces, and every crate it is built with and
//! its license, rolling slowly up a ruled page, with where the licenses' full texts are.

use vellmark_content::{sid, Credits, Face, GlyphAtlas, StringId, StringTable};

use crate::kit;
use crate::{Canvas, Shown, TextStyle};

/// Pixels a second the credits roll, before the interface's density.
pub const ROLL: f32 = 28.0;

pub struct CreditsView<'a> {
    pub atlas: &'a GlyphAtlas,
    pub strings: &'a StringTable,
    pub credits: &'a Credits,
    pub size: (u32, u32),
    pub density: f32,
}

enum Line<'a> {
    Heading(StringId),
    Text(Shown<'a>),
    Gap,
}

impl CreditsView<'_> {
    fn lines(&self) -> Vec<Line<'_>> {
        let mut out = vec![Line::Heading(sid!("ui.credits.title")), Line::Gap];
        if !self.credits.made_by.is_empty() {
            out.push(Line::Heading(sid!("ui.credits.made_by")));
            out.extend(self.credits.made_by.iter().map(|m| Line::Text(Shown::from_data(m))));
            out.push(Line::Gap);
        }
        out.push(Line::Heading(sid!("ui.credits.typefaces")));
        for (name, author, license) in &self.credits.typefaces {
            out.push(Line::Text(Shown::from_data(name)));
            out.push(Line::Text(Shown::from_data(author)));
            out.push(Line::Text(Shown::from_data(license)));
            out.push(Line::Gap);
        }
        out.push(Line::Heading(sid!("ui.credits.built_with")));
        for (name, license) in &self.credits.crates {
            out.push(Line::Text(Shown::from_data(name)));
            out.push(Line::Text(Shown::from_data(license)));
        }
        out.push(Line::Gap);
        out.push(Line::Heading(sid!("ui.credits.notices")));
        out
    }

    /// The height of everything, for knowing when the roll is done.
    pub fn height(&self) -> f32 {
        let d = self.density.max(1.0);
        let line = Canvas::line_height(self.atlas, &TextStyle { face: Face::Large, size: (17.0 * d).round(), colour: kit::ink() });
        self.lines().len() as f32 * line
    }

    /// `scroll` is how far the roll has come, in pixels.
    pub fn draw(&self, canvas: &mut Canvas, scroll: f32) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let d = self.density.max(1.0);
        canvas.rect([0.0, 0.0, w, h], [0.0, 0.0, 0.0, 0.85]);
        let pw = (640.0 * d).min(w * 0.9);
        let panel = [(w - pw) * 0.5, h * 0.06, pw, h * 0.88];
        kit::panel(canvas, self.atlas, self.strings, panel, d);
        let body = TextStyle { face: Face::Large, size: (17.0 * d).round(), colour: kit::ink() };
        let heading = TextStyle { face: Face::Large, size: (21.0 * d).round(), colour: kit::gold() };
        let line = Canvas::line_height(self.atlas, &body);
        let (top, bottom) = (panel[1] + 30.0 * d, panel[1] + panel[3] - 20.0 * d);
        let mut y = bottom - scroll;
        for l in self.lines() {
            y += line;
            if y < top || y > bottom {
                continue;
            }
            let (style, text) = match l {
                Line::Heading(id) => (heading, Shown::from_table(self.strings, id)),
                Line::Text(t) => (body, t),
                Line::Gap => continue,
            };
            let tw = Canvas::measure(self.atlas, &style, &text);
            canvas.text(self.atlas, panel[0] + (panel[2] - tw).max(0.0) * 0.5, y, &style, &text);
        }
    }
}
