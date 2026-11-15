//! The soak: a bot plays the real valley through `World::step` for a long while, faster
//! than real time: it wanders, sprints and jumps, fights the creatures with steel and
//! with every word, reads the Pages, takes up what falls, and saves and loads as it goes;
//! the creatures come back when they are all down. Every second it checks that nothing
//! has broken: no number gone bad, no one fallen out of the world, nothing growing without
//! end, and the player never stuck. Three minutes run with the tests;
//! `VELLMARK_SOAK_MINUTES=240 cargo test -p vellmark-harness --release --test soak -- --ignored`
//! runs four hours and writes `docs/evidence/S07/soak.txt`.

use std::sync::Arc;

use glam::Vec3;
use vellmark_content::{AssetId, AssetKind, LootTable, Scene, LOOT_KEY};
use vellmark_harness::capture::baked_base_pack;
use vellmark_sim::{buttons, PlayerInput, World, WorldConfig, STEP_HZ};

struct Report {
    seconds: u64,
    saves: u32,
    deaths: u32,
    kills: usize,
    words: usize,
    most_letters: usize,
    most_dropped: usize,
    most_arrows: usize,
    longest_stuck: f32,
    wall_seconds: f32,
}

fn soak(minutes: u64) -> Report {
    let pack = baked_base_pack().expect("the base pack bakes");
    let scene = Scene::from_bytes(pack.get(AssetKind::Scene, AssetId::from_key("scene.valley")).unwrap()).unwrap();
    let terrain = Arc::new(vellmark_world::Terrain::from_pack(&pack).unwrap());
    let rules = vellmark_render::word_rules(&pack);
    let mut w = World::new(WorldConfig { seed: 41, ..Default::default() });
    let (solids, breakables) = vellmark_render::scene_rules(&scene);
    w.set_scene(solids, breakables);
    w.set_trunks(vellmark_render::tree_trunks(&pack, &scene));
    w.set_readables(vellmark_render::scene_readables(&pack, "scene.valley"));
    w.set_targets(vellmark_render::scene_targets(&pack, "scene.valley", &rules));
    let words: Vec<String> = rules.words.iter().map(|r| r.spelling.clone()).collect();
    w.set_word_rules(rules);
    if let Some(loot) = pack.get(AssetKind::Table, AssetId::from_key(LOOT_KEY)).and_then(|b| LootTable::from_bytes(b).ok()) {
        w.set_loot(loot);
    }
    w.set_ground(terrain.clone());
    let spawn = Vec3::new(scene.spawn[0], scene.spawn[1] - vellmark_sim::EYE_HEIGHT, scene.spawn[2]);
    let me = w.add_player(spawn, scene.spawn[3]);
    for c in vellmark_render::scene_creatures(&scene) {
        w.add_creature(c);
    }
    for word in &words {
        w.learn(me, word);
    }
    let started = std::time::Instant::now();
    let mut rng: u64 = 0x9e37_79b9_7f4a_7c15;
    let mut next = || {
        rng ^= rng << 13;
        rng ^= rng >> 7;
        rng ^= rng << 17;
        rng
    };
    let mut report = Report { seconds: 0, saves: 0, deaths: 0, kills: 0, words: 0, most_letters: 0, most_dropped: 0, most_arrows: 0, longest_stuck: 0.0, wall_seconds: 0.0 };
    let mut save = w.save().to_bytes();
    let mut heading = 0.0f32;
    let mut window_start = spawn;
    let mut stuck_for = 0.0f32;
    let total = minutes * 60 * STEP_HZ as u64;
    for step in 0..total {
        let s = w.snapshot();
        let p = &s.players[0];
        let t = step as f32 / STEP_HZ as f32;
        // Every four seconds a new way, turned home when it strays far.
        if step % (STEP_HZ as u64 * 4) == 0 {
            heading = (next() % 6283) as f32 / 1000.0;
            let home = spawn - p.feet;
            if Vec3::new(home.x, 0.0, home.z).length() > 160.0 {
                heading = home.x.atan2(-home.z);
            }
        }
        // A creature near: turn on it, and fight.
        let foe = s.creatures.iter().filter(|c| c.alive()).map(|c| (c.feet.distance(p.feet), c.feet)).filter(|(d, _)| *d < 25.0).min_by(|a, b| a.0.total_cmp(&b.0));
        if let Some((_, at)) = foe {
            let to = at - p.feet;
            heading = to.x.atan2(-to.z);
        }
        let turn = ((heading - p.yaw + std::f32::consts::PI).rem_euclid(std::f32::consts::TAU) - std::f32::consts::PI).clamp(-0.08, 0.08);
        let r = next();
        let mut held = 0;
        if r % 7 == 0 {
            held |= buttons::SPRINT;
        }
        if step % 300 == 17 {
            held |= buttons::JUMP;
        }
        if step % 40 < 3 {
            held |= buttons::ATTACK_MAIN;
        }
        if step % 150 < 50 {
            held |= buttons::ATTACK_OFF;
        }
        if step % 600 == 5 {
            held |= buttons::NEXT_WORD;
        }
        if step % 90 == 7 {
            held |= buttons::INTERACT;
        }
        if step % 1800 < 120 {
            held |= buttons::READ;
        }
        w.step(&[PlayerInput { move_y: 1.0, look_yaw: turn, look_pitch: -p.pitch * 0.1, buttons: held, ..Default::default() }]);
        let s = w.snapshot();
        report.kills += s.hits.iter().filter(|h| h.killed).count();
        report.words += s.landings.len();
        assert!(s.heard.len() < 400, "{} sounds in one step", s.heard.len());
        if step % STEP_HZ as u64 != 0 {
            continue;
        }
        // Each second: nothing broken.
        report.seconds += 1;
        let p = &s.players[0];
        for v in [p.feet.x, p.feet.y, p.feet.z, p.yaw, p.pitch, p.health, p.stamina, p.ink] {
            assert!(v.is_finite(), "a number went bad at {t} s: {p:?}");
        }
        let ground = terrain.height_at(p.feet.x, p.feet.z).unwrap_or(p.feet.y);
        assert!(p.feet.y > ground - 3.0, "fell out of the world at {t} s: {} under {ground}", p.feet);
        assert!(p.feet.y < ground + 60.0, "flew off at {t} s: {}", p.feet);
        report.most_letters = report.most_letters.max(s.letters.len());
        report.most_dropped = report.most_dropped.max(s.dropped.len());
        report.most_arrows = report.most_arrows.max(s.arrows.len());
        assert!(s.letters.len() < 70_000, "letters grow without end: {}", s.letters.len());
        assert!(s.dropped.len() < 400, "dropped items grow without end: {}", s.dropped.len());
        assert!(s.arrows.len() < 400, "arrows grow without end: {}", s.arrows.len());
        assert!(s.flights.len() < 200, "flights grow without end: {}", s.flights.len());
        // Stuck: walking every second of a window and gone nowhere.
        if report.seconds % 20 == 0 {
            let moved = Vec3::new(p.feet.x - window_start.x, 0.0, p.feet.z - window_start.z).length();
            let held = p.talking.is_some() || !s.learning.is_empty();
            if moved < 1.0 && !held && p.health > 0.0 {
                stuck_for += 20.0;
                report.longest_stuck = report.longest_stuck.max(stuck_for);
                // A real player would turn round: so does the bot, and so it must be free.
                heading += std::f32::consts::PI;
            } else {
                stuck_for = 0.0;
            }
            assert!(stuck_for < 120.0, "stuck at {} for {stuck_for} s", p.feet);
            window_start = p.feet;
        }
        // Death: back to the last save, as a player would.
        if p.health <= 0.0 {
            report.deaths += 1;
            w.load(&vellmark_sim::save::SaveGame::from_bytes(&save).unwrap());
            if w.snapshot().players[0].health <= 0.0 {
                w.place_player(me, spawn, 0.0);
            }
        }
        // Each minute: a save, read back at once as a loaded game.
        if report.seconds % 60 == 0 && w.snapshot().players[0].health > 0.0 {
            save = w.save().to_bytes();
            let back = vellmark_sim::save::SaveGame::from_bytes(&save).expect("a save reads back");
            assert_eq!(back.to_bytes(), save, "a save writes back as it was read");
            w.load(&back);
            report.saves += 1;
        }
        // Every five minutes, to the Inklings' yards off the bridge road, as the scripted
        // bot is placed at its stations.
        if report.seconds % 300 == 150 {
            let at = Vec3::new(62.0, 0.0, -62.0);
            w.place_player(me, Vec3::new(at.x, terrain.height_at(at.x, at.z).unwrap_or(8.0) + 0.2, at.z), 0.0);
            window_start = w.snapshot().players[0].feet;
        }
        // When every creature is down, they come back.
        if s.creatures.iter().all(|c| !c.alive()) {
            w.remove_creatures();
            for c in vellmark_render::scene_creatures(&scene) {
                w.add_creature(c);
            }
        }
    }
    report.wall_seconds = started.elapsed().as_secs_f32();
    report
}

#[test]
fn three_minutes_of_play_break_nothing() {
    let r = soak(3);
    assert!(r.words > 5, "the bot wrote words: {}", r.words);
    assert!(r.kills > 0, "the bot fought and killed: {}", r.kills);
    assert_eq!(r.saves, 3);
}

#[test]
#[ignore]
fn the_long_soak() {
    let minutes: u64 = std::env::var("VELLMARK_SOAK_MINUTES").ok().and_then(|m| m.parse().ok()).unwrap_or(240);
    let r = soak(minutes);
    let text = format!(
        "The sim soak (crates/harness/tests/soak.rs), in the valley, with nothing rendered.\n\
         Game time: {} min ({} s), run in {:.0} s of wall time.\n\
         Saves made and loaded back: {}. Deaths, each back to the last save: {}.\n\
         Creatures killed: {}. Words that landed: {}.\n\
         Most letters at once: {}. Most items lying about: {}. Most arrows: {}.\n\
         Longest stretch walking and going nowhere: {:.0} s (the bot turns round after 20 s).\n\
         No number went bad, no one fell out of the world, nothing grew without end.\n",
        minutes, r.seconds, r.wall_seconds, r.saves, r.deaths, r.kills, r.words, r.most_letters, r.most_dropped, r.most_arrows, r.longest_stuck
    );
    let path = vellmark_harness::repo_root().join("docs/evidence/S07/soak.txt");
    std::fs::write(path, text).unwrap();
}
