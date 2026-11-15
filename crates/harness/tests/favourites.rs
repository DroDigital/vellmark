//! The favourites wheel, with the real tables: pairings kept from the hands, set round the
//! wheel, aimed at, and taken up at once through `World::step`; drawn through the real
//! renderer over the yard, full and empty. No text is smaller than the brief's 12 px at
//! 1280x800. Saved as evidence.

use glam::Vec3;
use vellmark_content::{AssetId, AssetKind, LootTable, StringId, StringTable, LOOT_KEY};
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_render::{Camera, FrameInput, LoadedAssets, ShaderLibrary, View};
use vellmark_sim::casting::Holding;
use vellmark_sim::inventory::{Favourite, Inventory};
use vellmark_sim::items::Item;
use vellmark_sim::melee::WeaponKind;
use vellmark_sim::{PlayerInput, World, WorldConfig};
use vellmark_ui::favourites_view::{FavouritesView, Wheel};
use vellmark_ui::Canvas;

#[test]
fn pairings_kept_from_the_hands_are_taken_up_from_the_wheel() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    let loot = LootTable::from_bytes(pack.get(AssetKind::Table, AssetId::from_key(LOOT_KEY)).unwrap()).unwrap();
    let assets = LoadedAssets::from_pack(&pack).unwrap();
    let rules = vellmark_render::word_rules(&pack);
    for key in ["title", "none", "bare_hand", "empty_hand", "pointer", "keep", "kept", "forgotten"] {
        let key = format!("ui.favourites.{key}");
        assert!(strings.get(StringId::from_key(&key)).is_some(), "no string {key}");
    }

    let mut w = World::new(WorldConfig::default());
    let me = w.add_player(Vec3::ZERO, 0.0);
    w.set_word_rules(rules.clone());
    w.set_loot(loot.clone());
    let sword = w.give_item(me, Item::plain(&loot, "sword", "iron", 1).unwrap()).unwrap();
    let bow = w.give_item(me, Item::plain(&loot, "bow", "marcher_steel", 2).unwrap()).unwrap();
    let spade = w.give_item(me, Item::canonical(&loot, "wicks_spade", 3).unwrap()).unwrap();
    let shield = w.give_item(me, Item::plain(&loot, "shield", "rootwood", 4).unwrap()).unwrap();
    // The sword with OPEN in the other hand; the bow alone; the spade and the shield.
    w.equip(me, sword);
    w.hold_word(me, 1, "OPEN");
    assert_eq!(w.keep_hands(me), Some(true));
    w.equip(me, bow);
    assert_eq!(w.keep_hands(me), Some(true));
    w.add_favourite(me, Some(spade), Favourite::Item(shield));
    w.hold_word(me, 1, "CALM");
    w.add_favourite(me, Some(sword), Favourite::Word(rules.word("CALM").unwrap()));
    assert_eq!(w.inventory(me).unwrap().favourites.len(), 4);

    // The first is up on the wheel; taking it puts the sword and OPEN back in the hands.
    let mut wheel = Wheel::default();
    wheel.stick(0.0, -1.0);
    assert_eq!(wheel.slot(4), Some(0));
    w.step(&[PlayerInput { favourite: 1, ..Default::default() }]);
    let s = w.snapshot();
    assert_eq!(s.players[0].weapon, WeaponKind::Sword);
    assert_eq!(s.players[0].holding[1], Holding::Word(rules.word("OPEN").unwrap()));
    // The second, the bow, aimed at to the right.
    wheel.stick(1.0, 0.0);
    w.step(&[PlayerInput { favourite: wheel.slot(4).unwrap() as u8 + 1, ..Default::default() }]);
    assert_eq!(w.snapshot().players[0].weapon, WeaponKind::Bow);

    let (width, height) = (1280u32, 800u32);
    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), width, height, pack, Some("scene.yard")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    let view = FavouritesView { atlas: &assets.glyphs, strings: &strings, rules: &rules, loot: &loot, size: (width, height), density: 1.0 };
    let out = vellmark_harness::repo_root().join("build/captures/favourites");
    std::fs::create_dir_all(&out).unwrap();
    let floor = {
        let mut floor = f32::MAX;
        for face in [vellmark_content::Face::Small, vellmark_content::Face::Large] {
            let mut twelve = Canvas::new();
            twelve.text(&assets.glyphs, 0.0, 20.0, &vellmark_ui::TextStyle { face, size: 12.0, colour: [1.0; 4] }, &vellmark_ui::dev_text(" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~"));
            floor = floor.min(twelve.quads.iter().map(|q| q.rect[3]).fold(f32::MAX, f32::min));
        }
        floor
    };
    let empty = Inventory::default();
    let full = w.inventory(me).unwrap();
    for (name, inv) in [("favourites", full), ("favourites_empty", &empty)] {
        let mut canvas = Canvas::new();
        view.draw(&mut canvas, &wheel, inv, Some("prompt.key.c"));
        let glyphs: Vec<_> = canvas.quads.iter().filter(|q| q.layer != vellmark_ui::Quad::SOLID).collect();
        assert!(glyphs.len() > 20, "{name}: the wheel is written ({} glyphs)", glyphs.len());
        let cell = glyphs.iter().map(|q| q.rect[3]).fold(f32::MAX, f32::min);
        assert!(cell >= floor - 0.01, "{name}: text smaller than 12 px: {cell} against {floor}");
        let views = [View::full(Camera::looking((0.0, 1.68, 1.5).into(), 0.4, 0.02))];
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
        offscreen.frame(&input).save(&out.join(format!("{name}.png"))).unwrap();
    }
}
