//! Generated truth on bark. Each tree's record (its kind and how long it has stood) is
//! written round its trunk in the fine print, as the ledger would keep it. Only the trees
//! nearest the eye carry it, since it can be read only at arm's length: a few pages at
//! the end of the page table and the bottom of the page sheet are kept for them, and a
//! page is rewritten only when the tree it is lent to changes.

use glam::Vec3;
use vellmark_content::{PageEntry, TextPages};

use crate::world_gpu::{PAGE_SHEET, SceneGpu};

/// How many trees carry their record at once.
pub const SLOTS: usize = TextPages::GENERATED_PAGES;
/// The page table's entry for slot 0. Authored pages come before.
pub const FIRST_ENTRY: usize = TextPages::MAX_PAGES - SLOTS;
/// Each slot's square of the sheet, in characters: the bottom rows, side by side.
pub const SIDE: u32 = PAGE_SHEET / SLOTS as u32;
/// Where the slots' squares start on the sheet.
pub const SHEET_TOP: u32 = PAGE_SHEET - TextPages::GENERATED_ROWS;

/// The cascade level the record is written at: letters about two and a half centimetres
/// tall, read at arm's length.
const LEVEL: u32 = 1;
const CELL: f32 = 0.0064 * 4.0;
/// The band of the trunk the record runs round, in metres up the bark's page from the
/// foot of the trunk.
const BAND: (f32, f32) = (0.35, 2.75);
/// Lines of the record are this many rows apart, blank between, so that the record
/// reads as one band; the bark's finer grain still shows under it.
const LEADING: u32 = 2;
/// Trees this near an eye, along the ground, carry their record.
const REACH: f32 = 14.0;

/// The words a record is written in, from the string table.
#[derive(Clone, Debug, Default)]
pub struct BarkWords {
    /// With `{kind}` and `{years}` to fill.
    pub record: String,
    /// Each kind's key and its name.
    pub kinds: Vec<(String, String)>,
}

impl BarkWords {
    pub fn write(&self, kind: &str, years: u32) -> String {
        let name = self
            .kinds
            .iter()
            .find(|(k, _)| k == kind)
            .map_or(kind, |(_, n)| n.as_str());
        self.record
            .replace("{kind}", name)
            .replace("{years}", &years.to_string())
    }
}

struct Tree {
    /// Its place in the scene's instance order.
    placed: usize,
    foot: Vec3,
    text: String,
    girth: f32,
}

/// One slot's page: where it goes in the table and its characters, `SIDE` square.
pub struct Written {
    pub slot: usize,
    pub entry: PageEntry,
    pub chars: Vec<u8>,
}

#[derive(Default)]
pub struct Bark {
    trees: Vec<Tree>,
    /// Which tree holds each slot.
    slots: [Option<usize>; SLOTS],
}

impl Bark {
    pub fn new(
        scene: &SceneGpu,
        records: &vellmark_content::TreeRecords,
        words: &BarkWords,
    ) -> Self {
        let place: std::collections::HashMap<u64, usize> = scene
            .placed
            .iter()
            .enumerate()
            .map(|(i, p)| (p.key.0, i))
            .collect();
        let trees = records
            .trees
            .iter()
            .filter_map(|r| {
                let &placed = place.get(&r.tree.0)?;
                Some(Tree {
                    placed,
                    foot: scene.placed[placed].transform.w_axis.truncate(),
                    text: words.write(&r.kind, r.years),
                    girth: r.girth,
                })
            })
            .collect();
        Self {
            trees,
            slots: [None; SLOTS],
        }
    }

    /// Lends the slots to the trees nearest the eyes. A tree that keeps its slot keeps its
    /// page as it is; only the pages of newly lent slots are written and returned.
    pub fn update(&mut self, eyes: &[Vec3]) -> Vec<Written> {
        let near = |t: &Tree| {
            eyes.iter()
                .map(|e| (t.foot.x - e.x).hypot(t.foot.z - e.z))
                .fold(f32::MAX, f32::min)
        };
        let mut nearest: Vec<(f32, usize)> = self
            .trees
            .iter()
            .enumerate()
            .map(|(i, t)| (near(t), i))
            .filter(|(d, _)| *d < REACH)
            .collect();
        nearest.sort_by(|a, b| a.0.total_cmp(&b.0));
        nearest.truncate(SLOTS);
        let wanted: Vec<usize> = nearest.iter().map(|&(_, i)| i).collect();
        for slot in self.slots.iter_mut() {
            if slot.is_some_and(|t| !wanted.contains(&t)) {
                *slot = None;
            }
        }
        let mut written = Vec::new();
        for t in wanted {
            if self.slots.contains(&Some(t)) {
                continue;
            }
            let Some(slot) = self.slots.iter().position(Option::is_none) else {
                break;
            };
            self.slots[slot] = Some(t);
            written.push(self.write(slot, &self.trees[t]));
        }
        written
    }

    /// For each placed thing that carries a record, its page, counted from 1.
    pub fn pages(&self) -> impl Iterator<Item = (usize, u32)> + '_ {
        self.slots
            .iter()
            .enumerate()
            .filter_map(|(s, t)| t.map(|t| (self.trees[t].placed, (FIRST_ENTRY + s + 1) as u32)))
    }

    /// The record written round the trunk, line under line, each line a little further
    /// round than the one above so that the words do not stand in columns.
    fn write(&self, slot: usize, tree: &Tree) -> Written {
        let columns = ((tree.girth / CELL).floor() as u32).clamp(1, SIDE);
        let rows = (((BAND.1 - BAND.0) / CELL).floor() as u32).min(SIDE);
        let mut chars = vec![0u8; (SIDE * SIDE) as usize];
        for row in 0..rows {
            chars[(row * SIDE) as usize..(row * SIDE + columns) as usize].fill(b' ');
        }
        let line: Vec<u8> = format!("{}  ", tree.text)
            .bytes()
            .filter(|b| (32..127).contains(b))
            .collect();
        if !line.is_empty() {
            for row in (0..rows).step_by(LEADING as usize) {
                let shift = (row / LEADING) as usize * 7;
                for column in 0..columns {
                    chars[(row * SIDE + column) as usize] =
                        line[(column as usize + shift) % line.len()];
                }
            }
        }
        Written {
            slot,
            entry: PageEntry {
                level: LEVEL,
                left: 0,
                top: (BAND.1 / CELL).floor() as i32,
                width: columns,
                height: rows,
                sheet_x: slot as u32 * SIDE,
                sheet_y: SHEET_TOP,
            },
            chars,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn words() -> BarkWords {
        BarkWords {
            record: "{kind}. Stood {years} years.".into(),
            kinds: vec![("oak".into(), "Oak".into())],
        }
    }

    #[test]
    fn a_record_is_written_from_its_data() {
        assert_eq!(words().write("oak", 143), "Oak. Stood 143 years.");
    }

    fn bark(feet: &[(f32, f32)]) -> Bark {
        let trees = feet
            .iter()
            .enumerate()
            .map(|(i, &(x, z))| Tree {
                placed: i,
                foot: Vec3::new(x, 0.0, z),
                text: words().write("oak", 100 + i as u32),
                girth: 2.0,
            })
            .collect();
        Bark {
            trees,
            slots: [None; SLOTS],
        }
    }

    #[test]
    fn the_nearest_trees_carry_their_records_and_pages_are_kept() {
        let feet: Vec<(f32, f32)> = (0..12).map(|i| (i as f32 * 1.5, 0.0)).collect();
        let mut b = bark(&feet);
        let first = b.update(&[Vec3::ZERO]);
        // Every slot is lent, to the nearest trees, and only within reach.
        assert_eq!(first.len(), SLOTS);
        let carrying: Vec<usize> = b.pages().map(|(p, _)| p).collect();
        assert_eq!(carrying, (0..SLOTS).collect::<Vec<_>>());
        // Nothing has moved: nothing is written again.
        assert!(b.update(&[Vec3::ZERO]).is_empty());
        // A step along: one tree falls out of the nearest and one comes in; the others
        // keep their slots and their pages.
        let moved = b.update(&[Vec3::new(6.1, 0.0, 0.0)]);
        assert_eq!(moved.len(), 1);
        assert_eq!(moved[0].slot, 0, "the slot given up is lent again");
        // Far from every tree, no tree carries its record.
        b.update(&[Vec3::new(500.0, 0.0, 0.0)]);
        assert_eq!(b.pages().count(), 0);
    }

    #[test]
    fn the_record_runs_round_the_trunk_in_lines() {
        let b = bark(&[(0.0, 0.0)]);
        let w = b.write(3, &b.trees[0]);
        assert_eq!(w.entry.sheet_x, 3 * SIDE);
        assert_eq!(w.entry.width, (2.0 / CELL) as u32);
        let row = |r: u32| {
            String::from_utf8(
                w.chars[(r * SIDE) as usize..(r * SIDE + w.entry.width) as usize].to_vec(),
            )
            .unwrap()
        };
        assert!(
            row(0).starts_with("Oak. Stood 100 years.  Oak."),
            "{}",
            row(0)
        );
        // Blank between the lines, and nothing beyond the band.
        assert!(
            w.chars[SIDE as usize..(SIDE + w.entry.width) as usize]
                .iter()
                .all(|&c| c == b' ')
        );
        assert!(
            w.chars[(SIDE * w.entry.height) as usize..]
                .iter()
                .all(|&c| c == 0)
        );
        assert!(row(2).contains("Stood 100 years."));
        assert_ne!(row(0), row(2), "each line starts further round");
    }
}
