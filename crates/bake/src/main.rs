use std::path::PathBuf;
use std::process::ExitCode;
use vellmark_bake::{bake_to_dir, BakeInputs};

fn main() -> ExitCode {
    let mut assets = PathBuf::from("assets");
    let mut third_party = PathBuf::from("third_party");
    let mut out = PathBuf::from("build/packs");
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = |name: &str| args.next().unwrap_or_else(|| panic!("{name} needs a path"));
        match arg.as_str() {
            "--assets" => assets = value("--assets").into(),
            "--third-party" => third_party = value("--third-party").into(),
            "--out" => out = value("--out").into(),
            other => {
                eprintln!("unknown argument `{other}`\nusage: vellmark-bake [--assets DIR] [--third-party DIR] [--out DIR]");
                return ExitCode::from(2);
            }
        }
    }
    match bake_to_dir(&BakeInputs { assets, third_party }, &out) {
        Ok(output) => {
            for line in &output.report {
                println!("{line}");
            }
            for (name, bytes) in &output.packs {
                println!("{}: {} bytes", out.join(name).display(), bytes.len());
            }
            ExitCode::SUCCESS
        }
        Err(e) => {
            eprintln!("bake failed: {e}");
            ExitCode::FAILURE
        }
    }
}
