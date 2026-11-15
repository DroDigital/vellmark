//! The credits from the baked pack: both typefaces, and every crate the game is built
//! with and its license, read from the notices; drawn rolling for the evidence.

use vellmark_content::{AssetId, AssetKind, Credits, StringTable, CREDITS_KEY};
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_render::{Camera, FrameInput, LoadedAssets, ShaderLibrary, View};
use vellmark_ui::credits_view::CreditsView;
use vellmark_ui::Canvas;

#[test]
fn the_credits_name_the_typefaces_and_every_crate() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let credits = Credits::from_bytes(pack.get(AssetKind::Table, AssetId::from_key(CREDITS_KEY)).expect("the credits")).unwrap();
    assert_eq!(credits.typefaces.len(), 2);
    let notices = std::fs::read_to_string(vellmark_harness::repo_root().join("THIRD_PARTY_NOTICES.txt")).unwrap();
    assert!(credits.crates.len() > 150, "{} crates", credits.crates.len());
    for (name, _) in &credits.crates {
        assert!(notices.contains(name.as_str()), "{name} is in the notices");
    }
    assert!(credits.crates.iter().any(|(n, l)| n.starts_with("sdl3-src") && l == "Zlib"), "SDL, linked statically, is credited");
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    let assets = LoadedAssets::from_pack(&pack).unwrap();
    let (w, h) = (960u32, 600u32);
    let mut o = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), w, h, pack, Some("scene.yard")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    let view = CreditsView { atlas: &assets.glyphs, strings: &strings, credits: &credits, size: (w, h), density: 1.0 };
    let mut canvas = Canvas::new();
    view.draw(&mut canvas, 420.0);
    assert!(canvas.quads.len() > 100);
    let views = [View::full(Camera::looking((0.0, 1.68, 1.5).into(), 0.0, 0.0))];
    let input = FrameInput {
        views: &views,
        time_of_day: 12.0,
        seconds: 0.0,
        glyph_layer: true,
        close_reading: 0.0,
        dive: 0.0,
        lamps: 1.0,
        tuning: Default::default(),
        reveal_mask: false,
        letters: &[],
        letters_step: 0,
        broken: &[],
        heat: &[],
        weather: Default::default(),
        wind_gust: None,
        relief: Default::default(),
        ui: Some(&canvas),
    };
    let out = vellmark_harness::repo_root().join("build/captures/credits");
    std::fs::create_dir_all(&out).unwrap();
    o.frame(&input).save(&out.join("credits.png")).unwrap();
}
