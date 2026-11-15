//! Writes `THIRD_PARTY_NOTICES.txt`: every crate in `Cargo.lock` with its license and
//! the license and notice texts it carries, SDL's own (it is linked statically), and the
//! typefaces' licenses. Run from the repository's root once the crates are fetched:
//! `cargo run -p vellmark-bake --bin notices`. Over-inclusive across platforms, never
//! under: a crate locked for one platform is noticed for all.

use std::path::{Path, PathBuf};

/// The crates locked, as name and version, the game's own left out.
fn locked(lock: &str) -> Vec<(String, String)> {
    let mut out = Vec::new();
    let mut name = None;
    for line in lock.lines() {
        if let Some(n) = line.strip_prefix("name = ") {
            name = Some(n.trim_matches('"').to_string());
        } else if let (Some(v), Some(n)) = (line.strip_prefix("version = "), name.take()) {
            if !n.starts_with("vellmark") {
                out.push((n, v.trim_matches('"').to_string()));
            }
        }
    }
    out.sort();
    out.dedup();
    out
}

/// Where Cargo keeps a crate's source.
fn source(registry: &[PathBuf], name: &str, version: &str) -> Option<PathBuf> {
    registry.iter().map(|r| r.join(format!("{name}-{version}"))).find(|p| p.is_dir())
}

/// The license its manifest states.
fn license(dir: &Path) -> String {
    let manifest = std::fs::read_to_string(dir.join("Cargo.toml")).unwrap_or_default();
    manifest
        .lines()
        .find_map(|l| l.strip_prefix("license = ").map(|v| v.trim_matches('"').to_string()))
        .unwrap_or_else(|| "see its license file".into())
}

/// The license and notice texts at its root, and in `SDL/` for SDL's own.
fn texts(dir: &Path) -> Vec<(String, String)> {
    let mut out = Vec::new();
    for d in [dir.to_path_buf(), dir.join("SDL")] {
        let Ok(read) = std::fs::read_dir(&d) else { continue };
        let mut files: Vec<PathBuf> = read.filter_map(|e| e.ok().map(|e| e.path())).filter(|p| p.is_file()).collect();
        files.sort();
        for f in files {
            let name = f.file_name().and_then(|n| n.to_str()).unwrap_or_default().to_ascii_uppercase();
            if ["LICENSE", "LICENCE", "COPYING", "NOTICE", "COPYRIGHT"].iter().any(|k| name.starts_with(k)) {
                if let Ok(text) = std::fs::read_to_string(&f) {
                    out.push((f.strip_prefix(dir).unwrap_or(&f).display().to_string(), text));
                }
            }
        }
    }
    out
}

fn main() {
    let root = std::env::current_dir().expect("a working directory");
    let lock = std::fs::read_to_string(root.join("Cargo.lock")).expect("run from the repository's root, with its Cargo.lock");
    let home = std::env::var_os("CARGO_HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).unwrap_or_default()).join(".cargo"));
    let registry: Vec<PathBuf> = std::fs::read_dir(home.join("registry/src")).map(|r| r.filter_map(|e| e.ok().map(|e| e.path())).collect()).unwrap_or_default();
    let mut out = String::from(
        "THIRD-PARTY NOTICES\n\nVellmark is built with the following, each under its own license. Generated from the\n\
         dependency tree by crates/bake/src/bin/notices.rs; do not edit by hand.\n\n",
    );
    let mut missing = Vec::new();
    let crates = locked(&lock);
    // Each crate and its license first; then each distinct license or notice text once,
    // with the crates it comes from.
    let mut index = String::new();
    let mut by_text: Vec<(String, Vec<String>)> = Vec::new();
    for (name, version) in &crates {
        let Some(dir) = source(&registry, name, version) else {
            missing.push(format!("{name} {version}"));
            continue;
        };
        index.push_str(&format!("{name} {version}: {}\n", license(&dir)));
        for (_, text) in texts(&dir) {
            let text = text.trim_end().to_string();
            match by_text.iter_mut().find(|(t, _)| *t == text) {
                Some((_, from)) => from.push(format!("{name} {version}")),
                None => by_text.push((text, vec![format!("{name} {version}")])),
            }
        }
    }
    out.push_str("CRATES AND THEIR LICENSES\n\n");
    out.push_str(&index);
    out.push_str("\nLICENSE AND NOTICE TEXTS\n\n");
    for (text, from) in &by_text {
        out.push_str(&format!("{}\nFrom: {}\n\n{text}\n\n", "=".repeat(78), from.join(", ")));
    }
    for font in ["imfellenglish", "tiny5"] {
        let ofl = root.join("third_party/fonts").join(font).join("OFL.txt");
        if let Ok(text) = std::fs::read_to_string(&ofl) {
            out.push_str(&format!("{}\nTypeface: {font}\nLicense: OFL-1.1\n\n{}\n\n", "=".repeat(78), text.trim_end()));
        }
    }
    std::fs::write(root.join("THIRD_PARTY_NOTICES.txt"), &out).expect("THIRD_PARTY_NOTICES.txt is written");
    println!("{} crates noticed, {} not fetched", crates.len() - missing.len(), missing.len());
    if !missing.is_empty() {
        eprintln!("not fetched, so not noticed (fetch them and run again): {}", missing.join(", "));
        std::process::exit(1);
    }
}
