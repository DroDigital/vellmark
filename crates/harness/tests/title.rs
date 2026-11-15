//! The title sequence over the valley, through the real renderer: black with one
//! enormous character; its strokes made of letters, and those of letters; the ink
//! thinning to the dawn sky; the letters parting round the valley; the title assembled;
//! and the call to begin. Saved as a contact sheet for the evidence.

use vellmark_content::{AssetId, AssetKind, CameraPath, StringTable};
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_harness::image::{contact_sheet, Image};
use vellmark_render::{Camera, FrameInput, LoadedAssets, ShaderLibrary, View};
use vellmark_ui::title::{Title, TitleState, READY};
use vellmark_ui::Canvas;

fn mean(image: &Image) -> f32 {
    image.rgba.chunks(4).map(|p| (p[0] as f32 + p[1] as f32 + p[2] as f32) / 3.0).sum::<f32>() / (image.width * image.height) as f32
}

#[test]
fn the_title_dives_through_letters_into_the_valley() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    let assets = LoadedAssets::from_pack(&pack).unwrap();
    let path = CameraPath::from_bytes(pack.get(AssetKind::CameraPath, AssetId::from_key("path.title_valley")).expect("the title's path")).unwrap();
    assert!((path.duration() - 24.0).abs() < 0.01, "the camera reaches the player's place as the title is up");
    let (w, h) = (640u32, 400u32);
    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), w, h, pack, Some("scene.valley")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    let mut title = Title::new(&assets.glyphs, &strings, false);
    let shots = [0.6, 2.5, 5.5, 8.5, 11.0, 13.5, 15.5, 17.5, 20.5, 22.5, 26.0];
    let mut frames = Vec::new();
    let mut most_quads = 0;
    for at in shots {
        while title.seconds() < at {
            assert_eq!(title.update(1.0 / 30.0), TitleState::Running);
        }
        let mut canvas = Canvas::new();
        title.draw(&mut canvas, &assets.glyphs, &strings, (w, h), 1.0);
        most_quads = most_quads.max(canvas.quads.len());
        let key = path.at(at).unwrap();
        let views = [View::full(Camera::looking(key.eye.into(), key.yaw, key.pitch))];
        let input = FrameInput {
            views: &views,
            time_of_day: key.hour,
            seconds: at,
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
        frames.push((at, image));
    }
    // The opening is dark but for the letters; the end shows the valley.
    let first = mean(&frames[0].1);
    let last = mean(&frames.last().unwrap().1);
    assert!(first < 60.0, "black with one enormous character: mean {first}");
    assert!(last > 70.0, "the valley at dawn: mean {last}");
    assert!(most_quads < 40_000, "{most_quads} quads at most");
    // Every push frame has letters in it: the dive never runs out of ink.
    for (at, image) in &frames[1..6] {
        let bright = image.rgba.chunks(4).filter(|p| p[0] > 150).count();
        assert!(bright > 200, "letters on screen at {at} s: {bright} bright pixels");
    }
    let out = vellmark_harness::repo_root().join("build/captures/title");
    std::fs::create_dir_all(&out).unwrap();
    for (at, image) in &frames {
        image.save(&out.join(format!("title_{:04.1}.png", at))).unwrap();
    }
    let images: Vec<Image> = frames.into_iter().map(|(_, i)| i).collect();
    contact_sheet(&images, 4).save(&out.join("sheet.png")).unwrap();

    // A press before the title is up brings it at once; a press after leaves.
    title.press();
    assert_eq!(title.update(0.6), TitleState::Leaving);
    assert_eq!(title.update(0.7), TitleState::Done);
    let mut early = Title::new(&assets.glyphs, &strings, false);
    early.update(3.0);
    early.press();
    assert!(early.seconds() > 20.0 && early.seconds() < READY);
}

#[test]
fn with_reduced_motion_the_title_is_a_fade() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    let assets = LoadedAssets::from_pack(&pack).unwrap();
    let mut title = Title::new(&assets.glyphs, &strings, true);
    let mut most = 0;
    for _ in 0..240 {
        title.update(1.0 / 30.0);
        let mut canvas = Canvas::new();
        title.draw(&mut canvas, &assets.glyphs, &strings, (1280, 800), 1.0);
        most = most.max(canvas.quads.len());
    }
    assert!(most < 40, "no dive, no smoke: {most} quads at most");
    assert!(title.reveal() > 0.99);
}
