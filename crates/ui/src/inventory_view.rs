//! The inventory: what is carried, as a list on a ruled page, with the focused item's page
//! beside it: its name, its tier in plain words, its numbers, its material's trait, and
//! what it does. Equipped items carry the held mark. Built from the interface kit.
//!
//! The turning full-body character the brief asks for stands in the space to the left of
//! the list once bodies exist (S04); the list and the page do not move when it arrives.

use vellmark_content::{sid, Face, GlyphAtlas, LootTable, StringId, StringTable};
use vellmark_sim::inventory::Inventory;

use crate::kit::{self, Focus, Row};
use crate::{item_text, Canvas, Part, Shown, TextStyle};

pub struct InventoryView<'a> {
    pub atlas: &'a GlyphAtlas,
    pub strings: &'a StringTable,
    pub loot: &'a LootTable,
    pub size: (u32, u32),
    pub density: f32,
    /// A line said after the last thing done, such as a pairing kept as a favourite.
    pub note: Option<StringId>,
}

/// The rows of the list at this size, so that the caller can keep the focus on screen.
pub fn rows_shown(atlas: &GlyphAtlas, size: (u32, u32), density: f32) -> usize {
    let (_, list) = layout(size, density);
    let style = TextStyle { face: Face::Large, size: (LIST_SIZE * density).round(), colour: kit::ink() };
    ((list[3] / (Canvas::line_height(atlas, &style) * 1.15)).floor() as usize).max(1)
}

const LIST_SIZE: f32 = 17.0;

/// The page's panel, and the list inside it.
fn layout(size: (u32, u32), density: f32) -> ([f32; 4], [f32; 4]) {
    let (w, h) = (size.0 as f32, size.1 as f32);
    let d = density.max(1.0);
    let pw = (w * 0.62).min(1100.0 * d);
    let ph = h * 0.78;
    let panel = [w - pw - w * 0.05, (h - ph) * 0.5, pw, ph];
    let pad = 30.0 * d;
    let list = [panel[0] + pad, panel[1] + pad + 40.0 * d, pw * 0.5 - pad * 1.5, ph - pad * 2.0 - 80.0 * d];
    (panel, list)
}

/// Which slot holds this index, if any.
fn equipped(inv: &Inventory, i: usize) -> bool {
    inv.main == Some(i) || inv.off == Some(i) || inv.body == Some(i) || inv.head == Some(i) || inv.trinkets.contains(&Some(i))
}

impl InventoryView<'_> {
    /// `keys` are the prompt keys for equipping, closing, and keeping the hands as a
    /// favourite, as the platform names them.
    pub fn draw(&self, canvas: &mut Canvas, inv: &Inventory, focus: Focus, keys: (Option<&str>, Option<&str>, Option<&str>)) {
        let d = self.density.max(1.0);
        let (w, h) = (self.size.0 as f32, self.size.1 as f32);
        let (panel, list) = layout(self.size, self.density);
        canvas.rect([0.0, 0.0, w, h], [0.0, 0.0, 0.0, 0.35]);
        kit::panel(canvas, self.atlas, self.strings, panel, d);
        let title = TextStyle { face: Face::Large, size: (24.0 * d).round(), colour: kit::ink() };
        let small = TextStyle { face: Face::Small, size: (15.0 * d).round(), colour: kit::ink() };
        let pad = 30.0 * d;
        canvas.text(self.atlas, panel[0] + pad, panel[1] + pad + 18.0 * d, &title, &Shown::from_table(self.strings, sid!("ui.inventory.title")));
        let marks = Shown::filled(self.strings, sid!("ui.inventory.marks"), &[("n", Part::Number(inv.marks))]);
        let mw = Canvas::measure(self.atlas, &small, &marks);
        canvas.text(self.atlas, list[0] + list[2] - mw, panel[1] + pad + 18.0 * d, &TextStyle { colour: kit::gold(), ..small }, &marks);
        let weight: f32 = inv.items.iter().map(|i| i.weight(self.loot)).sum();
        let carrying = Shown::filled(self.strings, sid!("ui.inventory.weight"), &[("n", Part::Number(weight.round() as u32))]);
        canvas.text(self.atlas, list[0], panel[1] + panel[3] - pad - 4.0 * d, &TextStyle { colour: kit::faint(), ..small }, &carrying);

        if inv.items.is_empty() {
            canvas.text(self.atlas, list[0], list[1] + 20.0 * d, &TextStyle { colour: kit::faint(), ..small }, &Shown::from_table(self.strings, sid!("ui.inventory.empty")));
        } else {
            let rows: Vec<Row<'_>> = inv
                .items
                .iter()
                .enumerate()
                .map(|(i, item)| Row { text: item_text::label(self.strings, self.loot, item), marked: equipped(inv, i), colour: tier_colour(item.tier as u8) })
                .collect();
            kit::list(canvas, self.atlas, self.strings, list, d, &rows, focus, LIST_SIZE);
        }

        // The focused item's page.
        let x = list[0] + list[2] + pad;
        let width = panel[0] + panel[2] - pad - x;
        canvas.rect([x - pad * 0.5, list[1], 1.0 * d, list[3]], kit::faint());
        if let Some(item) = inv.items.get(focus.at) {
            let mut y = list[1] + 6.0 * d;
            let head = TextStyle { face: Face::Large, size: (21.0 * d).round(), colour: tier_colour(item.tier as u8) };
            y += canvas.paragraph(self.atlas, x, y, width, &head, &item_text::name(self.strings, self.loot, item));
            let mut tier = item_text::tier(self.strings, item);
            if equipped(inv, focus.at) {
                let held = Shown::from_table(self.strings, sid!("ui.inventory.held"));
                tier = Shown::filled(self.strings, sid!("ui.inventory.tier_held"), &[("tier", Part::Shown(tier)), ("held", Part::Shown(held))]);
            }
            y += 4.0 * d;
            y += canvas.paragraph(self.atlas, x, y, width, &TextStyle { colour: kit::faint(), ..small }, &tier);
            y += 14.0 * d;
            let line = |canvas: &mut Canvas, y: &mut f32, text: &Shown<'_>, colour: [f32; 4]| {
                *y += canvas.paragraph(self.atlas, x, *y, width, &TextStyle { colour, ..small }, text) + 3.0 * d;
            };
            let stat = |key: StringId, n: f32| Shown::filled(self.strings, key, &[("n", Part::Number(n.round() as u32))]);
            if item.damage(self.loot) > 0.0 {
                line(canvas, &mut y, &stat(sid!("item.stat.damage"), item.damage(self.loot)), kit::ink());
            }
            if item.kills > 0 {
                line(canvas, &mut y, &stat(sid!("item.stat.kills"), item.kills as f32), kit::gold());
            }
            if item.armour(self.loot) > 0.0 {
                line(canvas, &mut y, &stat(sid!("item.stat.armour"), item.armour(self.loot)), kit::ink());
            }
            line(canvas, &mut y, &stat(sid!("item.stat.weight"), item.weight(self.loot)), kit::ink());
            line(canvas, &mut y, &stat(sid!("item.stat.value"), item.value(self.loot) as f32), kit::ink());
            y += 10.0 * d;
            line(canvas, &mut y, &item_text::trait_line(self.strings, self.loot, item), kit::faint());
            y += 6.0 * d;
            for effect in item_text::effects(self.strings, self.loot, item) {
                line(canvas, &mut y, &effect, kit::gold());
            }
        }

        // Prompts, along the foot of the page, and the note above them.
        let mut px = x;
        let py = panel[1] + panel[3] - pad - 4.0 * d;
        if let Some(note) = self.note {
            canvas.text(self.atlas, px, py - 24.0 * d, &TextStyle { colour: kit::gold(), ..small }, &Shown::from_table(self.strings, note));
        }
        for (key, id) in [(keys.0, sid!("ui.inventory.equip")), (keys.2, sid!("ui.favourites.keep")), (keys.1, sid!("ui.inventory.close"))] {
            let Some(key) = key else { continue };
            if id == sid!("ui.inventory.equip") && inv.items.is_empty() {
                continue;
            }
            let text = Shown::filled(self.strings, id, &[("key", Part::Shown(Shown::from_table(self.strings, StringId::from_key(key))))]);
            px += canvas.text(self.atlas, px, py, &small, &text) + 26.0 * d;
        }
    }
}

/// The colour an item's name is set in: plain ink for the lower tiers, gold from Rare, and
/// the colour of fresh ink for Unique things.
fn tier_colour(tier: u8) -> [f32; 4] {
    kit::tier_colour(tier)
}
