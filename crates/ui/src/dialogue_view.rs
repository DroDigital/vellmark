//! A conversation as the player sees it: the speaker's name, the line being said written
//! out quickly as if by hand, and after a node's last line the choices, with the focus.
//! Before one begins, the crosshair offers to talk to whoever is in reach.

use vellmark_content::{sid, Face, GlyphAtlas, StringId, StringTable};
use vellmark_sim::TalkSnap;

use crate::kit::{self, Focus, Row};
use crate::{Canvas, Part, Shown, TextStyle};

/// Characters written each second as a line comes.
pub const WRITTEN_PER_SECOND: f64 = 70.0;

const SHADOW: [f32; 4] = [0.0, 0.0, 0.0, 0.6];

pub struct DialogueView<'a> {
    pub atlas: &'a GlyphAtlas,
    pub strings: &'a StringTable,
    pub size: (u32, u32),
    pub density: f32,
}

/// Whether the line is written out in full by now.
pub fn written(strings: &StringTable, talk: &TalkSnap, now: f64) -> bool {
    talk.line.is_none_or(|l| ((now - talk.since) * WRITTEN_PER_SECOND) as usize >= strings.text(StringId(l)).chars().count())
}

impl DialogueView<'_> {
    fn choice_line(&self) -> f32 {
        let style = TextStyle { face: Face::Large, size: (19.0 * self.density.max(1.0)).round(), colour: kit::ink() };
        Canvas::line_height(self.atlas, &style) * 1.15
    }

    /// The rows of choices the panel shows at once.
    pub fn rows(&self) -> usize {
        ((self.size.1 as f32 * 0.3 / self.choice_line()).floor() as usize).max(1)
    }

    /// The panel, sized to hold the whole line and the choices, so that it does not grow
    /// as the line is written.
    fn panel(&self, talk: &TalkSnap, body: &TextStyle) -> [f32; 4] {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let d = self.density.max(1.0);
        let pw = (980.0 * d).min(w * 0.86);
        let pad = 32.0 * d;
        let mut inner = 30.0 * d + 28.0 * d;
        if let Some(line) = talk.line {
            inner += Canvas::new().paragraph(self.atlas, 0.0, 0.0, pw - pad * 2.0, body, &Shown::from_table(self.strings, StringId(line))) + 14.0 * d;
        }
        inner += talk.choices.len().min(self.rows()) as f32 * self.choice_line();
        let ph = (inner + pad * 2.0).clamp(150.0 * d, h * 0.5);
        [(w - pw) * 0.5, h - ph - 28.0 * d, pw, ph]
    }

    /// `now` is the world's seconds; `focus` the choice the player rests on; `key` the
    /// prompt key for Interact, or for confirming on a pad.
    pub fn draw(&self, canvas: &mut Canvas, talk: &TalkSnap, now: f64, focus: Focus, key: Option<&str>) {
        let d = self.density.max(1.0);
        let body = TextStyle { face: Face::Large, size: (21.0 * d).round(), colour: kit::ink() };
        let [x, y, w, h] = self.panel(talk, &body);
        kit::panel(canvas, self.atlas, self.strings, [x, y, w, h], d);
        let pad = 32.0 * d;
        let name = TextStyle { face: Face::Large, size: (22.0 * d).round(), colour: kit::gold() };
        let speaker = Shown::from_table(self.strings, StringId::from_key(&format!("speaker.{}", talk.by)));
        canvas.text(self.atlas, x + pad, y + pad + 14.0 * d, &name, &speaker);
        let mut top = y + pad + 30.0 * d;
        if let Some(line) = talk.line {
            let text = Shown::from_table(self.strings, StringId(line));
            let shown = ((now - talk.since).max(0.0) * WRITTEN_PER_SECOND) as usize;
            let part = if talk.choices.is_empty() { text.prefix(shown) } else { text.prefix(usize::MAX) };
            top += canvas.paragraph(self.atlas, x + pad, top, w - pad * 2.0, &body, &part) + 14.0 * d;
        }
        if !talk.choices.is_empty() {
            let rows: Vec<Row<'_>> = talk.choices.iter().map(|c| Row { text: Shown::from_table(self.strings, StringId(*c)), marked: false, colour: kit::ink() }).collect();
            kit::list(canvas, self.atlas, self.strings, [x + pad, top, w - pad * 2.0, y + h - pad - top - 10.0 * d], d, &rows, focus, 19.0);
        }
        // The prompt, at the panel's foot, once the line is written.
        if let Some(key) = key.filter(|_| !talk.choices.is_empty() || written(self.strings, talk, now)) {
            let small = TextStyle { face: Face::Small, size: (15.0 * d).round(), colour: kit::faint() };
            let verb = if talk.choices.is_empty() { sid!("ui.talk.continue") } else { sid!("ui.talk.choose") };
            let text = Shown::filled(self.strings, verb, &[("key", Part::Shown(Shown::from_table(self.strings, StringId::from_key(key))))]);
            let tw = Canvas::measure(self.atlas, &small, &text);
            canvas.text(self.atlas, x + w - pad - tw, y + h - pad * 0.6, &small, &text);
        }
    }

    /// The offer to talk, beneath the crosshair: `[E] Talk to Wick`.
    pub fn draw_offer(&self, canvas: &mut Canvas, speaker: &str, key: &str) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let d = self.density.max(1.0);
        let small = TextStyle { face: Face::Small, size: (18.0 * d).round(), colour: [0.93, 0.91, 0.85, 1.0] };
        let name = Shown::from_table(self.strings, StringId::from_key(&format!("speaker.{speaker}")));
        let text = Shown::filled(self.strings, sid!("ui.talk.offer"), &[("key", Part::Shown(Shown::from_table(self.strings, StringId::from_key(key)))), ("name", Part::Shown(name))]);
        let tw = Canvas::measure(self.atlas, &small, &text);
        let (tx, ty) = ((w - tw) * 0.5, h * 0.62);
        canvas.text(self.atlas, tx + 1.5 * d, ty + 1.5 * d, &TextStyle { colour: SHADOW, ..small }, &text);
        canvas.text(self.atlas, tx, ty, &small, &text);
    }
}
