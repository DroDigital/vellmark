//! A Page that writes as it is watched, through `World::step`: what stands written at the
//! start, the next entry begun as a reader comes near and written at a careful hand's pace,
//! the one after begun a few seconds later, reading showing only what is written, and the
//! finished Page staying written in a saved game.

use glam::Vec3;
use vellmark_content::WriteCue;
use vellmark_sim::reading::{ReadableText, ReadingSnap};
use vellmark_sim::{buttons, PlayerInput, World, WorldConfig, STEP_HZ};

fn stone() -> ReadableText {
    ReadableText {
        id: 9,
        text: 90,
        min: Vec3::new(-0.6, 0.0, -10.2),
        max: Vec3::new(0.6, 3.0, -10.0),
        hidden: Vec::new(),
        teaches: String::new(),
        writing: vec![
            WriteCue { upto: 20, near: 0.0, after: 0.0 },
            WriteCue { upto: 40, near: 4.0, after: 0.0 },
            WriteCue { upto: 55, near: 0.0, after: 2.0 },
        ],
    }
}

fn world() -> World {
    let mut w = World::new(WorldConfig::default());
    w.add_player(Vec3::new(0.0, 0.0, 0.0), 0.0);
    w.set_readables(vec![stone()]);
    w
}

fn shown(w: &World) -> u32 {
    w.snapshot().pages_written[0].1
}

fn wait(w: &mut World, seconds: f32) {
    for _ in 0..(seconds * STEP_HZ as f32) as u32 {
        w.step(&[PlayerInput::default()]);
    }
}

#[test]
fn a_page_writes_its_entries_as_it_is_watched() {
    let mut w = world();
    wait(&mut w, 5.0);
    assert_eq!(shown(&w), 20, "from ten metres, only what stood written");
    let me = w.snapshot().players[0].id;
    w.place_player(me, Vec3::new(0.0, 0.0, -8.0), 0.0);
    wait(&mut w, 1.0);
    let part = shown(&w);
    assert!(part > 20 && part < 40, "the next entry writing itself, at a hand's pace: {part}");
    // Reading shows only what is written.
    w.step(&[PlayerInput { buttons: buttons::READ, ..Default::default() }]);
    match &w.snapshot().players[0].reading {
        ReadingSnap::Text { written: Some(n), .. } => assert!(*n > 20 && *n < 40),
        other => panic!("reading the Page: {other:?}"),
    }
    wait(&mut w, 1.5);
    assert_eq!(shown(&w), 40);
    wait(&mut w, 2.0);
    assert!(shown(&w) > 40, "the last entry, a moment after");
    wait(&mut w, 2.0);
    assert_eq!(shown(&w), 55);
    // Finished, it stays written in a saved game.
    let bytes = w.save().to_bytes();
    let mut fresh = world();
    fresh.load(&vellmark_sim::save::SaveGame::from_bytes(&bytes).unwrap());
    assert_eq!(shown(&fresh), 55);
    // And a fresh world begins where the Page begins.
    assert_eq!(shown(&world()), 20);
}
