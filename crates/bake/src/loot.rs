//! The loot table, from `assets/data/loot.ron`: checked whole, so that every name has its
//! string, every source names only types and materials that exist, and every source's
//! tier chances add up to one.

use std::collections::BTreeMap;
use std::path::Path;

use serde::Deserialize;
use vellmark_content::{LootMaterial, LootModifier, LootSource, LootTable, LootType, LootWordClause};

use crate::BakeError;

#[derive(Deserialize)]
struct Authored {
    materials: Vec<Material>,
    types: Vec<Type>,
    bonuses: Vec<Modifier>,
    clauses: Vec<Modifier>,
    word_clauses: Vec<WordClause>,
    sources: Vec<Source>,
}

#[derive(Deserialize)]
struct Material {
    key: String,
    damage: f32,
    armour: f32,
    weight: f32,
    value: f32,
    #[serde(rename = "trait")]
    trait_key: String,
}

#[derive(Deserialize)]
struct Type {
    key: String,
    slot: String,
    damage: f32,
    armour: f32,
    weight: f32,
    value: u32,
}

#[derive(Deserialize)]
struct Modifier {
    key: String,
    effect: String,
    amount: f32,
}

#[derive(Deserialize)]
struct WordClause {
    key: String,
    word: String,
    effect: String,
}

#[derive(Deserialize)]
struct Source {
    key: String,
    tiers: (f32, f32, f32, f32, f32),
    types: Vec<(String, u32)>,
    materials: Vec<(String, u32)>,
}

/// The effects the rules know, for bonuses and clauses, and for word clauses.
pub const EFFECTS: &[&str] = &["damage", "speed", "reach", "stamina", "stagger", "ink_per_hit", "word_cost", "word_on_weapon", "first_blow", "ink_refill", "staggered_damage", "block_cost", "ink_per_kill", "unaware_damage"];
pub const WORD_EFFECTS: &[&str] = &["word_stays", "word_passes"];
const SLOTS: &[&str] = &["main", "off", "both", "body", "head", "trinket"];

pub fn bake_loot(file: &Path, strings: &BTreeMap<String, String>, words: &[String]) -> Result<LootTable, BakeError> {
    let err = |m: String| BakeError(format!("{}: {m}", file.display()));
    let text = std::fs::read_to_string(file).map_err(|e| err(e.to_string()))?;
    let a: Authored = ron::from_str(&text).map_err(|e| err(e.to_string()))?;
    let need = |key: String| -> Result<(), BakeError> {
        if strings.contains_key(&key) {
            Ok(())
        } else {
            Err(err(format!("no string `{key}` in assets/text")))
        }
    };
    for m in &a.materials {
        need(format!("item.material.{}", m.key))?;
        need(format!("item.trait.{}", m.trait_key))?;
    }
    for t in &a.types {
        need(format!("item.type.{}", t.key))?;
        if !SLOTS.contains(&t.slot.as_str()) {
            return Err(err(format!("{}: no slot called `{}`", t.key, t.slot)));
        }
    }
    for (list, prefix) in [(&a.bonuses, "item.maker"), (&a.clauses, "item.clause")] {
        for m in list {
            need(format!("{prefix}.{}", m.key))?;
            if !EFFECTS.contains(&m.effect.as_str()) {
                return Err(err(format!("{}: no effect called `{}`", m.key, m.effect)));
            }
        }
    }
    for c in &a.word_clauses {
        need(format!("item.word_clause.{}", c.key))?;
        if !words.contains(&c.word) {
            return Err(err(format!("{}: no word `{}`", c.key, c.word)));
        }
        if !WORD_EFFECTS.contains(&c.effect.as_str()) {
            return Err(err(format!("{}: no word effect called `{}`", c.key, c.effect)));
        }
    }
    let mut sources = Vec::new();
    for s in &a.sources {
        let tiers = [s.tiers.0, s.tiers.1, s.tiers.2, s.tiers.3, s.tiers.4];
        if (tiers.iter().sum::<f32>() - 1.0).abs() > 1e-3 {
            return Err(err(format!("{}: the tiers' chances add up to {}, not 1", s.key, tiers.iter().sum::<f32>())));
        }
        let index = |list: &[(String, u32)], keys: &[String], what: &str| -> Result<Vec<(u32, u32)>, BakeError> {
            list.iter()
                .map(|(k, w)| keys.iter().position(|x| x == k).map(|i| (i as u32, *w)).ok_or_else(|| err(format!("{}: no {what} called `{k}`", s.key))))
                .collect()
        };
        let type_keys: Vec<String> = a.types.iter().map(|t| t.key.clone()).collect();
        let material_keys: Vec<String> = a.materials.iter().map(|m| m.key.clone()).collect();
        sources.push(LootSource { key: s.key.clone(), tiers, types: index(&s.types, &type_keys, "type")?, materials: index(&s.materials, &material_keys, "material")? });
    }
    need("item.name".into())?;
    for tier in ["plain", "fair", "endorsed", "engrossed", "illuminated", "canonical"] {
        need(format!("item.tier.{tier}"))?;
    }
    Ok(LootTable {
        materials: a.materials.into_iter().map(|m| LootMaterial { key: m.key, damage: m.damage, armour: m.armour, weight: m.weight, value: m.value, trait_key: m.trait_key }).collect(),
        types: a.types.into_iter().map(|t| LootType { key: t.key, slot: t.slot, damage: t.damage, armour: t.armour, weight: t.weight, value: t.value }).collect(),
        bonuses: a.bonuses.into_iter().map(|m| LootModifier { key: m.key, effect: m.effect, amount: m.amount }).collect(),
        clauses: a.clauses.into_iter().map(|m| LootModifier { key: m.key, effect: m.effect, amount: m.amount }).collect(),
        word_clauses: a.word_clauses.into_iter().map(|c| LootWordClause { key: c.key, word: c.word, effect: c.effect }).collect(),
        sources,
    })
}
