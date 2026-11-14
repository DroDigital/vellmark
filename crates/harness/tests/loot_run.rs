//! The loot simulation of S05-T12: the Showcase's 45 minutes played ten thousand times
//! against the baked loot table, counting what is found. Every rolled tier must turn up
//! in at least nine runs in ten, the Illuminated item always (the Bone Clerk's), and an
//! Illuminated item of one material must never be weaker than a Plain one of the next.
//!
//! A run, as the Showcase plans it: the ossuary's floor, the village, and the orchard
//! (six chests), the mill cellar's chest, the hounds and the Shambling Entries (six
//! enemies), one annotated hound and one annotated Entry, and the Bone Clerk.

use vellmark_content::{AssetId, AssetKind, LootTable, LOOT_KEY};
use vellmark_harness::capture::baked_base_pack;
use vellmark_sim::items::{generate, Item, Tier};

const RUN: &[(&str, u32)] = &[("chest", 6), ("cellar", 1), ("enemy", 6), ("annotated", 2), ("boss", 1)];
const RUNS: u64 = 10_000;

#[test]
fn every_tier_turns_up_in_a_run_and_no_tier_inverts_across_materials() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let t = LootTable::from_bytes(pack.get(AssetKind::Table, AssetId::from_key(LOOT_KEY)).unwrap()).unwrap();
    let mut runs_with = [0u64; 5];
    let mut found: Vec<Item> = Vec::new();
    for run in 0..RUNS {
        let mut seen = [false; 5];
        let mut k = 0u64;
        for (source, count) in RUN {
            for _ in 0..*count {
                k += 1;
                let item = generate(&t, source, run.wrapping_mul(0x9e37_79b9_7f4a_7c15) ^ k.wrapping_mul(0xbf58_476d_1ce4_e5b9), &["COLD".into()]).unwrap();
                seen[item.tier as usize] = true;
                if run < 500 {
                    found.push(item);
                }
            }
        }
        for (n, s) in runs_with.iter_mut().zip(seen) {
            *n += s as u64;
        }
    }
    let share: Vec<f32> = runs_with.iter().map(|n| *n as f32 / RUNS as f32).collect();
    let report = format!(
        "runs in which each tier turned up, Plain to Illuminated: {}",
        Tier::ROLLED.iter().zip(&share).map(|(t, s)| format!("{t:?} {:.1}%", s * 100.0)).collect::<Vec<_>>().join(", ")
    );
    std::fs::create_dir_all(vellmark_harness::repo_root().join("build/captures/loot")).unwrap();
    std::fs::write(vellmark_harness::repo_root().join("build/captures/loot/run.txt"), &report).unwrap();
    for (tier, s) in Tier::ROLLED.iter().zip(&share) {
        assert!(*s >= 0.9, "{tier:?} turns up in too few runs: {report}");
    }
    assert_eq!(share[4], 1.0, "the Bone Clerk's Illuminated item, every run");
    // An Illuminated item of a weaker material is never weaker than a Plain one of the next.
    let mut order: Vec<usize> = (0..t.materials.len()).collect();
    order.sort_by(|a, b| t.materials[*a].damage.total_cmp(&t.materials[*b].damage));
    for pair in order.windows(2) {
        for kind in 0..t.types.len() {
            let worst_top = found.iter().filter(|i| i.kind == kind && i.material == pair[0] && i.tier == Tier::Illuminated).map(|i| i.score(&t)).fold(f32::MAX, f32::min);
            let best_plain = found.iter().filter(|i| i.kind == kind && i.material == pair[1] && i.tier == Tier::Plain).map(|i| i.score(&t)).fold(f32::MIN, f32::max);
            if worst_top < f32::MAX && best_plain > f32::MIN {
                assert!(worst_top >= best_plain, "{}: Illuminated {} ({worst_top}) is weaker than Plain {} ({best_plain})", t.types[kind].key, t.materials[pair[0]].key, t.materials[pair[1]].key);
            }
        }
    }
}
