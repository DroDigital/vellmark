//! Bakes the system strings into the executable. They are the messages shown when the
//! game cannot reach its packs at all, so they cannot live only in a pack. The text is
//! still authored in `assets/text`, never in code.

use std::path::{Path, PathBuf};

fn main() {
    let text = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/text/en");
    println!("cargo:rerun-if-changed={}", text.display());
    let baked = vellmark_bake::strings::bake_language(&text).expect("system strings");
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("strings.bin");
    std::fs::write(out, baked.bytes).unwrap();
}
