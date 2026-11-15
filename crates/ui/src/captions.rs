//! Captions for the sounds that matter: each word's motif, named by its word, and the
//! sounds a player must not miss, shown low on the screen for a few seconds, newest last,
//! the same sound heard again shown once.

use vellmark_content::{Face, GlyphAtlas, StringId, StringTable};

use crate::{Canvas, Part, Shown, TextStyle};

/// Seconds a caption stays, and the most shown at once.
const STAY: f32 = 3.0;
const MOST: usize = 3;

const WHITE: [f32; 4] = [0.95, 0.93, 0.88, 1.0];
const BACKING: [f32; 4] = [0.0, 0.0, 0.0, 0.55];

#[derive(Default)]
pub struct Captions {
    /// Each caption's string, the word it names if any, and its seconds left.
    lines: Vec<(StringId, Option<StringId>, f32)>,
}

impl Captions {
    /// A sound heard: shown anew, or kept longer if it is already shown.
    pub fn heard(&mut self, caption: StringId, word: Option<StringId>) {
        self.lines.retain(|(c, w, _)| !(*c == caption && *w == word));
        self.lines.push((caption, word, STAY));
        if self.lines.len() > MOST {
            self.lines.remove(0);
        }
    }

    pub fn update(&mut self, dt: f32) {
        for line in &mut self.lines {
            line.2 -= dt;
        }
        self.lines.retain(|l| l.2 > 0.0);
    }

    pub fn len(&self) -> usize {
        self.lines.len()
    }

    pub fn is_empty(&self) -> bool {
        self.lines.is_empty()
    }

    /// Drawn low and centred, above where the HUD's bars sit.
    pub fn draw(&self, canvas: &mut Canvas, atlas: &GlyphAtlas, strings: &StringTable, size: (u32, u32), density: f32) {
        let (w, h) = (size.0 as f32, size.1 as f32);
        let d = density.max(1.0);
        let style = TextStyle { face: Face::Large, size: (18.0 * d).round(), colour: WHITE };
        let line = Canvas::line_height(atlas, &style) * 1.2;
        let bottom = h * 0.80;
        for (k, (caption, word, left)) in self.lines.iter().rev().enumerate() {
            let fade = (left / 0.5).min(1.0);
            let text = match word {
                Some(word) => Shown::filled(strings, *caption, &[("word", Part::Shown(Shown::from_table(strings, *word)))]),
                None => Shown::from_table(strings, *caption),
            };
            let tw = Canvas::measure(atlas, &style, &text);
            let y = bottom - k as f32 * line;
            canvas.rect([(w - tw) * 0.5 - 8.0 * d, y - line * 0.8, tw + 16.0 * d, line], [BACKING[0], BACKING[1], BACKING[2], BACKING[3] * fade]);
            canvas.text(atlas, (w - tw) * 0.5, y, &TextStyle { colour: [WHITE[0], WHITE[1], WHITE[2], fade], ..style }, &text);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn captions_stay_a_while_fold_repeats_and_keep_the_newest() {
        let mut c = Captions::default();
        let (a, b) = (StringId(1), StringId(2));
        c.heard(a, None);
        c.heard(a, None);
        assert_eq!(c.len(), 1, "the same sound again is shown once");
        c.heard(b, Some(StringId(9)));
        c.heard(b, Some(StringId(8)));
        c.heard(a, Some(StringId(7)));
        assert_eq!(c.len(), 3);
        c.update(STAY + 0.1);
        assert!(c.is_empty());
    }
}
