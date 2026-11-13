//! A body walks the real valley: over the baked terrain, never sinking into it or
//! floating above it, and stopped by a tree's trunk.

use std::sync::Arc;

use glam::Vec3;
use vellmark_content::{AssetId, AssetKind, Scene};
use vellmark_harness::capture::baked_base_pack;
use vellmark_sim::{buttons, PlayerInput, World, WorldConfig, STEP_HZ};

fn valley() -> (World, Arc<vellmark_world::Terrain>, Vec<vellmark_sim::Trunk>) {
    let pack = baked_base_pack().expect("the base pack bakes");
    let scene = Scene::from_bytes(pack.get(AssetKind::Scene, AssetId::from_key("scene.valley")).expect("the valley")).unwrap();
    let terrain = Arc::new(vellmark_world::Terrain::from_pack(&pack).unwrap());
    let trunks = vellmark_render::tree_trunks(&pack, &scene);
    let mut world = World::new(WorldConfig::default());
    let (solids, breakables) = vellmark_render::scene_rules(&scene);
    world.set_scene(solids, breakables);
    world.set_trunks(trunks.clone());
    world.set_ground(terrain.clone());
    (world, terrain, trunks)
}

#[test]
fn a_body_walks_the_valley_on_its_ground_and_trees_stop_it() {
    let (mut world, terrain, trunks) = valley();
    assert!(trunks.len() > 500, "{} trunks", trunks.len());
    // Down Tithe Hill and across the pasture, turning as it goes.
    let ground = terrain.height_at(-104.0, 112.0).unwrap();
    world.add_player(Vec3::new(-104.0, ground, 112.0), 0.4);
    for i in 0..(20 * STEP_HZ) {
        let input = PlayerInput { move_y: 1.0, look_yaw: if i % 240 < 120 { 0.002 } else { -0.002 }, buttons: buttons::SPRINT, ..Default::default() };
        world.step(&[input]);
        let s = world.snapshot();
        let f = s.players[0].feet;
        let h = terrain.height_at(f.x, f.z).unwrap();
        assert!(f.y > h - 0.15 && f.y < h + 0.6, "step {i}: feet at {f}, ground at {h}");
    }
    // Walking at an oak's trunk, the body never passes through it: it stops, or slides
    // round it, never nearer its centre than the two radii.
    let oak = trunks.iter().find(|t| t.radius > 0.25).expect("an oak");
    let start = Vec3::new(oak.foot.x, terrain.height_at(oak.foot.x, oak.foot.z + 4.0).unwrap(), oak.foot.z + 4.0);
    world.add_player(start, 0.0);
    let mut nearest = f32::MAX;
    for _ in 0..(3 * STEP_HZ) {
        world.step(&[PlayerInput::default(), PlayerInput { move_y: 1.0, ..Default::default() }]);
        let f = world.snapshot().players[1].feet;
        nearest = nearest.min(((f.x - oak.foot.x).powi(2) + (f.z - oak.foot.z).powi(2)).sqrt());
    }
    let touching = oak.radius + vellmark_sim::character::RADIUS;
    assert!(nearest > touching - 0.05, "came within {nearest} m of the trunk's centre; touching is {touching}");
    assert!(nearest < touching + 0.3, "met the trunk: nearest {nearest}");
}
