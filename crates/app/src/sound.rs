//! The ambience, fed from the world each frame: the weather, the hour, how near running
//! water is, and how many trees stand about the listener. The mixer runs on the audio
//! thread; this only sets what it reads.

use std::sync::Arc;

use glam::{Vec2, Vec3};
use vellmark_audio::cues::{Cue, CueKind, Timbre, MOTIF_NOTES};
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
    /// Each word's motif and timbre, by its spelling and its typo's.
    motifs: Vec<(String, [u8; MOTIF_NOTES], Timbre)>,
}

impl Sound {
    /// Starts the sound. With no output device the game goes on silent.
    pub fn start(platform: &Platform, scene: &vellmark_content::Scene) -> Self {
        let controls = Controls::new();
        let mut mixer = Mixer::new(controls.clone(), 0x5eed);
        let out = match platform.open_audio(SAMPLE_RATE as i32, move |buffer| mixer.render(buffer)) {
            Ok(out) => Some(out),
            Err(e) => {
                crate::log::line(&format!("no sound: {e}"));
                None
            }
        };
        // Where the trees stand, from the scene: their keys say what they are.
        let trees = scene.objects.iter().filter(|o| tree_key(o.mesh)).map(|o| Vec2::new(o.transform[3], o.transform[11])).collect();
        Self { _out: out, controls, trees, water: 0.0, frame: 0, motifs: Vec::new() }
    }

    /// Learns every word's motif from the table of words.
    pub fn set_words(&mut self, table: &vellmark_content::WordTable) {
        self.motifs = table
            .words
            .iter()
            .map(|w| {
                let mut notes = [0u8; MOTIF_NOTES];
                for (n, m) in notes.iter_mut().zip(&w.motif) {
                    *n = *m;
                }
                (w.word.clone(), notes, Timbre::of_class(&w.class))
            })
            .collect();
    }

    /// The sounds of one step of the rules, heard from the first view: words written,
    /// blows, swings, arrows, Inklings, things breaking and mending.
    pub fn heard(&mut self, snap: &vellmark_sim::Snapshot) {
        let Some(view) = snap.views.first() else { return };
        let (eye, right) = (view.eye, Vec3::new(view.yaw.cos(), 0.0, view.yaw.sin()));
        let place = |at: Vec3| {
            let to = at - eye;
            let d = to.length();
            let pan = if d < 0.3 { 0.0 } else { (to / d).dot(right) * 0.85 };
            (pan, 1.0 / (1.0 + d / 7.0))
        };
        let cues = &self.controls.cues;
        let push = |kind: CueKind, at: Vec3, loud: f32| {
            let (pan, gain) = place(at);
            cues.push(Cue { kind, pan, gain: gain * loud, ..Default::default() });
        };
        for l in &snap.landings {
            let Some((_, notes, timbre)) = self.motifs.iter().find(|(w, _, _)| *w == l.word) else { continue };
            let (pan, gain) = place(l.at);
            let base = Cue { pan, gain: gain * if l.link > 0 { 0.4 } else { 0.8 }, notes: *notes, timbre: *timbre, charge: l.charge, wrong: l.typo, ..Default::default() };
            use vellmark_sim::emendation::Landing;
            match &l.landing {
                Landing::Took { .. } => cues.push(Cue { kind: CueKind::Motif, ..base }),
                Landing::Resisted => {
                    let mut short = base.notes;
                    for n in short.iter_mut().skip(2) {
                        *n = 0;
                    }
                    cues.push(Cue { kind: CueKind::Motif, notes: short, gain: base.gain * 0.6, ..base });
                }
                Landing::Refused(_) | Landing::Sealed => cues.push(Cue { kind: CueKind::Refusal, ..base }),
            }
        }
        use vellmark_sim::melee::SwingKind;
        for h in &snap.hits {
            let kind = match h.kind {
                SwingKind::Heavy => CueKind::HitHeavy,
                SwingKind::Bash => CueKind::Bash,
                SwingKind::Light { .. } if h.sneak || h.shattered => CueKind::HitHeavy,
                SwingKind::Light { .. } => CueKind::Hit,
            };
            push(kind, h.at, 0.9);
        }
        use vellmark_sim::Heard;
        let feet_of = |id: vellmark_sim::PlayerId| snap.players.iter().find(|p| p.id == id).map_or(eye, |p| p.feet + Vec3::Y * 1.2);
        for e in &snap.heard {
            match *e {
                Heard::Swing { player, kind } => push(if kind == SwingKind::Heavy { CueKind::SwingHeavy } else { CueKind::Swing }, feet_of(player), 0.7),
                Heard::Struck { player, blocked, .. } => push(if blocked { CueKind::Block } else { CueKind::Hurt }, feet_of(player), 0.9),
                Heard::Loose { at, .. } => push(CueKind::Loose, at, 0.8),
                Heard::Thunk { at } => push(CueKind::Thunk, at, 0.9),
                Heard::Gather { at } => push(CueKind::Gather, at, 0.7),
                Heard::Split { at } => push(CueKind::Split, at, 0.9),
                Heard::Spill { at } => push(CueKind::Spill, at, 0.9),
                Heard::Break { at } => push(CueKind::Break, at, 1.0),
                Heard::Mend { at } => push(CueKind::Mend, at, 1.0),
            }
        }
    }

    pub fn update(&mut self, eye: Vec3, weather: &vellmark_sim::Weather, hour: f32, terrain: Option<&vellmark_world::Terrain>) {
        self.frame += 1;
        // Water is looked for a few times a second, in rings about the listener.
        if self.frame % 15 == 1 {
            self.water = terrain.map_or(0.0, |t| water_near(t, eye));
        }
        let here = Vec2::new(eye.x, eye.z);
        let trees: f32 = self.trees.iter().map(|t| (1.0 - t.distance(here) / TREE_REACH).max(0.0)).sum();
        let c = &self.controls;
        c.wind.set(weather.wind_strength * 0.85 + 0.15 * weather.rain);
        c.rain.set(weather.rain);
        c.water.set(self.water);
        c.trees.set((trees / 6.0).min(1.0));
        c.daylight.set(daylight(hour));
    }
}

fn tree_key(mesh: vellmark_content::AssetId) -> bool {
    ["oak", "ash", "apple", "hawthorn"].iter().any(|k| mesh == vellmark_content::AssetId::from_key(&format!("tree.{k}.lod0")))
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
            if terrain.water_at(eye.x + r * a.cos(), eye.z + r * a.sin()).is_some() {
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
