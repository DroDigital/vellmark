//! The pause menu and the settings pages, drawn through the real renderer over the yard:
//! every page's labels and values are written, nothing is under 12 px at 1280x800, and
//! the pause, controls, and accessibility pages are saved as evidence.

use vellmark_content::{AssetKind, StringTable};
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_platform::prompts::PadStyle;
use vellmark_platform::{Bindings, DeviceKind, Settings};
use vellmark_render::{Camera, FrameInput, LoadedAssets, ShaderLibrary, View};
use vellmark_ui::menu::{MenuInput, MenuView};
use vellmark_ui::pages::{Menu, Page};
use vellmark_ui::{Canvas, Part, Shown};

const DOWN: MenuInput = MenuInput { step: 1, change: 0, confirm: false, back: false, pick: None };
const OK: MenuInput = MenuInput { step: 0, change: 0, confirm: true, back: false, pick: None };
const BACK: MenuInput = MenuInput { step: 0, change: 0, confirm: false, back: true, pick: None };

fn go(menu: &mut Menu, s: &mut Settings, inputs: &[MenuInput]) {
    for i in inputs {
        let len = menu.entries(s, &Bindings::default(), DeviceKind::KeyboardMouse, PadStyle::Xbox).len();
        menu.update(*i, s, len, 12);
    }
}

#[test]
fn every_page_is_written_and_drawn() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    let assets = LoadedAssets::from_pack(&pack).unwrap();
    let (w, h) = (1280u32, 800u32);
    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), w, h, pack, Some("scene.yard")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    let mut s = Settings { preset: Some("medium".into()), master_volume: 0.8, reduced_motion: true, ..Default::default() };
    let out = vellmark_harness::repo_root().join("build/captures/menus");
    std::fs::create_dir_all(&out).unwrap();
    // Every page in turn: pause, settings, controls, buttons, display, sound, accessibility.
    let mut menu = Menu::open();
    let walk: [(&str, Page, &[MenuInput]); 7] = [
        ("pause", Page::Pause, &[]),
        ("settings", Page::Settings, &[DOWN, OK]),
        ("controls", Page::Controls, &[OK, DOWN]),
        ("buttons", Page::Buttons, &[DOWN, DOWN, DOWN, OK]),
        ("display", Page::Display, &[BACK, BACK, DOWN, OK]),
        ("sound", Page::Sound, &[BACK, DOWN, OK]),
        ("access", Page::Access, &[BACK, DOWN, OK, DOWN]),
    ];
    for (name, page, inputs) in walk {
        go(&mut menu, &mut s, inputs);
        assert_eq!(menu.page(), page, "{name}");
        let entries = menu.entries(&s, &s.input.bindings(), DeviceKind::KeyboardMouse, PadStyle::Xbox);
        let mut canvas = Canvas::new();
        let key = |k: &str| Part::Shown(Shown::from_table(&strings, vellmark_content::StringId::from_key(k)));
        let hint = Shown::filled(&strings, vellmark_content::sid!("ui.menu.hint"), &[("confirm", key("prompt.key.enter")), ("back", key("prompt.key.escape"))]);
        let view = MenuView { atlas: &assets.glyphs, strings: &strings, size: (w, h), density: 1.0 };
        assert!(view.rows(entries.len(), &assets.glyphs) >= entries.len(), "{name}: every entry fits on the page");
        view.draw(&mut canvas, menu.title(), &entries, menu.focus(), Some(hint));
        // No string is missing: a missing one is drawn as its id in angle brackets.
        let title = Shown::from_table(&strings, menu.title());
        assert!(!title.as_str().starts_with('<'), "{name}: no title");
        for e in &entries {
            assert!(!Shown::from_table(&strings, e.label).as_str().starts_with('<'), "{name}: an entry has no label");
        }
        // No glyph is smaller than the same letters at 12 px.
        let cell = |c: &Canvas| c.quads.iter().filter(|q| q.layer != vellmark_ui::Quad::SOLID).map(|q| q.rect[3]).fold(f32::MAX, f32::min);
        let mut twelve = Canvas::new();
        for face in [vellmark_content::Face::Small, vellmark_content::Face::Large] {
            twelve.text(&assets.glyphs, 0.0, 20.0, &vellmark_ui::TextStyle { face, size: 12.0, colour: [1.0; 4] }, &vellmark_ui::dev_text("|.>*+-"));
        }
        assert!(cell(&canvas) >= cell(&twelve) - 0.01, "{name}: text under 12 px");
        if ["pause", "controls", "access"].contains(&name) {
            let views = [View::full(Camera::looking((0.0, 1.68, 1.5).into(), -0.5, 0.05))];
            let input = FrameInput {
                views: &views,
                time_of_day: 16.0,
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
    }
    // High contrast and larger letters: the accessibility page still holds every entry.
    vellmark_ui::kit::set_high_contrast(true);
    vellmark_ui::set_letter_scale(1.4);
    let entries = menu.entries(&s, &s.input.bindings(), DeviceKind::KeyboardMouse, PadStyle::Xbox);
    let view = MenuView { atlas: &assets.glyphs, strings: &strings, size: (w, h), density: 1.0 };
    assert!(view.rows(entries.len(), &assets.glyphs) >= entries.len(), "every entry fits at the larger letters");
    let mut canvas = Canvas::new();
    view.draw(&mut canvas, menu.title(), &entries, menu.focus(), None);
    vellmark_ui::kit::set_high_contrast(false);
    vellmark_ui::set_letter_scale(1.0);
    let views = [View::full(Camera::looking((0.0, 1.68, 1.5).into(), -0.5, 0.05))];
    let input = FrameInput {
        views: &views,
        time_of_day: 16.0,
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
    offscreen.frame(&input).save(&out.join("access_high.png")).unwrap();
}

#[test]
fn the_cursor_picks_a_row_and_a_click_confirms_it() {
    use vellmark_platform::{ActionState, MouseButton, RawInput};
    let pack = baked_base_pack().expect("the base pack bakes");
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    let assets = LoadedAssets::from_pack(&pack).unwrap();
    let mut s = Settings::default();
    let mut menu = Menu::open();
    let len = menu.entries(&s, &Bindings::default(), DeviceKind::KeyboardMouse, PadStyle::Xbox).len();
    // At twice the density, as on a Mac's screen: the window's points are half the pixels.
    let view = MenuView { atlas: &assets.glyphs, strings: &strings, size: (2560, 1600), density: 2.0 };
    let middle = |k: usize| -> (f32, f32) {
        // Search a column down the middle of the page for the row's band, in pixels.
        (0..1600).map(|y| (1280.0, y as f32)).find(|p| view.row_at(len, menu.focus(), *p) == Some(k)).expect("the row is on the page")
    };
    let (x, y) = middle(1);
    assert_eq!(view.row_at(len, menu.focus(), (5.0, 5.0)), None, "nothing off the page");
    let bindings = Bindings::default();
    let mut actions = ActionState::default();
    actions.begin_frame();
    let raw = [RawInput::Pointer { x: x * 0.5, y: (y + 4.0) * 0.5 }];
    for r in raw {
        actions.feed(&bindings, r);
    }
    let focus = menu.focus();
    let input = MenuInput::default().with_pointer(&raw, &actions, 2.0, |p| view.row_at(len, focus, p));
    assert_eq!(input.pick, Some(1), "resting on the second row picks it");
    assert!(!input.confirm);
    menu.update(input, &mut s, len, view.rows(len, &assets.glyphs));
    assert_eq!(menu.focus().at, 1);
    assert_eq!(menu.page(), Page::Pause);
    // A click on the row opens it: the second row is the settings.
    actions.begin_frame();
    let raw = [RawInput::MouseButton { button: MouseButton::Left, down: true }];
    let focus = menu.focus();
    let input = MenuInput::default().with_pointer(&raw, &actions, 2.0, |p| view.row_at(len, focus, p));
    assert!(input.confirm, "a click on a row confirms it");
    menu.update(input, &mut s, len, view.rows(len, &assets.glyphs));
    assert_eq!(menu.page(), Page::Settings);
    // The wheel steps.
    let raw = [RawInput::MouseWheel { dy: -1.0 }];
    let input = MenuInput::default().with_pointer(&raw, &actions, 2.0, |_| None);
    assert_eq!(input.step, 1);
}
