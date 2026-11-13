//! The ambience, fed from the world each frame: the weather, the hour, how near running
//! water is, and how many trees stand about the listener. The mixer runs on the audio
//! thread; this only sets what it reads.

use std::sync::Arc;

use glam::{Vec2, Vec3};
use vellmark_audio::{Controls, Mixer, SAMPLE_RATE};
use vellmark_platform::{AudioOut, Platform};

/// How far round the listener water and trees are heard, in metres.
const WATER_REACH: f32 = 45.0;
const TREE_REACH: f32 = 28.0;

pub struct Sound {
    _out: Option<AudioOut>,
    controls: Arc<Controls>,
    trees: Vec<Vec2>,
    water: f32,
    frame: u64,
}

impl Sound {
    /// Starts the sound. With no output device the game goes on silent.
    pub fn start(platform: &Platform, scene: &vellmark_content::Scene) -> Self {
        let controls = Controls::new();
        let mut mixer = Mixer::new(controls.clone(), 0x5eed);
        let out = match platform.open_audio(SAMPLE_RATE as i32, move |buffer| mixer.render(buffer))
        {
            Ok(out) => Some(out),
            Err(e) => {
                crate::log::line(&format!("no sound: {e}"));
                None
            }
        };
        // Where the trees stand, from the scene: their keys say what they are.
        let trees = scene
            .objects
            .iter()
            .filter(|o| tree_key(o.mesh))
            .map(|o| Vec2::new(o.transform[3], o.transform[11]))
            .collect();
        Self {
            _out: out,
            controls,
            trees,
            water: 0.0,
            frame: 0,
        }
    }

    pub fn update(
        &mut self,
        eye: Vec3,
        weather: &vellmark_sim::Weather,
        hour: f32,
        terrain: Option<&vellmark_world::Terrain>,
    ) {
        self.frame += 1;
        // Water is looked for a few times a second, in rings about the listener.
        if self.frame % 15 == 1 {
            self.water = terrain.map_or(0.0, |t| water_near(t, eye));
        }
        let here = Vec2::new(eye.x, eye.z);
        let trees: f32 = self
            .trees
            .iter()
            .map(|t| (1.0 - t.distance(here) / TREE_REACH).max(0.0))
            .sum();
        let c = &self.controls;
        c.wind
            .set(weather.wind_strength * 0.85 + 0.15 * weather.rain);
        c.rain.set(weather.rain);
        c.water.set(self.water);
        c.trees.set((trees / 6.0).min(1.0));
        c.daylight.set(daylight(hour));
    }
}

fn tree_key(mesh: vellmark_content::AssetId) -> bool {
    ["oak", "ash", "apple", "hawthorn"]
        .iter()
        .any(|k| mesh == vellmark_content::AssetId::from_key(&format!("tree.{k}.lod0")))
}

/// 1 from soon after dawn to towards dusk, 0 through the night, with a dawn chorus
/// and an evening song in between.
fn daylight(hour: f32) -> f32 {
    let rise = smooth(4.8, 6.2, hour);
    let set = 1.0 - smooth(19.5, 21.0, hour);
    (rise * set).clamp(0.0, 1.0)
}

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// How loud running water is here: the nearest water in rings about the listener, louder
/// as it is nearer and falling away to nothing at the reach.
fn water_near(terrain: &vellmark_world::Terrain, eye: Vec3) -> f32 {
    let mut nearest = f32::MAX;
    let mut r = 2.0f32;
    while r <= WATER_REACH {
        let steps = ((r * std::f32::consts::TAU) / 3.0).ceil().max(8.0) as usize;
        for k in 0..steps {
            let a = k as f32 / steps as f32 * std::f32::consts::TAU;
            if terrain
                .water_at(eye.x + r * a.cos(), eye.z + r * a.sin())
                .is_some()
            {
                nearest = nearest.min(r);
            }
        }
        if nearest < f32::MAX {
            break;
        }
        r += 3.0;
    }
    if terrain.water_at(eye.x, eye.z).is_some() {
        nearest = 0.0;
    }
    (1.0 - nearest / WATER_REACH).clamp(0.0, 1.0).powf(1.6)
}
