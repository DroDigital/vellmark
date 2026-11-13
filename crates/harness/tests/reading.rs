//! Reading as it is seen: the prompt when the yard's wall is in focus, and reading mode
//! over it, the ledger large on paper with a hidden line beneath. Saved as evidence.

use glam::Vec3;
use vellmark_content::{AssetKind, StringId, StringTable};
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_render::{Camera, FrameInput, LoadedAssets, ShaderLibrary, View};
use vellmark_sim::reading::ReadingSnap;
use vellmark_sim::{PlayerId, PlayerSnap};
use vellmark_ui::reading_view::ReadingView;
use vellmark_ui::Canvas;

fn player(reading: ReadingSnap, in_focus: Option<u64>) -> PlayerSnap {
    PlayerSnap {
        id: PlayerId(0),
        feet: Vec3::ZERO,
        yaw: 0.0,
        pitch: 0.0,
        grounded: true,
        crouching: false,
        swimming: false,
        untouchable: false,
        reading,
        in_focus,
        ink: 100.0,
        holding: [vellmark_sim::casting::Holding::Weapon, vellmark_sim::casting::Holding::Empty],
        charge: [0.0; 3],
        preview: [None, None],
    }
}

#[test]
fn reading_is_shown_as_a_prompt_and_as_the_text_on_paper() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    let assets = LoadedAssets::from_pack(&pack).unwrap();
    let readables = vellmark_render::scene_readables(&pack, "scene.yard");
    let wall = readables.iter().find(|r| r.text == StringId::from_key("wall.yard.ledger").0).expect("the ledger is readable");
    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), 960, 540, pack, Some("scene.yard")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    let shots = [
        ("prompt", player(ReadingSnap::None, Some(wall.id))),
        ("reading", player(ReadingSnap::Text { id: wall.id, text: wall.text, hidden: vec![StringId::from_key("wall.yard.bench").0] }, Some(wall.id))),
    ];
    let out = vellmark_harness::repo_root().join("build/captures/reading");
    std::fs::create_dir_all(&out).unwrap();
    let mut frames = Vec::new();
    for (name, p) in shots {
        let mut canvas = Canvas::new();
        ReadingView { atlas: &assets.glyphs, strings: &strings, size: (960, 540), density: 1.0 }.draw(&mut canvas, &p, &Default::default(), &Default::default(), Default::default());
        assert!(!canvas.quads.is_empty(), "{name} draws something");
        let views = [View::full(Camera::looking((0.0, 1.68, 1.5).into(), 0.0, 0.05))];
        let input = FrameInput {
            views: &views,
            time_of_day: 15.0,
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
        let image = offscreen.frame(&input);
        image.save(&out.join(format!("{name}.png"))).unwrap();
        frames.push(image);
    }
    // Reading mode covers the middle of the view with paper.
    let at = |img: &vellmark_harness::image::Image, x: u32, y: u32| {
        let i = ((y * img.width + x) * 4) as usize;
        [img.rgba[i], img.rgba[i + 1], img.rgba[i + 2]]
    };
    let paper = at(&frames[1], 150, 270);
    assert!(paper[0] > 170 && paper[1] > 160, "paper behind the text: {paper:?}");
}
