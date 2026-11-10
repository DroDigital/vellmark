use std::path::PathBuf;
use std::process::ExitCode;
use vellmark_bake::{bake_to_dir, BakeInputs};

fn main() -> ExitCode {
    let mut assets = PathBuf::from("assets");
    let mut third_party = PathBuf::from("third_party");
    let mut out = PathBuf::from("build/packs");
    let mut cache = Some(PathBuf::from("build/bake-cache"));
    let mut preview: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = |name: &str| args.next().unwrap_or_else(|| panic!("{name} needs a path"));
        match arg.as_str() {
            "--assets" => assets = value("--assets").into(),
            "--third-party" => third_party = value("--third-party").into(),
            "--out" => out = value("--out").into(),
            "--no-cache" => cache = None,
            "--preview-atlas" => preview = Some(value("--preview-atlas").into()),
            other => {
                eprintln!("unknown argument `{other}`\nusage: vellmark-bake [--assets DIR] [--third-party DIR] [--out DIR]");
                return ExitCode::from(2);
            }
        }
    }
    if let Some(folder) = preview {
        return match preview_atlas(&third_party, cache.as_deref(), &folder) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("preview failed: {e}");
                ExitCode::FAILURE
            }
        };
    }
    match bake_to_dir(&BakeInputs { assets, third_party, cache }, &out) {
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

/// Writes sheets of every glyph drawn from the atlas at several sizes, to look at.
fn preview_atlas(third_party: &std::path::Path, cache: Option<&std::path::Path>, folder: &std::path::Path) -> Result<(), String> {
    let bytes = vellmark_bake::glyphs::bake_atlas(third_party, cache).map_err(|e| e.to_string())?;
    let atlas = vellmark_content::GlyphAtlas::from_bytes(&bytes)?;
    std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    for (px, threshold, name) in [(8u32, 0.5f32, "8px"), (16, 0.5, "16px"), (48, 0.5, "48px"), (48, 0.35, "48px_heavy"), (48, 0.65, "48px_light")] {
        let (rgba, w, h) = vellmark_bake::glyphs::preview_sheet(&atlas, px, threshold);
        let file = std::fs::File::create(folder.join(format!("atlas_{name}.png"))).map_err(|e| e.to_string())?;
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), w, h);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder.write_header().and_then(|mut w| w.write_image_data(&rgba)).map_err(|e| e.to_string())?;
    }
    Ok(())
}
