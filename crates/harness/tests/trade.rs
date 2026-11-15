//! The trade screen, drawn through the real renderer over the yard: a merchant's stock
//! from the real loot table beside the pack, with prices, purses, and each refusal in
//! plain words. Trades are made through `World` with the real tables, and the screen is
//! drawn after each. No text is smaller than the brief's 12 px at 1280x800. Saved as
//! evidence.

use glam::Vec3;
use vellmark_content::{AssetId, AssetKind, LootTable, StringId, StringTable, LOOT_KEY};
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_render::{Camera, FrameInput, LoadedAssets, ShaderLibrary, View};
use vellmark_sim::items::Item;
use vellmark_sim::merchant::Refusal;
use vellmark_sim::{World, WorldConfig};
use vellmark_ui::trade_view::{Side, Trade, TradeView};
use vellmark_ui::Canvas;

const MERCHANT: u64 = 0x70b1;

#[test]
fn every_refusal_and_heading_is_written() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    for key in ["theirs", "yours", "price", "buy", "sell", "leave", "too_dear", "purse_empty", "in_use", "nothing"] {
        let key = format!("ui.trade.{key}");
        assert!(strings.get(StringId::from_key(&key)).is_some(), "no string {key}");
    }
}

#[test]
fn trading_moves_items_and_marks_and_the_screen_shows_it() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let strings = StringTable::from_bytes(pack.get(AssetKind::StringTable, vellmark_bake::string_table_id(vellmark_bake::BASE_LANGUAGE)).unwrap()).unwrap();
    let loot = LootTable::from_bytes(pack.get(AssetKind::Table, AssetId::from_key(LOOT_KEY)).unwrap()).unwrap();
    let assets = LoadedAssets::from_pack(&pack).unwrap();

    let mut w = World::new(WorldConfig::default());
    w.add_player(Vec3::ZERO, 0.0);
    w.set_loot(loot.clone());
    w.add_merchant(MERCHANT);
    let me = w.snapshot().players[0].id;
    w.give_item(me, Item::plain(&loot, "sword", "iron", 1).unwrap());
    w.give_item(me, Item::canonical(&loot, "wicks_spade", 2).unwrap());
    let stock = w.merchant(MERCHANT).unwrap().stock.len();
    assert!(stock > 0, "the merchant has stock from the real table");

    // Too dear with nothing in the purse; then bought, with Marks given.
    let mut trade = Trade { merchant: MERCHANT, ..Default::default() };
    trade.refused = w.buy(me, MERCHANT, 0).err();
    assert_eq!(trade.refused, Some(Refusal::TooDear));
    let refused = trade.clone();
    let price = w.merchant(MERCHANT).unwrap().asks(&loot, &w.merchant(MERCHANT).unwrap().stock[0]);
    w.give_marks(me, price + 40);
    assert_eq!(w.buy(me, MERCHANT, 0), Ok(()));
    assert_eq!(w.inventory(me).unwrap().marks, 40);
    assert_eq!(w.merchant(MERCHANT).unwrap().stock.len(), stock - 1);
    // Sold back: the sword, for what the merchant pays.
    let pays = w.merchant(MERCHANT).unwrap().pays(&loot, &w.inventory(me).unwrap().items[0]);
    assert_eq!(w.sell(me, MERCHANT, 0), Ok(()));
    assert_eq!(w.inventory(me).unwrap().marks, 40 + pays);
    let sold = Trade { merchant: MERCHANT, side: Side::Yours, ..Default::default() };

    let (width, height) = (1280u32, 800u32);
    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), width, height, pack, Some("scene.yard")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    let view = TradeView { atlas: &assets.glyphs, strings: &strings, loot: &loot, size: (width, height), density: 1.0 };
    assert!(view.rows() >= 8, "each side shows at least eight rows at 1280x800: {}", view.rows());
    let out = vellmark_harness::repo_root().join("build/captures/trade");
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
    let merchant = w.merchant(MERCHANT).unwrap();
    let inv = w.inventory(me).unwrap();
    for (name, t) in [("trade_refused", &refused), ("trade_sold", &sold)] {
        let mut canvas = Canvas::new();
        view.draw(&mut canvas, t, merchant, inv, (Some("prompt.key.e"), Some("prompt.key.escape")));
        let text = canvas.quads.iter().filter(|q| q.layer != vellmark_ui::Quad::SOLID).count();
        assert!(text > 200, "{name}: the pages are written ({text} glyphs)");
        let cell = canvas.quads.iter().filter(|q| q.layer != vellmark_ui::Quad::SOLID).map(|q| q.rect[3]).fold(f32::MAX, f32::min);
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
