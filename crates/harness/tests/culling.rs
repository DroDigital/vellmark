//! Culling hides what a view cannot see and nothing else: what is off screen is never
//! drawn, what lies wholly behind a wall stops being drawn once the depth pyramid holds
//! the wall, and what is in view is always drawn.

use std::path::Path;

use vellmark_content::{AssetId, Pack};
use vellmark_harness::capture::Offscreen;
use vellmark_render::{Camera, FrameInput, ShaderLibrary, View};

/// A scene of its own: a wall, a crate hidden behind it, and a crate beside it.
const SCENE: &str = r#"#![enable(implicit_some)]
(
    spawn: (eye: (0.0, 1.7, 6.0), yaw: 0.0, pitch: 0.0),
    boxes: [
        (material: "earth", min: (-30.0, -0.5, -30.0), max: (30.0, 0.0, 30.0)),
        (material: "masonry", min: (-4.0, 0.0, -0.3), max: (4.0, 4.0, 0.3)),
    ],
    objects: [
        (key: "hidden", position: (0.0, 0.0, -3.0), boxes: [(material: "timber", min: (-0.5, 0.0, -0.5), max: (0.5, 1.0, 0.5))]),
        (key: "beside", position: (6.0, 0.0, -3.0), boxes: [(material: "timber", min: (-0.5, 0.0, -0.5), max: (0.5, 1.0, 0.5))]),
    ],
)
"#;

fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let target = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &target);
        } else {
            std::fs::copy(entry.path(), target).unwrap();
        }
    }
}

fn pack_with_the_scene() -> Pack {
    let root = vellmark_harness::repo_root();
    let assets = std::env::temp_dir().join(format!("vellmark-culling-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&assets);
    copy_tree(&root.join("assets"), &assets);
    // Only the test's scene, so that nothing else stands in the way.
    for file in std::fs::read_dir(assets.join("scenes")).unwrap() {
        std::fs::remove_file(file.unwrap().path()).unwrap();
    }
    std::fs::write(assets.join("scenes/cull.ron"), SCENE).unwrap();
    let inputs = vellmark_bake::BakeInputs {
        assets: assets.clone(),
        third_party: root.join("third_party"),
        cache: Some(root.join("build/bake-cache")),
    };
    let output = vellmark_bake::bake(&inputs).expect("the test's scene bakes");
    let _ = std::fs::remove_dir_all(&assets);
    Pack::from_bytes(output.packs.into_iter().next().unwrap().1).unwrap()
}

fn drawn(offscreen: &mut Offscreen, eye: (f32, f32, f32), yaw: f32) -> (u32, u32) {
    let views = [View::full(Camera::looking(
        eye.into(),
        yaw.to_radians(),
        0.0,
    ))];
    let frame = FrameInput {
        views: &views,
        time_of_day: 13.0,
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
        ui: None,
    };
    offscreen.frame(&frame);
    let counts = offscreen.renderer.drawn_last_frame(&offscreen.gpu);
    let of = |name: &str| {
        counts
            .iter()
            .find(|(k, _)| *k == AssetId::from_key(&format!("scene.cull.{name}")))
            .map_or(99, |(_, n)| *n)
    };
    (of("hidden"), of("beside"))
}

#[test]
fn culling_hides_only_what_cannot_be_seen() {
    let pack = pack_with_the_scene();
    let mut offscreen = match Offscreen::with_pack(
        &Default::default(),
        &ShaderLibrary::embedded(),
        640,
        360,
        pack,
        Some("scene.cull"),
    ) {
        Ok(o) => o,
        Err(e)
            if e.contains("no usable graphics adapter")
                && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() =>
        {
            return eprintln!("skipped: {e}");
        }
        Err(e) => panic!("{e}"),
    };
    // Looking at the wall from the south. The first frame has no pyramid yet: both crates
    // are in the frustum, so both are drawn.
    let front = (1.0, 1.7, 6.0);
    assert_eq!(
        drawn(&mut offscreen, front, 0.0),
        (1, 1),
        "before there is a pyramid, everything in view is drawn"
    );
    // The second frame culls against the first frame's depth: the crate behind the wall
    // is hidden, the one beside it is not.
    assert_eq!(
        drawn(&mut offscreen, front, 0.0),
        (0, 1),
        "the crate behind the wall is culled; the one in sight is not"
    );
    // Turned round, both are out of the frustum. The camera cut means only the frustum is
    // trusted, and that is enough.
    assert_eq!(
        drawn(&mut offscreen, front, 180.0),
        (0, 0),
        "nothing behind the eye is drawn"
    );
    // Walking round the end of the wall, the hidden crate comes into sight: it must be
    // drawn the moment it is seen, though last frame's pyramid hid it.
    assert_eq!(
        drawn(&mut offscreen, (-7.0, 1.7, -3.0), 90.0).0,
        1,
        "a crate in sight after a cut is drawn"
    );
}
