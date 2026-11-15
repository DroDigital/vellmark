//! The item generator against the Showcase's baked loot table: the same seed gives the
//! same item; each source's tiers come up as often as the table says; each tier carries
//! what it should; word clauses favour the finder's words; merchants never sell the top
//! tier; and every name's parts are in the string tables.

use vellmark_content::{AssetId, AssetKind, LootTable, StringId, StringTable, LOOT_KEY};
use vellmark_harness::capture::baked_base_pack;
use vellmark_sim::items::{generate, Tier};

fn table() -> (LootTable, StringTable) {
    let pack = baked_base_pack().expect("the base pack bakes");
    let loot = LootTable::from_bytes(pack.get(AssetKind::Table, AssetId::from_key(LOOT_KEY)).expect("the loot table is in the pack")).unwrap();
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    (loot, strings)
}

#[test]
fn the_generator_is_deterministic_and_follows_the_table() {
    let (t, strings) = table();
    assert_eq!(generate(&t, "chest", 42, &[]), generate(&t, "chest", 42, &[]));
    assert_ne!(generate(&t, "chest", 42, &[]), generate(&t, "chest", 43, &[]));
    for source in &t.sources {
        let n = 20_000;
        let mut counts = [0usize; 5];
        for seed in 0..n {
            let item = generate(&t, &source.key, seed * 7919 + 1, &[]).unwrap();
            counts[item.tier as usize] += 1;
            // Each tier carries what it should.
            let (b, c, w) = item.tier.carries();
            let weapon = matches!(item.slot(&t), "main" | "both" | "off");
            assert_eq!(item.bonuses.len(), b, "{:?}", item);
            assert_eq!(item.clauses.len(), c, "{:?}", item);
            assert_eq!(item.word_clause.is_some(), w > 0 && weapon, "{:?}", item);
            // Every part of its name is written.
            let name = item.name(&t);
            for key in [Some(name.material), Some(name.kind), name.maker, name.clause, name.word_clause].into_iter().flatten() {
                assert!(!strings.text(StringId::from_key(&key)).is_empty(), "no string {key}");
            }
        }
        for (k, chance) in source.tiers.iter().enumerate() {
            let seen = counts[k] as f32 / n as f32;
            assert!((seen - chance).abs() < 0.015, "{}: {:?} came up {seen}, the table says {chance}", source.key, Tier::ROLLED[k]);
        }
    }
}

#[test]
fn annotated_enemies_drop_endorsed_or_better_and_merchants_never_sell_the_top() {
    let (t, _) = table();
    for seed in 0..5000 {
        assert!(generate(&t, "annotated", seed, &[]).unwrap().tier >= Tier::Endorsed);
        assert!(generate(&t, "merchant", seed, &[]).unwrap().tier < Tier::Illuminated);
        assert_eq!(generate(&t, "boss", seed, &[]).unwrap().tier, Tier::Illuminated, "the Bone Clerk's drop is Illuminated");
    }
}

#[test]
fn word_clauses_favour_the_finders_words() {
    let (t, _) = table();
    let known = vec!["COLD".to_string()];
    let (mut cold, mut all) = (0, 0);
    for seed in 0..20_000u64 {
        let item = generate(&t, "boss", seed, &known).unwrap();
        if let Some(w) = item.word_clause {
            all += 1;
            if t.word_clauses[w].word == "COLD" {
                cold += 1;
            }
        }
    }
    let share = cold as f32 / all as f32;
    // Two times in three for an owned word, and half of the rest by chance.
    assert!((share - (2.0 / 3.0 + 1.0 / 6.0)).abs() < 0.03, "{share}");
}

#[test]
fn an_item_is_worth_more_and_on_average_does_more_as_its_tier_rises() {
    let (t, _) = table();
    let mut score_by_tier = [(0.0f32, 0u32); 5];
    let mut worth_by_tier = [0u32; 5];
    for seed in 0..20_000 {
        let item = generate(&t, "chest", seed, &[]).unwrap();
        if t.types[item.kind].key == "sword" && t.materials[item.material].key == "iron" {
            let k = item.tier as usize;
            score_by_tier[k].0 += item.score(&t);
            score_by_tier[k].1 += 1;
            worth_by_tier[k] = worth_by_tier[k].max(item.value(&t));
        }
    }
    let mean: Vec<f32> = score_by_tier.iter().map(|(s, n)| s / (*n).max(1) as f32).collect();
    for k in 1..5 {
        assert!(worth_by_tier[k] > worth_by_tier[k - 1], "{worth_by_tier:?}");
        assert!(mean[k] > mean[k - 1], "on average each tier does more: {mean:?}");
    }
}
