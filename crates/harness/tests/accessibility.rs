//! Accessibility's words: every caption the game can show is written, for every word
//! and every typo it can ring, every sound not to miss, and every state a creature can be
//! said to be in.

use vellmark_content::{AssetKind, StringId, StringTable};
use vellmark_harness::capture::baked_base_pack;

#[test]
fn every_caption_and_every_creature_state_is_written() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    let has = |key: &str| assert!(strings.get(StringId::from_key(key)).is_some(), "no string {key}");
    let rules = vellmark_render::word_rules(&pack);
    for w in &rules.words {
        has(&format!("caption.word.{}", w.spelling.to_ascii_lowercase()));
        has(&format!("caption.word.{}", w.typo.to_ascii_lowercase()));
    }
    for key in ["motif", "motif_wrong", "learned", "gather", "call", "break", "mend", "redactor"] {
        has(&format!("caption.sound.{key}"));
    }
    for state in ["unaware", "aware", "gathering", "staggered", "calm", "burning"] {
        has(&format!("creature.state.{state}"));
    }
}
