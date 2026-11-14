//! The first-person hands, as they are drawn: at rest with a sword and a word, mid-swing,
//! the heavy attack drawn back, a shield raised, a word charging, and the bow drawn. In
//! the yard, by day. Saved for the evidence; each pose must put the kit on screen.

use glam::{Quat, Vec3};
use vellmark_content::AssetKind;
use vellmark_harness::capture::{baked_base_pack, Offscreen};
use vellmark_render::viewmodel::ViewModel;
use vellmark_render::{Camera, FrameInput, ShaderLibrary, View};
use vellmark_sim::casting::Holding;
use vellmark_sim::melee::{SwingKind, WeaponKind};
use vellmark_sim::reading::ReadingSnap;
use vellmark_sim::{PlayerId, PlayerSnap};

fn player() -> PlayerSnap {
    PlayerSnap {
        id: PlayerId(0),
        feet: Vec3::new(0.0, 0.0, 2.3),
        yaw: 0.0,
        pitch: 0.0,
        grounded: true,
        crouching: false,
        swimming: false,
        untouchable: false,
        reading: ReadingSnap::None,
        in_focus: None,
        ink: 100.0,
        holding: [Holding::Weapon, Holding::Word(0)],
        charge: [0.0; 3],
        preview: [None, None],
        health: 100.0,
        stamina: 100.0,
        weapon: WeaponKind::Sword,
        swing: None,
        drawing: None,
        quiver: 20,
        on_weapon: Vec::new(),
        blocking: false,
        staggered: false,
        learned: None,
    }
}

#[test]
fn the_hands_are_drawn_in_each_pose() {
    let pack = baked_base_pack().expect("the base pack bakes");
    assert!(pack.get(AssetKind::Scene, vellmark_content::AssetId::from_key("scene.yard")).is_some());
    let mut offscreen = match Offscreen::with_pack(&Default::default(), &ShaderLibrary::embedded(), 480, 270, pack, Some("scene.yard")) {
        Ok(o) => o,
        Err(e) if e.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some() => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    let eye = Vec3::new(0.0, 1.68, 2.3);
    let camera = Camera::looking(eye, 0.0, -0.05);
    let poses: Vec<(&str, PlayerSnap)> = vec![
        ("rest", player()),
        ("swing", PlayerSnap { swing: Some((SwingKind::Light { from_left: false }, 0.45)), ..player() }),
        ("heavy", PlayerSnap { drawing: Some(0.6), ..player() }),
        ("shield", PlayerSnap { holding: [Holding::Weapon, Holding::Shield], blocking: true, ..player() }),
        ("writing", PlayerSnap { charge: [0.0, 0.8, 0.0], ..player() }),
        ("bow", PlayerSnap { holding: [Holding::Weapon, Holding::Empty], weapon: WeaponKind::Bow, drawing: Some(0.9), ..player() }),
    ];
    let out = vellmark_harness::repo_root().join("build/captures/hands");
    std::fs::create_dir_all(&out).unwrap();
    // The same view with no hands, to see what each pose adds.
    let mut bare = None;
    for (name, p) in poses.iter().map(|(n, p)| (*n, p.clone())).chain(std::iter::once(("bare", PlayerSnap { reading: ReadingSnap::Close, ..player() }))) {
        let mut vm = ViewModel::default();
        for (key, place) in vm.place(&p, camera.eye, camera.rotation, &[], 3.0, 1.0 / 60.0) {
            offscreen.renderer.scene_mut().place_kit(key, place);
        }
        let views = [View::full(camera)];
        let input = FrameInput {
            views: &views,
            time_of_day: 10.5,
            seconds: 3.0,
            glyph_layer: true,
            close_reading: 0.0,
            dive: 0.0,
            lamps: 0.0,
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
        let img = offscreen.frame(&input);
        img.save(&out.join(format!("{name}.png"))).unwrap();
        if name == "bare" {
            bare = Some(img);
        } else {
            let _ = Quat::IDENTITY;
            shots_push(&out, name, img);
        }
    }
    let bare = bare.unwrap();
    for (name, _) in &poses {
        let img = vellmark_harness::image::Image::load(&out.join(format!("{name}.png"))).unwrap();
        let changed = img.rgba.chunks(4).zip(bare.rgba.chunks(4)).filter(|(a, b)| a.iter().zip(b.iter()).any(|(x, y)| x.abs_diff(*y) > 30)).count();
        let share = changed as f32 / (img.rgba.len() / 4) as f32;
        assert!(share > 0.01 && share < 0.45, "{name}: the hands cover {:.1}% of the frame", share * 100.0);
    }
}

fn shots_push(_out: &std::path::Path, _name: &str, _img: vellmark_harness::image::Image) {}
