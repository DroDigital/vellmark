//! What reading shows: a prompt when a text is in focus, and reading mode, the text set
//! large on paper over the dimmed view, with the hidden lines the reader sees beneath it
//! in a different ink. Close reading is drawn by the renderer itself.

use vellmark_content::{sid, GlyphAtlas, StringId, StringTable};
use vellmark_platform::{prompts, ActionState, Bindings};
use vellmark_sim::reading::ReadingSnap;
use vellmark_sim::PlayerSnap;
use crate::{Canvas, Shown, TextStyle};

const PAPER: [f32; 4] = [0.86, 0.82, 0.72, 0.97];
const INK: [f32; 4] = [0.11, 0.09, 0.07, 1.0];
const HIDDEN_INK: [f32; 4] = [0.45, 0.12, 0.08, 1.0];
const DIM: [f32; 4] = [0.0, 0.0, 0.0, 0.55];
const PROMPT: [f32; 4] = [0.93, 0.91, 0.85, 1.0];
const SHADOW: [f32; 4] = [0.0, 0.0, 0.0, 0.6];

pub struct ReadingView<'a> {
    pub atlas: &'a GlyphAtlas,
    pub strings: &'a StringTable,
    pub size: (u32, u32),
    pub density: f32,
}

impl ReadingView<'_> {
    pub fn draw(&self, canvas: &mut Canvas, player: &PlayerSnap, actions: &ActionState, bindings: &Bindings, style: prompts::PadStyle) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let d = self.density.max(1.0);
        match &player.reading {
            ReadingSnap::Text { text, hidden, .. } => {
                canvas.rect([0.0, 0.0, w, h], DIM);
                let width = (760.0 * d).min(w * 0.82);
                let x = (w - width) * 0.5;
                let pad = 34.0 * d;
                // As large as fits on the screen: set off screen to learn the height, a
                // size smaller each time, then on the paper.
                let fits = |size: f32| {
                    let body = TextStyle { face: vellmark_content::Face::Large, size, colour: INK };
                    let mut measure = Canvas::new();
                    let mut height = measure.paragraph(self.atlas, 0.0, 0.0, width, &body, &Shown::from_table(self.strings, StringId(*text)));
                    for line in hidden {
                        height += measure.paragraph(self.atlas, 0.0, 0.0, width, &body, &Shown::from_table(self.strings, StringId(*line))) + size * 0.6;
                    }
                    (body, height)
                };
                let mut size = (30.0 * d).round();
                let (mut body, mut height) = fits(size);
                while height + pad * 2.0 > h * 0.92 && size > 16.0 * d {
                    size -= 2.0 * d;
                    (body, height) = fits(size);
                }
                let hidden_style = TextStyle { colour: HIDDEN_INK, ..body };
                let top = ((h - height) * 0.5 - pad).max(pad);
                canvas.rect([x - pad, top, width + pad * 2.0, height + pad * 2.0], PAPER);
                let mut y = top + pad;
                y += canvas.paragraph(self.atlas, x, y, width, &body, &Shown::from_table(self.strings, StringId(*text)));
                for line in hidden {
                    y += body.size * 0.6;
                    y += canvas.paragraph(self.atlas, x, y, width, &hidden_style, &Shown::from_table(self.strings, StringId(*line)));
                }
            }
            ReadingSnap::None if player.in_focus.is_some() => {
                let Some(key) = prompts::prompt_key(vellmark_platform::Action::Read, actions.last_device, style, bindings) else { return };
                let small = TextStyle { face: vellmark_content::Face::Small, size: (18.0 * d).round(), colour: PROMPT };
                let button = Shown::from_table(self.strings, vellmark_content::StringId::from_key(&key));
                let verb = Shown::from_table(self.strings, sid!("ui.read"));
                let gap = small.size * 0.5;
                let total = Canvas::measure(self.atlas, &small, &button) + gap + Canvas::measure(self.atlas, &small, &verb);
                let x = (w - total) * 0.5;
                let y = h * 0.62;
                for (dx, colour) in [(1.5 * d, SHADOW), (0.0, PROMPT)] {
                    let s = TextStyle { colour, ..small };
                    let bw = canvas.text(self.atlas, x + dx, y + dx, &s, &button);
                    canvas.text(self.atlas, x + bw + gap + dx, y + dx, &s, &verb);
                }
            }
            _ => {}
        }
    }
}
