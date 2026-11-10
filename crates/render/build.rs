//! Validates every shader at build time and embeds their sources.

use std::path::{Path, PathBuf};

include!("src/shader_pre.rs");

fn main() {
    let dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../shaders");
    println!("cargo:rerun-if-changed={}", dir.display());
    let mut files: Vec<PathBuf> = std::fs::read_dir(&dir)
        .expect("the shaders folder is missing")
        .map(|e| e.unwrap().path())
        .filter(|p| p.extension().and_then(|e| e.to_str()) == Some("wgsl"))
        .collect();
    files.sort();

    let name_of = |p: &Path| p.file_stem().unwrap().to_str().unwrap().to_owned();
    let source_of = |name: &str| std::fs::read_to_string(dir.join(format!("{name}.wgsl"))).ok();

    let mut table = String::from("pub static EMBEDDED_SHADERS: &[(&str, &str)] = &[\n");
    let mut failed = false;
    for file in &files {
        println!("cargo:rerun-if-changed={}", file.display());
        let name = name_of(file);
        let raw = std::fs::read_to_string(file).unwrap();
        // A file with no entry point is a library, checked through the shaders that import it.
        let is_entry = raw.contains("@vertex") || raw.contains("@fragment") || raw.contains("@compute");
        if is_entry {
            match expand_shader(&name, &source_of) {
                Err(e) => {
                    println!("cargo:warning={e}");
                    failed = true;
                }
                Ok(full) => match naga::front::wgsl::parse_str(&full) {
                    Err(e) => {
                        for line in e.emit_to_string(&full).lines() {
                            println!("cargo:warning={name}.wgsl: {line}");
                        }
                        failed = true;
                    }
                    Ok(module) => {
                        let mut validator = naga::valid::Validator::new(
                            naga::valid::ValidationFlags::all(),
                            naga::valid::Capabilities::empty(),
                        );
                        if let Err(e) = validator.validate(&module) {
                            for line in e.emit_to_string(&full).lines() {
                                println!("cargo:warning={name}.wgsl: {line}");
                            }
                            failed = true;
                        }
                    }
                },
            }
        }
        let path = file.canonicalize().unwrap();
        table.push_str(&format!("    ({name:?}, include_str!({:?})),\n", path.display().to_string()));
    }
    table.push_str("];\n");
    if failed {
        panic!("shader validation failed");
    }
    let out = PathBuf::from(std::env::var("OUT_DIR").unwrap()).join("embedded_shaders.rs");
    std::fs::write(out, table).unwrap();
}
