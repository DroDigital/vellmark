//! Footsteps in the real valley: walked on grass, on the road, and through the barley,
//! each stride is heard, on what the ground there is.

use glam::Vec3;
use vellmark_content::Ground;
use vellmark_harness::capture::baked_base_pack;
use vellmark_sim::{Heard, PlayerInput, World, WorldConfig, STEP_HZ};

#[test]
fn each_stride_is_heard_on_what_is_underfoot() {
    let pack = baked_base_pack().expect("the base pack bakes");
    let terrain = std::sync::Arc::new(vellmark_world::Terrain::from_pack(&pack).unwrap());
    // A place where a few metres north are all of one ground, for each kind wanted.
    let find = |want: Ground| -> Vec3 {
        for i in -60..60 {
            for j in -60..60 {
                let (x, z) = (i as f32 * 4.0, j as f32 * 4.0);
                let all = (0..20).all(|k| terrain.ground_at(x, z - k as f32 * 0.5) == Some(want) && terrain.water_at(x, z - k as f32 * 0.5).is_none());
                if all {
                    return Vec3::new(x, terrain.height_at(x, z).unwrap() + 0.1, z);
                }
            }
        }
        panic!("no stretch of {want:?}");
    };
    for (ground, surface) in [(Ground::Grass, 0u8), (Ground::Road, 2), (Ground::Field, 1)] {
        let mut w = World::new(WorldConfig::default());
        w.set_scene(Vec::new(), Vec::new());
        w.set_ground(terrain.clone());
        w.add_player(find(ground), 0.0);
        w.step(&[PlayerInput::default()]);
        let mut steps = Vec::new();
        for _ in 0..STEP_HZ * 2 {
            w.step(&[PlayerInput { move_y: 1.0, ..Default::default() }]);
            for h in &w.snapshot().heard {
                if let Heard::Step { surface, soft, .. } = h {
                    steps.push((*surface, *soft));
                }
            }
        }
        assert!(steps.len() >= 3 && steps.len() <= 8, "{ground:?}: a stride's pace, {} steps in two seconds", steps.len());
        assert!(steps.iter().all(|(s, soft)| *s == surface && !soft), "{ground:?}: {steps:?}");
    }
}
