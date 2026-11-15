//! The Line in the real valley: the road from the hill to the village is baked with the
//! terrain, Obelus's way down it never goes into the ground, and in the world it comes,
//! passes a player walking down the road at night, wounds them, and asks for the end card.

use glam::Vec3;
use vellmark_content::{AssetId, AssetKind, RoadTable, ROADS_KEY};
use vellmark_harness::capture::baked_base_pack;
use vellmark_sim::line::{State, END_EVENT};
use vellmark_sim::{PlayerInput, World, WorldConfig, STEP_HZ};

#[test]
fn obelus_comes_down_the_valley_road() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let terrain = std::sync::Arc::new(vellmark_world::Terrain::from_pack(&pack).unwrap());
    let roads = RoadTable::from_bytes(pack.get(AssetKind::Table, AssetId::from_key(ROADS_KEY)).expect("the roads are baked")).unwrap();
    let road = roads.road("penance_road").expect("the road from the hill to the village");
    assert!(road.points.len() >= 2);
    let flown = vellmark_sim::line::road_over(&road.points, |x, z| terrain.height_at(x, z));

    // A player halfway down, facing the village.
    let mid = flown[flown.len() / 2];
    let toward = flown[flown.len() - 1] - mid;
    let yaw = toward.x.atan2(-toward.z);
    let mut path = vellmark_sim::line::flight(&flown, mid, Vec3::new(yaw.sin(), 0.0, -yaw.cos()));
    vellmark_sim::line::clear_ground(&mut path, |x, z| terrain.height_at(x, z));
    for w in path.windows(2) {
        for k in 0..=40 {
            let at = w[0].lerp(w[1], k as f32 / 40.0);
            if let Some(g) = terrain.height_at(at.x, at.z) {
                assert!(at.y > g + 0.5, "into the ground at {at:?} (ground {g})");
            }
        }
    }

    let mut w = World::new(WorldConfig { start_time_of_day: 22.5, hours_per_second: 0.0, ..Default::default() });
    w.set_ground(terrain.clone());
    w.add_player(Vec3::new(mid.x, mid.y - 1.5 + 0.1, mid.z), yaw);
    w.set_line_road(flown);
    w.begin_line();
    let mut asked = 0;
    for _ in 0..(150.0 * STEP_HZ as f32) as u32 {
        w.step(&[PlayerInput::default()]);
        asked += w.events().iter().filter(|(_, e)| e == END_EVENT).count();
    }
    assert_eq!(*w.line(), State::Passed);
    assert_eq!(asked, 1, "the end card, once");
    let p = &w.snapshot().players[0];
    assert!(p.health < 100.0 && p.health > 0.0, "caught standing on the road: {}", p.health);
}
