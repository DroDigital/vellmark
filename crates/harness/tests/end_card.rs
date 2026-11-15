//! The end card, drawn through the real renderer over the valley at night: the name, what
//! the full game will be, and the choices. Every line is written and none is under 12 px
//! at 1280x800. The owner's links are empty until given, so they are not offered. Saved
//! as evidence.

use vellmark_content::{sid, AssetKind, StringTable};
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_render::{Camera, FrameInput, LoadedAssets, ShaderLibrary, View};
use vellmark_ui::end_card::{choices, Choice, EndCard, EndCardView};
use vellmark_ui::{Canvas, Shown};

#[test]
fn the_end_card_is_written_and_drawn() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    let assets = LoadedAssets::from_pack(&pack).unwrap();
    for key in ["end.line", "end.more", "end.thanks", "end.keep", "end.wishlist", "end.feedback", "end.again", "end.opened", "end.no_browser", "end.link.wishlist", "end.link.feedback", "ui.menu.start_again"] {
        assert!(strings.get(vellmark_content::StringId::from_key(key)).is_some(), "no string {key}");
    }
    let offered = choices(&strings);
    assert_eq!(offered.first(), Some(&Choice::KeepPlaying));
    assert_eq!(offered.last(), Some(&Choice::StartAgain));

    let (w, h) = (1280u32, 800u32);
    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), w, h, pack, Some("scene.valley")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    let card = EndCard { since: 3.0, ..Default::default() };
    let mut canvas = Canvas::new();
    let hint = Shown::from_table(&strings, sid!("title.press"));
    EndCardView { atlas: &assets.glyphs, strings: &strings, size: (w, h), density: 1.0 }.draw(&mut canvas, &card, Some(hint));
    let glyphs: Vec<_> = canvas.quads.iter().filter(|q| q.layer != vellmark_ui::Quad::SOLID).collect();
    assert!(glyphs.len() > 150, "the card is written ({} glyphs)", glyphs.len());
    let mut twelve = Canvas::new();
    twelve.text(&assets.glyphs, 0.0, 20.0, &vellmark_ui::TextStyle { face: vellmark_content::Face::Small, size: 12.0, colour: [1.0; 4] }, &vellmark_ui::dev_text("abcdefghijklmnopqrstuvwxyz.,"));
    let floor = twelve.quads.iter().map(|q| q.rect[3]).fold(f32::MAX, f32::min);
    let cell = glyphs.iter().map(|q| q.rect[3]).fold(f32::MAX, f32::min);
    assert!(cell >= floor - 0.01, "text smaller than 12 px: {cell} against {floor}");
    let views = [View::full(Camera::looking((0.0, 52.0, 30.0).into(), 0.3, -0.05))];
    let input = FrameInput {
        views: &views,
        time_of_day: 21.5,
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
    let out = vellmark_harness::repo_root().join("build/captures/end_card");
    std::fs::create_dir_all(&out).unwrap();
    offscreen.frame(&input).save(&out.join("end_card.png")).unwrap();
}
