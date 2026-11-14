//! Dialogue from the baked pack: every scripted line is the voice table's text exactly;
//! Wick's grave and return run through `World::step` as the script has them, with the
//! name's event, the flags, and his twelve Marks; and the conversation is drawn over the
//! yard, as a line and as choices, for the evidence.

use glam::Vec3;
use vellmark_content::{AssetId, AssetKind, DialogueTable, StringId, StringTable, DIALOGUE_KEY};
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_render::{Camera, FrameInput, LoadedAssets, ShaderLibrary, View};
use vellmark_sim::dialogue::Talker;
use vellmark_sim::{buttons, PlayerInput, World, WorldConfig};
use vellmark_ui::dialogue_view::DialogueView;
use vellmark_ui::kit::Focus;
use vellmark_ui::Canvas;

fn world(table: DialogueTable, players: usize) -> World {
    let mut w = World::new(WorldConfig::default());
    for k in 0..players {
        w.add_player(Vec3::new(k as f32 * 0.5, 0.0, 0.0), 0.0);
    }
    w.set_dialogue(table);
    w.add_talker(Talker { id: 1, speaker: "wick".into(), feet: Vec3::new(0.0, 0.0, -1.8), height: 1.75 });
    w
}

fn step(w: &mut World, input: PlayerInput) {
    let n = w.snapshot().players.len();
    let mut all = vec![PlayerInput::default(); n];
    all[0] = input;
    w.step(&all);
    w.step(&vec![PlayerInput::default(); n]);
}

fn line(w: &World) -> Option<String> {
    w.snapshot().players[0].talking.as_ref().and_then(|t| t.line).map(|l| format!("{l:016x}"))
}

fn key(k: &str) -> String {
    format!("{:016x}", StringId::from_key(k).0)
}

#[test]
fn the_scripted_lines_are_the_voice_tables_exactly() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    let tsv = std::fs::read_to_string(vellmark_harness::repo_root().join("full-game/docs/voice/voice_lines.tsv")).unwrap();
    let mut checked = 0;
    for row in tsv.lines().skip(1) {
        let cols: Vec<&str> = row.split('\t').collect();
        let (id, text) = (cols[0], cols[6]);
        if let Some(ours) = strings.get(StringId::from_key(&format!("line.{}", id.to_ascii_lowercase()))) {
            assert_eq!(ours, text, "line.{id} differs from the script");
            checked += 1;
        }
    }
    assert!(checked >= 19, "{checked} lines checked");
}

#[test]
fn wicks_grave_and_return_run_as_the_script_has_them() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let table = DialogueTable::from_bytes(pack.get(AssetKind::Table, AssetId::from_key(DIALOGUE_KEY)).unwrap()).unwrap();
    let mut w = world(table.clone(), 1);
    let interact = PlayerInput { buttons: buttons::INTERACT, ..Default::default() };
    step(&mut w, interact);
    assert_eq!(line(&w), Some(key("line.q1_001")));
    step(&mut w, interact);
    assert_eq!(line(&w), Some(key("line.q1_002")));
    step(&mut w, interact);
    assert_eq!(w.snapshot().players[0].talking.as_ref().unwrap().choices.len(), 3);
    step(&mut w, PlayerInput { say: 2, ..Default::default() });
    assert_eq!(line(&w), Some(key("line.q1_004")), "Who are you? Wick answers");
    step(&mut w, interact);
    assert_eq!(line(&w), Some(key("line.q1_006")));
    step(&mut w, interact);
    step(&mut w, interact);
    // The name: it fades, twice, and Wick gives up on the pencil.
    step(&mut w, PlayerInput { say: 1, ..Default::default() });
    assert_eq!(line(&w), Some(key("line.q1_008")));
    step(&mut w, interact);
    step(&mut w, PlayerInput { say: 1, ..Default::default() });
    assert_eq!(line(&w), Some(key("line.q1_009")));
    for want in ["line.q1_010", "line.q1_011"] {
        step(&mut w, interact);
        assert_eq!(line(&w), Some(key(want)));
    }
    step(&mut w, interact);
    assert!(w.snapshot().players[0].talking.is_none(), "with one player there is no witness line");
    assert!(w.flags().contains("q1_wick_spoke"));
    // Not until the shovel is found does he ask how it went.
    step(&mut w, interact);
    assert!(w.snapshot().players[0].talking.is_none());
    w.set_flag("q1_shovel");
    step(&mut w, interact);
    assert_eq!(line(&w), Some(key("line.q1_014")));
    step(&mut w, interact);
    step(&mut w, PlayerInput { say: 3, ..Default::default() });
    assert_eq!(line(&w), Some(key("line.q1_018")), "the white in the north: every answer leads to the clerk");
    step(&mut w, interact);
    step(&mut w, interact);
    assert_eq!(line(&w), Some(key("line.q1_020")));
    assert_eq!(w.snapshot().players[0].marks, 12, "twelve Marks in a twist of paper");
    step(&mut w, interact);
    assert!(w.flags().contains("q1_complete") && w.flags().contains("wick_met"));

    // With a second player, Wick has the witness line.
    let mut two = world(table, 2);
    for input in [interact, interact, interact, PlayerInput { say: 1, ..Default::default() }, interact, interact, interact, PlayerInput { say: 1, ..Default::default() }, interact, PlayerInput { say: 1, ..Default::default() }, interact, interact, interact] {
        step(&mut two, input);
    }
    assert_eq!(line(&two), Some(key("line.q1_012")), "And there's two of you");
}

#[test]
fn the_conversation_is_drawn_over_the_yard() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    let table = DialogueTable::from_bytes(pack.get(AssetKind::Table, AssetId::from_key(DIALOGUE_KEY)).unwrap()).unwrap();
    let assets = LoadedAssets::from_pack(&pack).unwrap();
    let (w, h) = (1280u32, 800u32);
    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), w, h, pack, Some("scene.yard")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    let mut world = world(table, 1);
    let interact = PlayerInput { buttons: buttons::INTERACT, ..Default::default() };
    step(&mut world, interact);
    step(&mut world, interact);
    let speaking = world.snapshot().players[0].talking.clone().unwrap();
    step(&mut world, interact);
    let choosing = world.snapshot().players[0].talking.clone().unwrap();
    let out = vellmark_harness::repo_root().join("build/captures/dialogue");
    std::fs::create_dir_all(&out).unwrap();
    let view = DialogueView { atlas: &assets.glyphs, strings: &strings, size: (w, h), density: 1.0 };
    for (name, talk, now, focus) in [("line", &speaking, speaking.since + 0.6, Focus::default()), ("choices", &choosing, choosing.since + 5.0, Focus { at: 1, scroll: 0 })] {
        let mut canvas = Canvas::new();
        view.draw(&mut canvas, talk, now, focus, Some("prompt.key.e"));
        assert!(!canvas.quads.is_empty());
        let views = [View::full(Camera::looking((0.0, 1.68, 2.5).into(), 0.15, 0.02))];
        let input = FrameInput {
            views: &views,
            time_of_day: 17.0,
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
        offscreen.frame(&input).save(&out.join(format!("{name}.png"))).unwrap();
    }
    // The offer to talk, as the crosshair would show it.
    let mut canvas = Canvas::new();
    view.draw_offer(&mut canvas, "wick", "prompt.key.e");
    assert!(!canvas.quads.is_empty());
}
