//! The pack, drawn through the real renderer over the yard: a list of what is carried with
//! the equipped marked, and the focused item's page. Every line it can show for the
//! Showcase's loot is written, and no text on it is smaller than the brief's 12 px at
//! 1280x800. Saved as evidence.

use vellmark_content::{AssetId, AssetKind, LootTable, StringId, StringTable, LOOT_KEY};
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_render::{Camera, FrameInput, LoadedAssets, ShaderLibrary, View};
use vellmark_sim::inventory::Inventory;
use vellmark_sim::items::{generate, Item};
use vellmark_ui::inventory_view::{rows_shown, InventoryView};
use vellmark_ui::kit::Focus;
use vellmark_ui::{item_text, Canvas};

fn strings_and_loot(pack: &vellmark_content::Pack) -> (StringTable, LootTable) {
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    let loot = LootTable::from_bytes(pack.get(AssetKind::Table, AssetId::from_key(LOOT_KEY)).unwrap()).unwrap();
    (strings, loot)
}

fn missing(text: &str) -> bool {
    text.contains("<") && text.contains(">") && text.len() >= 18
}

#[test]
fn every_line_an_item_can_show_is_written() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let (strings, loot) = strings_and_loot(&pack);
    let effects = loot.bonuses.iter().chain(&loot.clauses).map(|m| m.effect.as_str()).chain(loot.word_clauses.iter().map(|c| c.effect.as_str())).chain(loot.canonical.iter().map(|c| c.effect.as_str()));
    for effect in effects {
        let key = format!("item.effect.{effect}");
        assert!(strings.get(StringId::from_key(&key)).is_some(), "no string {key}");
    }
    for source in ["chest", "boss", "annotated", "merchant"] {
        for seed in 0..400 {
            let item = generate(&loot, source, seed, &["COLD".into()]).unwrap();
            let mut lines = vec![item_text::label(&strings, &loot, &item), item_text::tier(&strings, &item), item_text::trait_line(&strings, &loot, &item)];
            lines.extend(item_text::effects(&strings, &loot, &item));
            for line in lines {
                assert!(!missing(line.as_str()), "{source} {seed}: {}", line.as_str());
                assert!(!line.as_str().contains('{'), "an unfilled place: {}", line.as_str());
            }
        }
    }
}

#[test]
fn the_pack_lists_what_is_carried_and_pages_the_focused_item() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let (strings, loot) = strings_and_loot(&pack);
    let assets = LoadedAssets::from_pack(&pack).unwrap();
    let mut inv = Inventory { marks: 137, ..Default::default() };
    inv.take(Item::plain(&loot, "sword", "iron", 1).unwrap());
    inv.take(Item::canonical(&loot, "wicks_spade", 2).unwrap());
    inv.take(Item::canonical(&loot, "promised_lamp", 3).unwrap());
    // A spread of what the boss and chests give, so that every tier is on the list.
    let mut seen = [false; 5];
    for seed in 0..4000u64 {
        let item = generate(&loot, if seed % 2 == 0 { "boss" } else { "chest" }, seed, &["COLD".into()]).unwrap();
        if !seen[item.tier as usize] && (item.tier as usize != 4 || item.word_clause.is_some()) {
            seen[item.tier as usize] = true;
            inv.take(item);
        }
    }
    for seed in 0..10u64 {
        inv.take(generate(&loot, "enemy", 900 + seed, &[]).unwrap());
    }
    inv.equip(&loot, 0);
    inv.equip(&loot, 2);
    let (w, h) = (1280u32, 800u32);
    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), w, h, pack, Some("scene.yard")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    let rows = rows_shown(&assets.glyphs, (w, h), 1.0);
    assert!(rows >= 10, "the list shows at least ten rows at 1280x800: {rows}");
    let out = vellmark_harness::repo_root().join("build/captures/inventory");
    std::fs::create_dir_all(&out).unwrap();
    // The focus on the Illuminated item, with its word clause; then scrolled to the foot.
    let illuminated = inv.items.iter().position(|i| i.tier as u8 == 4).expect("an Illuminated item");
    let mut focus = Focus::default();
    focus.step(illuminated as i32, inv.items.len(), rows);
    let mut foot = focus;
    foot.step(inv.items.len() as i32, inv.items.len(), rows);
    assert_eq!(foot.at, inv.items.len() - 1);
    for (name, focus) in [("pack", focus), ("pack_scrolled", foot)] {
        let mut canvas = Canvas::new();
        InventoryView { atlas: &assets.glyphs, strings: &strings, loot: &loot, size: (w, h), density: 1.0 }.draw(&mut canvas, &inv, focus, (Some("prompt.key.e"), Some("prompt.key.tab")));
        // Every glyph is at least 12 px tall, as the brief asks of menus at 1280x800.
        // A glyph's cell grows with its size, so the smallest cell is measured against the
        // cells of the same letters set at 12 px in each face.
        let cell = |c: &Canvas| c.quads.iter().filter(|q| q.layer != vellmark_ui::Quad::SOLID).map(|q| q.rect[3]).fold(f32::MAX, f32::min);
        let mut floor = f32::MAX;
        for face in [vellmark_content::Face::Small, vellmark_content::Face::Large] {
            let mut twelve = Canvas::new();
            twelve.text(&assets.glyphs, 0.0, 20.0, &vellmark_ui::TextStyle { face, size: 12.0, colour: [1.0; 4] }, &vellmark_ui::dev_text(" !\"#$%&'()*+,-./0123456789:;<=>?@ABCDEFGHIJKLMNOPQRSTUVWXYZ[\\]^_`abcdefghijklmnopqrstuvwxyz{|}~"));
            floor = floor.min(cell(&twelve));
        }
        assert!(cell(&canvas) >= floor - 0.01, "text smaller than 12 px: cells {} against {floor}", cell(&canvas));
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
