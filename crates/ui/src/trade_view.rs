//! Trade with a merchant: their stock and the pack side by side on ruled pages, each item
//! at its price (theirs at its worth, yours at what they will give), the Marks on each
//! side, and, when a trade cannot be made, why, in plain words.

use vellmark_content::{sid, Face, GlyphAtlas, LootTable, StringId, StringTable};
use vellmark_sim::inventory::Inventory;
use vellmark_sim::merchant::{Merchant, Refusal};

use crate::kit::{self, Focus, Row};
use crate::{item_text, Canvas, Part, Shown, TextStyle};

/// Which side the focus is on.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Side {
    #[default]
    Theirs,
    Yours,
}

/// The trade screen's own state: the side and the focus on each, and the last refusal.
#[derive(Clone, Debug, Default)]
pub struct Trade {
    pub merchant: u64,
    pub side: Side,
    pub theirs: Focus,
    pub yours: Focus,
    pub refused: Option<Refusal>,
}

const LIST_SIZE: f32 = 16.0;

pub struct TradeView<'a> {
    pub atlas: &'a GlyphAtlas,
    pub strings: &'a StringTable,
    pub loot: &'a LootTable,
    pub size: (u32, u32),
    pub density: f32,
}

impl TradeView<'_> {
    fn pages(&self) -> ([f32; 4], [f32; 4]) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let d = self.density.max(1.0);
        let pw = ((w * 0.9 - 24.0 * d) * 0.5).min(560.0 * d);
        let ph = h * 0.74;
        let top = (h - ph) * 0.5;
        let gap = 24.0 * d;
        let left = (w - pw * 2.0 - gap) * 0.5;
        ([left, top, pw, ph], [left + pw + gap, top, pw, ph])
    }

    /// The rows each page shows at once.
    pub fn rows(&self) -> usize {
        let (page, _) = self.pages();
        let style = TextStyle { face: Face::Large, size: (LIST_SIZE * self.density.max(1.0)).round(), colour: kit::ink() };
        (((page[3] - 140.0 * self.density.max(1.0)) / (Canvas::line_height(self.atlas, &style) * 1.15)).floor() as usize).max(1)
    }

    pub fn draw(&self, canvas: &mut Canvas, trade: &Trade, merchant: &Merchant, inv: &Inventory, keys: (Option<&str>, Option<&str>)) {
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let d = self.density.max(1.0);
        canvas.rect([0.0, 0.0, w, h], [0.0, 0.0, 0.0, 0.4]);
        let (left, right) = self.pages();
        let title = TextStyle { face: Face::Large, size: (22.0 * d).round(), colour: kit::ink() };
        let small = TextStyle { face: Face::Small, size: (15.0 * d).round(), colour: kit::ink() };
        let pad = 26.0 * d;
        for (page, side) in [(left, Side::Theirs), (right, Side::Yours)] {
            kit::panel(canvas, self.atlas, self.strings, page, d);
            let (heading, marks) = match side {
                Side::Theirs => (sid!("ui.trade.theirs"), merchant.purse),
                Side::Yours => (sid!("ui.trade.yours"), inv.marks),
            };
            let colour = if trade.side == side { kit::gold() } else { kit::ink() };
            canvas.text(self.atlas, page[0] + pad, page[1] + pad + 16.0 * d, &TextStyle { colour, ..title }, &Shown::from_table(self.strings, heading));
            let purse = Shown::filled(self.strings, sid!("ui.inventory.marks"), &[("n", Part::Number(marks))]);
            let mw = Canvas::measure(self.atlas, &small, &purse);
            canvas.text(self.atlas, page[0] + page[2] - pad - mw, page[1] + pad + 16.0 * d, &TextStyle { colour: kit::gold(), ..small }, &purse);
            let list = [page[0] + pad, page[1] + pad + 36.0 * d, page[2] - pad * 2.0, page[3] - pad * 2.0 - 80.0 * d];
            let (items, focus): (Vec<&vellmark_sim::items::Item>, Focus) = match side {
                Side::Theirs => (merchant.stock.iter().collect(), trade.theirs),
                Side::Yours => (inv.items.iter().collect(), trade.yours),
            };
            if items.is_empty() {
                canvas.text(self.atlas, list[0], list[1] + 20.0 * d, &TextStyle { colour: kit::faint(), ..small }, &Shown::from_table(self.strings, sid!("ui.inventory.empty")));
                continue;
            }
            let rows: Vec<Row<'_>> = items
                .iter()
                .map(|item| Row { text: item_text::label(self.strings, self.loot, item), marked: false, colour: kit::tier_colour(item.tier as u8) })
                .collect();
            let shown_focus = if trade.side == side { focus } else { Focus { at: usize::MAX, scroll: focus.scroll } };
            let style = TextStyle { face: Face::Large, size: (LIST_SIZE * d).round(), colour: kit::ink() };
            let price_w = Canvas::measure(self.atlas, &style, &Shown::filled(self.strings, sid!("ui.trade.price"), &[("n", Part::Number(9999))]));
            kit::list(canvas, self.atlas, self.strings, [list[0], list[1], list[2] - price_w - 8.0 * d, list[3]], d, &rows, shown_focus, LIST_SIZE);
            // Each price, right-aligned on its row.
            let line = Canvas::line_height(self.atlas, &style) * 1.15;
            let fit = ((list[3] / line).floor() as usize).max(1);
            for (k, item) in items.iter().enumerate().skip(focus.scroll).take(fit) {
                let price = match side {
                    Side::Theirs => merchant.asks(self.loot, item),
                    Side::Yours => merchant.pays(self.loot, item),
                };
                let text = Shown::filled(self.strings, sid!("ui.trade.price"), &[("n", Part::Number(price))]);
                let tw = Canvas::measure(self.atlas, &style, &text);
                let top = list[1] + (k - focus.scroll) as f32 * line + line * 0.8;
                let c = if trade.side == side && k == focus.at { kit::gold() } else { kit::faint() };
                canvas.text(self.atlas, list[0] + list[2] - tw, top, &TextStyle { colour: c, ..style }, &text);
            }
        }
        // Why the last trade could not be made, and the prompts, along the foot.
        let foot = left[1] + left[3] + 26.0 * d;
        if let Some(r) = trade.refused {
            let key = match r {
                Refusal::TooDear => sid!("ui.trade.too_dear"),
                Refusal::PurseEmpty => sid!("ui.trade.purse_empty"),
                Refusal::InUse => sid!("ui.trade.in_use"),
                Refusal::Nothing => sid!("ui.trade.nothing"),
            };
            let text = Shown::from_table(self.strings, key);
            let tw = Canvas::measure(self.atlas, &small, &text);
            canvas.text(self.atlas, (w - tw) * 0.5, foot, &TextStyle { colour: kit::gold(), ..small }, &text);
        }
        let mut px = left[0];
        let py = foot + 22.0 * d;
        let verb = if trade.side == Side::Theirs { sid!("ui.trade.buy") } else { sid!("ui.trade.sell") };
        for (key, id) in [(keys.0, verb), (keys.1, sid!("ui.trade.leave"))] {
            let Some(key) = key else { continue };
            let text = Shown::filled(self.strings, id, &[("key", Part::Shown(Shown::from_table(self.strings, StringId::from_key(key))))]);
            px += canvas.text(self.atlas, px, py, &TextStyle { colour: kit::faint(), ..small }, &text) + 26.0 * d;
        }
    }
}
