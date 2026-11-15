//! An item in words: its full name from its parts, its label between its tier's marks,
//! its tier in plain words, and each thing it does. The HUD and every screen that lists
//! items name them here, so an item reads the same wherever it is seen.

use vellmark_content::{sid, LootTable, StringId, StringTable};
use vellmark_sim::items::Item;

use crate::{Part, Shown};

fn keyed<'a>(strings: &'a StringTable, key: &str) -> Shown<'a> {
    Shown::from_table(strings, StringId::from_key(key))
}

/// Its full name: maker, material, type, clause, and word clause; or a Canonical item's
/// own name.
pub fn name<'a>(strings: &'a StringTable, table: &LootTable, item: &Item) -> Shown<'a> {
    let name = item.name(table);
    if name.material.is_empty() {
        return keyed(strings, &name.kind);
    }
    let part = |key: &Option<String>| key.as_ref().map_or(Part::Nothing, |k| Part::Shown(keyed(strings, k)));
    Shown::filled(
        strings,
        sid!("item.name"),
        &[
            ("maker", part(&name.maker)),
            ("material", part(&Some(name.material.clone()))),
            ("type", part(&Some(name.kind.clone()))),
            ("clause", part(&name.clause)),
            ("word_clause", part(&name.word_clause)),
        ],
    )
}

/// Its name between its tier's marks: `+ Tally's Iron Sword +`.
pub fn label<'a>(strings: &'a StringTable, table: &LootTable, item: &Item) -> Shown<'a> {
    let tier = item.tier.key();
    let mark = || Part::Shown(keyed(strings, &format!("item.mark.{tier}")));
    Shown::filled(strings, sid!("item.label"), &[("mark", mark()), ("name", Part::Shown(name(strings, table, item)))])
}

/// Its tier, in plain words: Common to Unique.
pub fn tier<'a>(strings: &'a StringTable, item: &Item) -> Shown<'a> {
    keyed(strings, &format!("item.tier.{}", item.tier.key()))
}

/// Its material's trait.
pub fn trait_line<'a>(strings: &'a StringTable, table: &LootTable, item: &Item) -> Shown<'a> {
    keyed(strings, &format!("item.trait.{}", table.materials[item.material].trait_key))
}

/// Effects counted in whole units rather than as shares (reach, in metres, is shown in
/// centimetres, so it is scaled as a share is).
const UNITS: &[&str] = &["ink_per_kill"];

/// One line for each thing it does beyond its numbers: its bonuses and clauses, its word
/// clause, and a Canonical item's own effect.
pub fn effects<'a>(strings: &'a StringTable, table: &LootTable, item: &Item) -> Vec<Shown<'a>> {
    let mut lines = Vec::new();
    let modifiers = item.bonuses.iter().filter_map(|i| table.bonuses.get(*i)).chain(item.clauses.iter().filter_map(|i| table.clauses.get(*i)));
    for m in modifiers {
        let n = if UNITS.contains(&m.effect.as_str()) { m.amount } else { m.amount * 100.0 };
        lines.push(Shown::filled(strings, StringId::from_key(&format!("item.effect.{}", m.effect)), &[("n", Part::Signed(n.round() as i32))]));
    }
    if let Some(c) = item.word_clause.and_then(|w| table.word_clauses.get(w)) {
        let word = Part::Shown(keyed(strings, &format!("caption.word.{}", c.word.to_ascii_lowercase())));
        lines.push(Shown::filled(strings, StringId::from_key(&format!("item.effect.{}", c.effect)), &[("word", word)]));
    }
    if let Some(effect) = item.canonical_effect(table) {
        lines.push(keyed(strings, &format!("item.effect.{effect}")));
    }
    lines
}
