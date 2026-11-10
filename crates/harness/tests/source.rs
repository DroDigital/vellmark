//! No player-facing text in code, and the bake is deterministic.

use vellmark_bake::{bake, BakeInputs};
use vellmark_content::{AssetKind, Pack, StringId, StringTable};
use vellmark_harness::{repo_root, source_checks};

fn inputs_with(cache: bool) -> BakeInputs {
    let root = repo_root();
    BakeInputs {
        assets: root.join("assets"),
        third_party: root.join("third_party"),
        cache: cache.then(|| root.join("build/bake-cache")),
    }
}

fn inputs() -> BakeInputs {
    inputs_with(true)
}

#[test]
fn two_bakes_are_byte_identical() {
    // Two bakes from nothing, and one through the cache: all three must agree.
    let a = bake(&inputs_with(false)).unwrap();
    let b = bake(&inputs_with(false)).unwrap();
    let cached = bake(&inputs_with(true)).unwrap();
    assert!(a.packs == cached.packs, "a cached bake differs from a fresh one");
    assert_eq!(a.packs.len(), b.packs.len());
    for ((name_a, bytes_a), (name_b, bytes_b)) in a.packs.iter().zip(&b.packs) {
        assert_eq!(name_a, name_b);
        assert!(bytes_a == bytes_b, "{name_a} differs between two bakes");
        Pack::from_bytes(bytes_a.clone()).unwrap().verify().unwrap();
    }
}

#[test]
fn every_string_key_in_code_exists_in_the_table() {
    let output = bake(&inputs()).unwrap();
    let pack = Pack::from_bytes(output.packs[0].1.clone()).unwrap();
    let bytes = pack
        .get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE))
        .expect("the base pack holds the string table");
    let table = StringTable::from_bytes(bytes).unwrap();
    let missing: Vec<String> = source_checks::string_keys_in_code(&repo_root())
        .into_iter()
        .filter(|u| !table.contains(StringId::from_key(&u.key)))
        .map(|u| format!("{}:{}: `{}`", u.file.display(), u.line, u.key))
        .collect();
    assert!(missing.is_empty(), "string keys used in code but not in assets/text:\n{}", missing.join("\n"));
}

#[test]
fn development_text_stays_in_development_modules() {
    let found = source_checks::dev_text_outside_dev_modules(&repo_root());
    assert!(found.is_empty(), "dev_text used outside a dev module: {found:?}");
}
