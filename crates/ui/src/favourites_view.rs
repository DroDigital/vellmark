//! The favourites wheel: up to eight pairings kept from the pack, set round a ring, each
//! the main hand's item over what the other hand holds. Held open by its button, aimed
//! by the mouse or a stick, and let go to take the pairing up at once.

use vellmark_content::{sid, Face, GlyphAtlas, LootTable, StringId, StringTable};
use vellmark_sim::emendation::WordRules;
use vellmark_sim::inventory::{Favourite, Inventory};

use crate::kit;
use crate::{item_text, Canvas, Part, Shown, TextStyle};

/// The most pairings the wheel holds, as the sim keeps.
pub const SLOTS: usize = 8;

/// How far the aim must be from the middle before a slot is chosen, as a share of a
/// stick's full throw.
const DEAD: f32 = 0.35;

/// Mouse movement, in counts, that makes a full throw.
const MOUSE_THROW: f32 = 120.0;

/// The wheel's own state while it is held open: the aim, from the middle.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
pub struct Wheel {
    pub aim: (f32, f32),
}

impl Wheel {
    /// Mouse movement moves the aim; it is kept within a full throw.
    pub fn mouse(&mut self, dx: f32, dy: f32) {
        let (x, y) = (self.aim.0 + dx / MOUSE_THROW, self.aim.1 + dy / MOUSE_THROW);
        let len = (x * x + y * y).sqrt();
        self.aim = if len > 1.0 { (x / len, y / len) } else { (x, y) };
    }

    /// A stick sets the aim where it points, and keeps the last choice when let back to
    /// the middle, so a pairing can be taken by releasing the button after the stick.
    pub fn stick(&mut self, x: f32, y: f32) {
        if x * x + y * y > DEAD * DEAD {
            self.aim = (x, y);
        }
    }

    /// The slot aimed at among `count`, if the aim is out of the middle. Slot 0 is at the
    /// top and the rest follow clockwise; screen y grows downward.
    pub fn slot(&self, count: usize) -> Option<usize> {
        let (x, y) = self.aim;
        if count == 0 || x * x + y * y < DEAD * DEAD {
            return None;
        }
        let angle = x.atan2(-y).rem_euclid(std::f32::consts::TAU);
        let step = std::f32::consts::TAU / count as f32;
        Some(((angle / step).round() as usize) % count)
    }
}

pub struct FavouritesView<'a> {
    pub atlas: &'a GlyphAtlas,
    pub strings: &'a StringTable,
    pub rules: &'a WordRules,
    pub loot: &'a LootTable,
    pub size: (u32, u32),
    pub density: f32,
}

impl FavouritesView<'_> {
    fn word(&self, w: usize) -> Shown<'_> {
        match self.rules.words.get(w) {
            Some(word) => Shown::from_table(self.strings, StringId::from_key(&format!("caption.word.{}", word.spelling.to_ascii_lowercase()))),
            None => Shown::from_table(self.strings, sid!("ui.favourites.empty_hand")),
        }
    }

    fn item(&self, inv: &Inventory, i: Option<usize>, none: StringId) -> (Shown<'_>, [f32; 4]) {
        match i.and_then(|i| inv.items.get(i)) {
            Some(item) => (item_text::label(self.strings, self.loot, item), kit::tier_colour(item.tier as u8)),
            None => (Shown::from_table(self.strings, none), kit::faint()),
        }
    }

    /// `keep` is the prompt key that keeps a pairing from the pack, for the empty wheel.
    pub fn draw(&self, canvas: &mut Canvas, wheel: &Wheel, inv: &Inventory, keep: Option<&str>) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let d = self.density.max(1.0);
        let (cx, cy) = (w * 0.5, h * 0.5);
        canvas.rect([0.0, 0.0, w, h], [0.0, 0.0, 0.0, 0.5]);
        // A field behind the middle, so the title reads over any sky or wall.
        let field = kit::paper();
        let title = TextStyle { face: Face::Large, size: (22.0 * d).round(), colour: kit::ink() };
        let small = TextStyle { face: Face::Small, size: (15.0 * d).round(), colour: kit::ink() };
        let line = TextStyle { face: Face::Large, size: (17.0 * d).round(), colour: kit::ink() };
        let centred = |canvas: &mut Canvas, x: f32, y: f32, style: &TextStyle, text: &Shown<'_>| {
            let tw = Canvas::measure(self.atlas, style, text);
            canvas.text(self.atlas, x - tw * 0.5, y, style, text);
        };
        let heading = Shown::from_table(self.strings, sid!("ui.favourites.title"));
        let count = inv.favourites.len().min(SLOTS);
        if count == 0 {
            centred(canvas, cx, cy - 8.0 * d, &title, &heading);
            if let Some(key) = keep {
                let text = Shown::filled(self.strings, sid!("ui.favourites.none"), &[("key", Part::Shown(Shown::from_table(self.strings, StringId::from_key(key))))]);
                centred(canvas, cx, cy + 24.0 * d, &TextStyle { colour: kit::faint(), ..small }, &text);
            }
            return;
        }
        let chosen = wheel.slot(count);
        // The middle: the title, and a mark that points the way of the aim.
        let tw = Canvas::measure(self.atlas, &title, &heading) + 32.0 * d;
        canvas.rect([cx - tw * 0.5, cy - 20.0 * d, tw, 36.0 * d], field);
        centred(canvas, cx, cy + 7.0 * d, &title, &heading);
        let radius = (h * 0.3).min(w * 0.28);
        if let Some(k) = chosen {
            let a = k as f32 * std::f32::consts::TAU / count as f32;
            let (s, c) = a.sin_cos();
            let mark = Shown::from_table(self.strings, sid!("ui.favourites.pointer"));
            centred(canvas, cx + s * radius * 0.42, cy - c * radius * 0.42 + 10.0 * d, &TextStyle { colour: kit::gold(), size: (30.0 * d).round(), ..line }, &mark);
        }
        let lh = Canvas::line_height(self.atlas, &line);
        for (k, (main, off)) in inv.favourites.iter().take(count).enumerate() {
            let a = k as f32 * std::f32::consts::TAU / count as f32;
            let (s, c) = a.sin_cos();
            let (x, y) = (cx + s * radius, cy - c * radius);
            let (top, top_colour) = self.item(inv, *main, sid!("ui.favourites.bare_hand"));
            let (bottom, bottom_colour) = match off {
                Favourite::Item(i) => self.item(inv, Some(*i), sid!("ui.favourites.empty_hand")),
                Favourite::Word(word) => (self.word(*word), kit::gold()),
                Favourite::Empty => (Shown::from_table(self.strings, sid!("ui.favourites.empty_hand")), kit::faint()),
            };
            let on = chosen == Some(k);
            let bw = Canvas::measure(self.atlas, &line, &top).max(Canvas::measure(self.atlas, &line, &bottom)) + 28.0 * d;
            let box_ = [x - bw * 0.5, y - lh * 1.2, bw, lh * 2.5];
            canvas.rect(box_, if on { field } else { [field[0], field[1], field[2], field[3] * 0.8] });
            if on {
                let rule = kit::gold();
                canvas.rect([box_[0], box_[1], box_[2], 1.5 * d], rule);
                canvas.rect([box_[0], box_[1] + box_[3] - 1.5 * d, box_[2], 1.5 * d], rule);
            }
            let dim = |c: [f32; 4]| if on { c } else { [c[0] * 0.8, c[1] * 0.8, c[2] * 0.8, c[3]] };
            centred(canvas, x, y - lh * 0.15, &TextStyle { colour: dim(top_colour), ..line }, &top);
            centred(canvas, x, y + lh * 0.95, &TextStyle { colour: dim(bottom_colour), ..line }, &bottom);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_aim_chooses_the_slot_it_points_at() {
        let mut wheel = Wheel::default();
        assert_eq!(wheel.slot(8), None, "the middle chooses nothing");
        wheel.stick(0.0, -1.0);
        assert_eq!(wheel.slot(8), Some(0), "up is the first");
        wheel.stick(1.0, 0.0);
        assert_eq!(wheel.slot(8), Some(2), "right is a quarter round");
        assert_eq!(wheel.slot(4), Some(1));
        wheel.stick(0.0, 0.0);
        assert_eq!(wheel.slot(8), Some(2), "a stick let go keeps its choice");
        wheel.stick(-0.7, 0.7);
        assert_eq!(wheel.slot(8), Some(5), "down and left");
        let mut wheel = Wheel::default();
        wheel.mouse(0.0, 30.0);
        assert_eq!(wheel.slot(8), None, "a small move stays in the middle");
        wheel.mouse(0.0, 400.0);
        assert_eq!(wheel.slot(8), Some(4), "down");
        assert!((wheel.aim.1 - 1.0).abs() < 1e-5, "kept within a full throw");
        assert_eq!(wheel.slot(0), None);
    }
}
