//! The masons' crates and keg at the head of the Mended Bridge, in the real valley: they
//! come to rest on its ground at load, neither sunk into it nor floating above it.

use vellmark_content::{AssetId, AssetKind, Scene};
use vellmark_harness::capture::baked_base_pack;
use vellmark_sim::{World, WorldConfig};

#[test]
fn the_masons_props_rest_on_the_valley_ground() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let scene = Scene::from_bytes(
        pack.get(AssetKind::Scene, AssetId::from_key("scene.valley"))
            .expect("the valley"),
    )
    .unwrap();
    let terrain = std::sync::Arc::new(vellmark_world::Terrain::from_pack(&pack).unwrap());
    let mut world = World::new(WorldConfig::default());
    let (solids, breakables) = vellmark_render::scene_rules(&scene);
    world.set_scene(solids, breakables);
    world.set_trunks(vellmark_render::tree_trunks(&pack, &scene));
    world.set_ground(terrain.clone());
    let props = world.snapshot().props.clone();
    assert_eq!(props.len(), 3, "two crates and a keg");
    for key in ["mason_crate", "mason_keg"] {
        let id = AssetId::from_key(&format!("scene.valley.{key}")).0;
        let (_, pose) = props.iter().find(|(p, _)| *p == id).expect(key);
        let at = glam::Vec3::from(pose.translation);
        let ground = terrain.height_at(at.x, at.z).unwrap();
        let up = at.y - ground;
        assert!(
            up > 0.1 && up < 0.45,
            "{key} rests on the ground: its middle is {up} m above it"
        );
    }
    let small = AssetId::from_key("scene.valley.mason_crate_small").0;
    let crate_ = AssetId::from_key("scene.valley.mason_crate").0;
    let top = props
        .iter()
        .find(|(p, _)| *p == small)
        .unwrap()
        .1
        .translation
        .y;
    let under = props
        .iter()
        .find(|(p, _)| *p == crate_)
        .unwrap()
        .1
        .translation
        .y;
    assert!(
        top - under > 0.3,
        "the small crate still stands on the big one: {top} over {under}"
    );
}
