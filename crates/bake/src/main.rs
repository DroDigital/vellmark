use std::path::PathBuf;
use std::process::ExitCode;
use vellmark_bake::{BakeInputs, bake_to_dir};

fn main() -> ExitCode {
    let mut assets = PathBuf::from("assets");
    let mut third_party = PathBuf::from("third_party");
    let mut out = PathBuf::from("build/packs");
    let mut cache = Some(PathBuf::from("build/bake-cache"));
    let mut preview: Option<PathBuf> = None;
    let mut terrain_maps: Option<PathBuf> = None;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        let mut value = |name: &str| args.next().unwrap_or_else(|| panic!("{name} needs a path"));
        match arg.as_str() {
            "--assets" => assets = value("--assets").into(),
            "--third-party" => third_party = value("--third-party").into(),
            "--out" => out = value("--out").into(),
            "--no-cache" => cache = None,
            "--preview-atlas" => preview = Some(value("--preview-atlas").into()),
            "--terrain-maps" => terrain_maps = Some(value("--terrain-maps").into()),
            other => {
                eprintln!(
                    "unknown argument `{other}`\nusage: vellmark-bake [--assets DIR] [--third-party DIR] [--out DIR]"
                );
                return ExitCode::from(2);
            }
        }
    }
    if let Some(folder) = terrain_maps {
        return match write_terrain_maps(&assets, &folder) {
            Ok(()) => ExitCode::SUCCESS,
            Err(e) => {
                eprintln!("terrain maps failed: {e}");
                ExitCode::FAILURE
            }
        };
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
    match bake_to_dir(
        &BakeInputs {
            assets,
            third_party,
            cache,
        },
        &out,
    ) {
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
fn preview_atlas(
    third_party: &std::path::Path,
    cache: Option<&std::path::Path>,
    folder: &std::path::Path,
) -> Result<(), String> {
    let bytes = vellmark_bake::glyphs::bake_atlas(third_party, cache).map_err(|e| e.to_string())?;
    let atlas = vellmark_content::GlyphAtlas::from_bytes(&bytes)?;
    std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    for (px, threshold, name) in [
        (8u32, 0.5f32, "8px"),
        (16, 0.5, "16px"),
        (48, 0.5, "48px"),
        (48, 0.35, "48px_heavy"),
        (48, 0.65, "48px_light"),
    ] {
        let (rgba, w, h) = vellmark_bake::glyphs::preview_sheet(&atlas, px, threshold);
        let file = std::fs::File::create(folder.join(format!("atlas_{name}.png")))
            .map_err(|e| e.to_string())?;
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), w, h);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .and_then(|mut w| w.write_image_data(&rgba))
            .map_err(|e| e.to_string())?;
    }
    Ok(())
}

/// Writes a shaded map of each level of the terrain, north up, to look at.
fn write_terrain_maps(assets: &std::path::Path, folder: &std::path::Path) -> Result<(), String> {
    let baked = vellmark_bake::terrain::generate(&assets.join("terrain/valley.ron"))
        .map_err(|e| e.to_string())?;
    std::fs::create_dir_all(folder).map_err(|e| e.to_string())?;
    for (level, half, heights, water, ground) in &baked.grids {
        let side = vellmark_bake::terrain::GRID_SAMPLES as u32;
        let rgba = vellmark_bake::terrain::map_image(
            half * 2.0 / (side - 1) as f32,
            heights,
            water,
            ground,
        );
        let path = folder.join(format!("terrain_level{level}.png"));
        let file = std::fs::File::create(&path).map_err(|e| e.to_string())?;
        let mut encoder = png::Encoder::new(std::io::BufWriter::new(file), side, side);
        encoder.set_color(png::ColorType::Rgba);
        encoder.set_depth(png::BitDepth::Eight);
        encoder
            .write_header()
            .and_then(|mut w| w.write_image_data(&rgba))
            .map_err(|e| e.to_string())?;
        println!("{}", path.display());
    }
    Ok(())
}
