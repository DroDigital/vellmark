//! What the player sees of their own hands and Ink, drawn in letters: a crosshair, the
//! plain preview beneath it of what the word in hand would do to the thing in the gaze, a
//! ring that fills as a charge reaches a tough target, the words in the hands, and Ink.
//! Class and resistance never appear on screen.

use vellmark_content::{sid, Face, GlyphAtlas, StringId, StringTable};
use vellmark_sim::casting::Holding;
use vellmark_sim::emendation::{Preview, WordRules, INK_MAX};
use vellmark_sim::PlayerSnap;

use crate::{Canvas, Shown, TextStyle};

const WHITE: [f32; 4] = [0.93, 0.91, 0.86, 0.95];
const REFUSED: [f32; 4] = [0.78, 0.74, 0.66, 0.9];
const GOLD: [f32; 4] = [0.98, 0.78, 0.30, 1.0];
const SHADOW: [f32; 4] = [0.0, 0.0, 0.0, 0.55];
const FAINT: [f32; 4] = [0.93, 0.91, 0.86, 0.35];

pub struct Hud<'a> {
    pub atlas: &'a GlyphAtlas,
    pub strings: &'a StringTable,
    pub rules: &'a WordRules,
    pub size: (u32, u32),
    pub density: f32,
}

impl Hud<'_> {
    fn shadowed(&self, canvas: &mut Canvas, x: f32, y: f32, style: &TextStyle, text: &Shown<'_>) -> f32 {
        let d = self.density.max(1.0);
        canvas.text(self.atlas, x + d, y + d, &TextStyle { colour: SHADOW, ..*style }, text);
        canvas.text(self.atlas, x, y, style, text)
    }

    fn centred(&self, canvas: &mut Canvas, cx: f32, y: f32, style: &TextStyle, text: &Shown<'_>) {
        let w = Canvas::measure(self.atlas, style, text);
        self.shadowed(canvas, cx - w * 0.5, y, style, text);
    }

    /// A word's name as the player sees it: its caption, `[OPEN]`.
    fn word_name(&self, word: usize) -> Shown<'_> {
        let key = format!("caption.word.{}", self.rules.words[word].spelling.to_ascii_lowercase());
        Shown::from_table(self.strings, StringId::from_key(&key))
    }

    pub fn draw(&self, canvas: &mut Canvas, player: &PlayerSnap) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let d = self.density.max(1.0);
        let (cx, cy) = (w * 0.5, h * 0.5);
        let small = TextStyle { face: Face::Small, size: (16.0 * d).round(), colour: WHITE };
        // The crosshair: a point, and a ring when a word is in hand.
        let words_in_hand = player.holding.iter().any(|h| matches!(h, Holding::Word(_)));
        let dot = TextStyle { size: (14.0 * d).round(), ..small };
        self.centred(canvas, cx, cy + dot.size * 0.35, &dot, &Shown::from_table(self.strings, sid!("ui.crosshair")));

        // The preview: the off hand's word first, as the hand that writes.
        let preview = player.preview[1].clone().or_else(|| player.preview[0].clone());
        if let Some((_, p)) = preview {
            let line_y = cy + 34.0 * d;
            let printed = TextStyle { face: Face::Large, size: (20.0 * d).round(), colour: WHITE };
            match p {
                Preview::Result(s) => self.centred(canvas, cx, line_y, &printed, &Shown::from_table(self.strings, StringId(s))),
                Preview::Refusal(s) => self.centred(canvas, cx, line_y, &TextStyle { colour: REFUSED, ..printed }, &Shown::from_table(self.strings, StringId(s))),
                Preview::Sealed => self.centred(canvas, cx, line_y, &TextStyle { colour: GOLD, ..printed }, &Shown::from_table(self.strings, sid!("ui.sealed"))),
                Preview::NeedsCharge(needed) => {
                    // A ring of twelve points about the crosshair, filling as the charge
                    // reaches what the target needs.
                    let charge = player.charge[1].max(player.charge[0]).max(player.charge[2]);
                    let filled = ((charge / needed.max(1.0e-3)).min(1.0) * 12.0).round() as usize;
                    let r = 22.0 * d;
                    for k in 0..12 {
                        let a = k as f32 / 12.0 * std::f32::consts::TAU - std::f32::consts::FRAC_PI_2;
                        let (px, py) = (cx + a.cos() * r, cy + a.sin() * r + dot.size * 0.35);
                        let (key, colour) = if k < filled { (sid!("ui.ring.full"), GOLD) } else { (sid!("ui.ring.empty"), FAINT) };
                        self.centred(canvas, px, py, &TextStyle { colour, ..dot }, &Shown::from_table(self.strings, key));
                    }
                }
            }
        } else if words_in_hand {
            // Nothing in the gaze: the hands' words wait, faintly.
        }

        // The words in hand, at the bottom: the off hand to the left, the main to the right.
        let base = h - 28.0 * d;
        for (i, holding) in player.holding.iter().enumerate() {
            if let Holding::Word(word) = holding {
                let name = self.word_name(*word);
                let charge = player.charge[i].max(player.charge[2]);
                let colour = if charge > 0.0 { GOLD } else { WHITE };
                let style = TextStyle { face: Face::Small, size: (18.0 * d).round(), colour };
                let width = Canvas::measure(self.atlas, &style, &name);
                let x = if i == 1 { 28.0 * d } else { w - 28.0 * d - width };
                self.shadowed(canvas, x, base, &style, &name);
            }
        }
        // Ink: a row of strokes beneath the off hand's word, as many as there is Ink.
        let strokes = ((player.ink / INK_MAX).clamp(0.0, 1.0) * 20.0).round() as usize;
        let ink_style = TextStyle { face: Face::Small, size: (12.0 * d).round(), colour: [0.62, 0.78, 1.0, 0.85] };
        let stroke = Shown::from_table(self.strings, sid!("ui.ink.stroke"));
        let step = Canvas::measure(self.atlas, &ink_style, &stroke);
        for k in 0..20 {
            let style = if k < strokes { ink_style } else { TextStyle { colour: FAINT, ..ink_style } };
            self.shadowed(canvas, 28.0 * d + k as f32 * step, base + 18.0 * d, &style, &stroke);
        }
    }
}
