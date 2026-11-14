//! The scripted run of Phase S03, in the real valley, driven as a bot drives it, through
//! `World::step` alone: read the Tithe Hill Stone and learn `OPEN`; open the locked gate
//! of the Inkling's yard and walk through it; light the lantern by it; fight the Inkling
//! and win with blade and word, with words alone, and with steel alone; read the Page at
//! the head of the Mended Bridge, learn `WHOLE`, mend the bridge, and walk across it.
//!
//! The bot is placed at each station; the walk between them is the Showcase's tour bot's
//! (S07). Words whose Standing Pages are built in S06 (`LIT`, `COLD`, `BURNING`) are given
//! where the run needs them, and say so.

use glam::Vec3;
use vellmark_content::{AssetId, AssetKind, Scene};
use vellmark_harness::capture::baked_base_pack;
use vellmark_sim::melee::WeaponKind;
use vellmark_sim::reading::ReadingSnap;
use vellmark_sim::{buttons, PlayerInput, World, WorldConfig, STEP_HZ};

struct Valley {
    world: World,
    terrain: std::sync::Arc<vellmark_world::Terrain>,
}

fn valley() -> Valley {
    let pack = baked_base_pack().expect("the base pack bakes");
    let scene = Scene::from_bytes(pack.get(AssetKind::Scene, AssetId::from_key("scene.valley")).expect("the valley")).unwrap();
    let terrain = std::sync::Arc::new(vellmark_world::Terrain::from_pack(&pack).unwrap());
    let rules = vellmark_render::word_rules(&pack);
    let mut world = World::new(WorldConfig { hours_per_second: 0.0, start_time_of_day: 11.0, ..Default::default() });
    let (solids, breakables) = vellmark_render::scene_rules(&scene);
    world.set_scene(solids, breakables);
    world.set_trunks(vellmark_render::tree_trunks(&pack, &scene));
    world.set_readables(vellmark_render::scene_readables(&pack, "scene.valley"));
    world.set_targets(vellmark_render::scene_targets(&pack, "scene.valley", &rules));
    world.set_word_rules(rules);
    world.add_player(Vec3::new(scene.spawn[0], scene.spawn[1], scene.spawn[2]), scene.spawn[3]);
    world.set_ground(terrain.clone());
    for c in vellmark_render::scene_creatures(&scene) {
        world.add_creature(c);
    }
    world.step(&[PlayerInput::default()]);
    Valley { world, terrain }
}

fn id(w: &World) -> vellmark_sim::PlayerId {
    w.snapshot().players[0].id
}

fn go(w: &mut World, steps: u32, input: PlayerInput) {
    for _ in 0..steps {
        w.step(&[input]);
    }
}

/// Turns the eye to look at `at`, in one step.
fn look_at(w: &mut World, at: Vec3) {
    let s = w.snapshot();
    let (p, v) = (&s.players[0], &s.views[0]);
    let to = at - v.eye;
    let yaw = to.x.atan2(-to.z);
    let pitch = (to.y / Vec3::new(to.x, 0.0, to.z).length().max(1e-3)).atan();
    w.step(&[PlayerInput { look_yaw: wrap(yaw - p.yaw), look_pitch: pitch - p.pitch, ..Default::default() }]);
}

fn wrap(mut a: f32) -> f32 {
    while a > std::f32::consts::PI {
        a -= std::f32::consts::TAU;
    }
    while a < -std::f32::consts::PI {
        a += std::f32::consts::TAU;
    }
    a
}

fn target(w: &World, key: &str) -> vellmark_sim::TargetSnap {
    let id = AssetId::from_key(&format!("scene.valley.{key}")).0;
    w.snapshot().targets.iter().find(|t| t.id == id).cloned().unwrap_or_else(|| panic!("{key} carries an Inscription"))
}

/// Writes the off hand's word on what is at `at`: a tap, or a charge of `steps`.
fn write_at(w: &mut World, at: Vec3, steps: u32) {
    look_at(w, at);
    go(w, steps.max(3), PlayerInput { buttons: buttons::ATTACK_OFF, ..Default::default() });
    for _ in 0..STEP_HZ / 2 {
        go(w, 1, PlayerInput::default());
    }
}

/// Puts `word` in the off hand, learning it if it must.
fn hold(w: &mut World, word: &str) {
    let me = id(w);
    w.hold_word(me, 1, word);
}

/// Reads the Standing Page at `stone` from `stand`, and returns the word it gave.
fn read_page(v: &mut Valley, stand: Vec3, stone: Vec3) -> Option<String> {
    let me = id(&v.world);
    v.world.place_player(me, stand, 0.0);
    look_at(&mut v.world, stone);
    let mut learned = None;
    let mut read = false;
    for _ in 0..STEP_HZ * 3 {
        v.world.step(&[PlayerInput { buttons: buttons::READ, ..Default::default() }]);
        let s = v.world.snapshot();
        read |= matches!(s.players[0].reading, ReadingSnap::Text { .. });
        if let Some(w) = &s.players[0].learned {
            learned = Some(w.clone());
        }
    }
    go(&mut v.world, 2, PlayerInput::default());
    assert!(read, "the Page is read");
    learned
}

#[test]
fn read_learn_open_light_and_mend() {
    let mut v = valley();
    // Read the Tithe Hill Stone, and learn OPEN.
    let stone = Vec3::new(-102.45, 48.1, 105.5);
    let stand = Vec3::new(-102.0, 0.0, 107.4);
    assert_eq!(read_page(&mut v, stand, stone).as_deref(), Some("OPEN"), "the Tithe Hill Stone gives OPEN");
    assert!(v.world.snapshot().players[0].holding[1] == vellmark_sim::casting::Holding::Word(v.world.word_rules().word("OPEN").unwrap()), "OPEN settles into the free hand");

    // The yard's gate is locked: it stops a body.
    let me = id(&v.world);
    let before_gate = Vec3::new(58.4, 0.0, -63.75);
    v.world.place_player(me, before_gate, -std::f32::consts::FRAC_PI_2);
    go(&mut v.world, STEP_HZ * 2, PlayerInput { move_y: 1.0, ..Default::default() });
    assert!(v.world.snapshot().players[0].feet.x > 56.0, "the locked gate stops the walker: {}", v.world.snapshot().players[0].feet);
    // Open it with OPEN, and walk through.
    v.world.place_player(me, before_gate, -std::f32::consts::FRAC_PI_2);
    let gate = target(&v.world, "yard_gate");
    write_at(&mut v.world, gate.centre, 3);
    assert!(target(&v.world, "yard_gate").words.contains(&"OPEN".to_string()), "the gate's Inscription says OPEN");
    go(&mut v.world, STEP_HZ, PlayerInput { move_y: 1.0, ..Default::default() });
    assert!(v.world.snapshot().players[0].feet.x < 55.5, "through the open gate: {}", v.world.snapshot().players[0].feet);

    // Light the lantern by the gate. (LIT's Page is the Lamp That Was Promised, in S06.)
    v.world.learn(me, "LIT");
    hold(&mut v.world, "LIT");
    v.world.place_player(me, before_gate + Vec3::new(1.0, 0.0, 0.0), 0.0);
    let lantern = target(&v.world, "yard_lantern");
    write_at(&mut v.world, lantern.centre, 3);
    let lantern = target(&v.world, "yard_lantern");
    assert!(lantern.effects.contains(&"light".to_string()), "the lantern is lit: {:?}", lantern.effects);

    // The Page at the head of the Mended Bridge gives WHOLE.
    let page = Vec3::new(128.05, 7.6, -98.3);
    assert_eq!(read_page(&mut v, Vec3::new(128.05, 0.0, -96.6), page).as_deref(), Some("WHOLE"), "the bridge's Page gives WHOLE");
    // Mend the bridge, charged, from its head; then walk across it.
    let bridge = target(&v.world, "mended_bridge");
    let head = Vec3::new(130.6, 0.0, -100.3);
    v.world.place_player(me, head, 0.0);
    write_at(&mut v.world, bridge.centre + Vec3::Y * -0.3, STEP_HZ + STEP_HZ / 2);
    let mut whole = false;
    for _ in 0..STEP_HZ * 10 {
        go(&mut v.world, 1, PlayerInput::default());
        if !v.world.snapshot().broken.contains(&bridge.id) {
            whole = true;
            break;
        }
    }
    assert!(whole, "the bridge stands whole");
    // Walk the length of it, toward the far bank.
    let along = Vec3::new(173.19f32.to_radians().sin(), 0.0, 173.19f32.to_radians().cos());
    v.world.place_player(me, head - along * 0.6, along.x.atan2(-along.z));
    let mut lowest = f32::MAX;
    for _ in 0..STEP_HZ * 9 {
        go(&mut v.world, 1, PlayerInput { move_y: 1.0, ..Default::default() });
        let f = v.world.snapshot().players[0].feet;
        lowest = lowest.min(f.y);
    }
    let f = v.world.snapshot().players[0].feet;
    let far = (f - Vec3::new(132.25, f.y, -113.25)).dot(along);
    assert!(far > 11.0, "across the bridge to the far bank: {f}");
    assert!(lowest > 3.5, "never in the river: lowest {lowest}");
    let _ = &v.terrain;
}

/// How the fight is fought.
#[derive(Clone, Copy, Debug, PartialEq)]
enum Build {
    BladeAndWord,
    WordsAlone,
    SteelAlone,
}

/// Fights every Inkling in the south yard, and returns whether all fell and the player
/// stood, and how long it took.
fn fight(build: Build) -> (bool, f32, f32) {
    let mut v = valley();
    let me = id(&v.world);
    let w = &mut v.world;
    // Inside the yard, by its gate.
    w.place_player(me, Vec3::new(54.5, 0.0, -63.75), -std::f32::consts::FRAC_PI_2);
    match build {
        Build::BladeAndWord => {
            w.arm(me, WeaponKind::Sword, false);
            w.learn(me, "COLD");
            hold(w, "COLD");
        }
        Build::WordsAlone => {
            // No weapon: a word in each hand, fire and light. (Their Pages are in S06.)
            w.learn(me, "BURNING");
            w.learn(me, "LIT");
            w.hold_word(me, 0, "BURNING");
            w.hold_word(me, 1, "LIT");
        }
        Build::SteelAlone => w.arm(me, WeaponKind::Sword, false),
    }
    let yard = |c: &vellmark_sim::melee::Creature| c.feet.x > 43.5 && c.feet.x < 56.5 && c.feet.z > -68.0 && c.feet.z < -57.0;
    let foes: Vec<u64> = w.creatures().iter().filter(|c| yard(c)).map(|c| c.id).collect();
    assert!(!foes.is_empty());
    let mut t = 0u32;
    while t < STEP_HZ * 120 {
        t += 1;
        let s = w.snapshot();
        let p = s.players[0].clone();
        if p.health <= 0.0 {
            break;
        }
        let alive: Vec<_> = w.creatures().iter().filter(|c| c.alive() && (foes.contains(&c.id) || yard(c))).cloned().collect();
        let Some(c) = alive.iter().min_by(|a, b| a.feet.distance(p.feet).total_cmp(&b.feet.distance(p.feet))).cloned() else { break };
        let to = c.centre() - s.views[0].eye;
        let flat = Vec3::new(c.feet.x - p.feet.x, 0.0, c.feet.z - p.feet.z);
        let d = flat.length();
        let mut input = PlayerInput {
            look_yaw: wrap(to.x.atan2(-to.z) - p.yaw),
            look_pitch: (to.y / Vec3::new(to.x, 0.0, to.z).length().max(0.1)).atan() - p.pitch,
            ..Default::default()
        };
        let pulse = (t / 3) % 2 == 0;
        // Its gathering is seen: step aside before the lunge.
        if c.telegraph > 0.7 && d < 2.8 && p.stamina > 20.0 {
            input.buttons |= buttons::SIDESTEP;
            input.move_x = if (t / 90) % 2 == 0 { 1.0 } else { -1.0 };
        } else {
            let marks = c.marks;
            match build {
                Build::SteelAlone => {
                    if d > 1.6 {
                        input.move_y = 1.0;
                    } else if pulse {
                        input.buttons |= buttons::ATTACK_MAIN;
                    }
                }
                Build::BladeAndWord => {
                    // Cold first, so that it does not split; then steel.
                    if marks & vellmark_sim::melee::marks::SLOWED == 0 && p.ink > 12.0 && d < 9.0 {
                        if pulse {
                            input.buttons |= buttons::ATTACK_OFF;
                        }
                    } else if d > 1.6 {
                        input.move_y = 1.0;
                    } else if pulse {
                        input.buttons |= buttons::ATTACK_MAIN;
                    }
                }
                Build::WordsAlone => {
                    // Set it alight, then drive it back with light while it burns.
                    if marks & vellmark_sim::melee::marks::BURNING == 0 && p.ink > 12.0 && d < 12.0 {
                        if pulse {
                            input.buttons |= buttons::ATTACK_MAIN;
                        }
                    } else if marks & vellmark_sim::melee::marks::DAZZLED == 0 && p.ink > 10.0 && d < 8.0 {
                        if pulse {
                            input.buttons |= buttons::ATTACK_OFF;
                        }
                    } else if d < 3.0 {
                        input.move_y = -1.0;
                    }
                }
            }
        }
        w.step(&[input]);
    }
    let s = w.snapshot();
    let all_fell = w.creatures().iter().filter(|c| foes.contains(&c.id) || yard(c)).all(|c| !c.alive());
    (all_fell && s.players[0].health > 0.0, t as f32 / STEP_HZ as f32, s.players[0].health)
}

/// Records a fight's result for the evidence.
fn record(build: Build, seconds: f32, health: f32) {
    let dir = vellmark_harness::repo_root().join("build/captures/fights");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join(format!("{build:?}.txt")), format!("{build:?}: won in {seconds:.1} s with {health:.0} health left\n")).unwrap();
}

#[test]
fn the_fight_is_won_with_blade_and_word() {
    let (won, seconds, health) = fight(Build::BladeAndWord);
    record(Build::BladeAndWord, seconds, health);
    assert!(won, "blade and word: lost, after {seconds} s with {health} health");
}

#[test]
fn the_fight_is_won_with_words_alone() {
    let (won, seconds, health) = fight(Build::WordsAlone);
    record(Build::WordsAlone, seconds, health);
    assert!(won, "words alone: lost, after {seconds} s with {health} health");
}

#[test]
fn the_fight_is_won_with_steel_alone() {
    let (won, seconds, health) = fight(Build::SteelAlone);
    record(Build::SteelAlone, seconds, health);
    assert!(won, "steel alone: lost, after {seconds} s with {health} health");
}
