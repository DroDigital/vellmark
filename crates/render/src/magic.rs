//! Emendation as it is seen: its five parts, each from the same few parameters per word.
//!
//! 1. The charge: gold characters gathering at the casting hand, more and larger as the
//!    charge builds, and a light there.
//! 2. The flight: the word as a stream of glyph particles with a bright head, a moving
//!    light on the world it passes.
//! 3. The impact: a burst shaped by the word's class.
//! 4. The rewrite wave: from the impact, the target's own fine print rewrites itself
//!    outward in the word's colour, legible from afar while it passes, then sinks back.
//! 5. The lasting state: while the word holds, its thing shows it.
//!
//! The particles are worked out on the GPU from a few emitters a frame
//! (`vs_spark` in `shaders/opaque.wgsl`); the wave and the lasting states are drawn in
//! the surface shader.

use glam::{Quat, Vec3};
use vellmark_content::{AssetId, Light, WordTable};
use vellmark_sim::casting::Holding;
use vellmark_sim::emendation::Landing;
use vellmark_sim::Snapshot;

use crate::frame::{ViewUniform, MAX_EMITTERS, MAX_WAVES};

/// How long an impact's burst lasts, and a wave after it has run its reach.
const BURST_SECONDS: f32 = 0.9;
const WAVE_TAIL: f32 = 1.4;

#[derive(Clone, Debug)]
struct Look {
    word: String,
    colour: [f32; 3],
    charge_glyphs: u32,
    impact: f32,
    wave_speed: f32,
}

#[derive(Clone, Debug)]
struct Wave {
    centre: Vec3,
    began: f32,
    colour: [f32; 3],
    target: u64,
    speed: f32,
    reach: f32,
    strength: f32,
}

#[derive(Clone, Debug)]
struct Burst {
    centre: Vec3,
    began: f32,
    colour: [f32; 3],
    shape: f32,
    seed: f32,
    text: [u32; 4],
    strength: f32,
}

/// The impact's shapes, by the name a sheet gives them, as the shader numbers them.
fn shape(name: &str) -> f32 {
    match name {
        "burst" => 0.0,
        "creep" => 1.0,
        "knit" => 2.0,
        "crack" => 3.0,
        "drift" => 4.0,
        "drop" => 5.0,
        _ => 6.0,
    }
}

/// Up to sixteen characters, four to a word.
fn pack(text: &str) -> [u32; 4] {
    let mut out = [0u32; 4];
    for (i, b) in text.bytes().filter(|b| (33..127).contains(b)).take(16).enumerate() {
        out[i / 4] |= (b as u32) << ((i % 4) * 8);
    }
    out
}

/// What a lasting effect looks like on its thing, as the shader numbers it: 1 shining,
/// 2 opened, 3 knitting, 4 quieted, 5 edged. Heat and cold are drawn by heat.
fn lasting(effect: &str) -> f32 {
    match effect {
        "light" | "glow" | "repel" | "flare" | "blaze" | "glint" => 1.0,
        "open" | "unfold" | "gape" => 2.0,
        "mend" | "heal" => 3.0,
        "pacify" | "still" | "subdue" | "rapt" | "grateful" | "shut" => 4.0,
        "sharpen" | "edge" | "thorns" | "cutting" | "rime" => 5.0,
        _ => 0.0,
    }
}

#[derive(Default)]
pub struct Magic {
    looks: Vec<Look>,
    waves: Vec<Wave>,
    bursts: Vec<Burst>,
    last_step: u64,
    /// This frame's.
    emitters: Vec<([[f32; 4]; 4], [u32; 4])>,
    lights: Vec<Light>,
    lasting: Vec<(u64, f32)>,
    /// Lamps put out by a word.
    dark: Vec<u64>,
}

impl Magic {
    pub fn new(words: &WordTable) -> Self {
        let mut looks: Vec<Look> = words
            .words
            .iter()
            .map(|w| Look { word: w.word.clone(), colour: w.look.colour, charge_glyphs: w.look.charge_glyphs, impact: shape(&w.look.impact), wave_speed: w.look.wave_speed })
            .collect();
        // A typo lands as its word would, in a soured light.
        looks.extend(words.words.iter().map(|w| Look {
            word: w.typo.word.clone(),
            colour: [0.75, 0.5, 0.45],
            charge_glyphs: w.look.charge_glyphs,
            impact: shape(&w.look.impact),
            wave_speed: w.look.wave_speed,
        }));
        Self { looks, ..Default::default() }
    }

    fn look(&self, word: &str) -> Option<&Look> {
        self.looks.iter().find(|l| l.word == word)
    }

    /// Takes in the world as it is now. `seconds` is the clock the shaders run on.
    pub fn observe(&mut self, snap: &Snapshot, seconds: f32) {
        // Words that landed since the last step seen.
        if snap.step != self.last_step {
            self.last_step = snap.step;
            for l in &snap.landings {
                let Some(look) = self.look(&l.word).cloned() else { continue };
                let target = snap.targets.iter().find(|t| t.id == l.target);
                let reach = target.map_or(2.0, |t| t.radius * 1.6 + 0.6);
                let took = matches!(l.landing, Landing::Took { .. });
                let strength = if took { 1.0 + 0.6 * l.charge + if l.both { 0.5 } else { 0.0 } } else { 0.35 };
                let colour = if took { look.colour } else { [0.62, 0.6, 0.56] };
                self.bursts.push(Burst { centre: l.at, began: seconds, colour, shape: if took { look.impact } else { 6.0 }, seed: (l.target % 997) as f32, text: pack(&l.word), strength });
                // A compound: a second burst, larger, in gold, written with its name.
                if let Some((name, Landing::Took { .. })) = &l.compound {
                    self.bursts.push(Burst { centre: l.at, began: seconds, colour: [1.0, 0.82, 0.35], shape: 0.0, seed: (l.target % 991) as f32 + 0.5, text: pack(&name.to_ascii_uppercase()), strength: 2.2 });
                }
                if took {
                    self.waves.push(Wave { centre: l.at, began: seconds, colour: look.colour, target: l.target, speed: look.wave_speed, reach, strength });
                }
            }
        }
        self.bursts.retain(|b| seconds - b.began < BURST_SECONDS);
        self.waves.retain(|w| seconds - w.began < w.reach / w.speed.max(0.1) + WAVE_TAIL);
        while self.waves.len() > MAX_WAVES {
            self.waves.remove(0);
        }

        self.emitters.clear();
        self.lights.clear();
        // The casting hands of the first player, from where their eye is.
        if let (Some(p), Some(v)) = (snap.players.first(), snap.views.first()) {
            let rot = Quat::from_rotation_y(-v.yaw) * Quat::from_rotation_x(v.pitch);
            for (hand, side) in [(0usize, 0.22f32), (1, -0.22)] {
                let Holding::Word(w) = p.holding[hand] else { continue };
                let charge = p.charge[hand].max(p.charge[2]);
                if charge <= 0.0 {
                    continue;
                }
                let Some(look) = self.looks.get(w).cloned() else { continue };
                let at = v.eye + rot * Vec3::new(side, -0.24, -0.5);
                let count = (look.charge_glyphs as f32 * (0.3 + 0.7 * charge)).max(4.0);
                self.emitters.push((
                    [[at.x, at.y, at.z, 0.0], [0.0, 1.0, 0.0, 0.0], [look.colour[0], look.colour[1], look.colour[2], 1.0 + 2.0 * charge], [count, 0.012 + 0.012 * charge, hand as f32, charge]],
                    pack(&look.word),
                ));
                self.lights.push(Light { key: AssetId(0), position: at.into(), colour: look.colour.map(|c| c * (0.6 + 2.4 * charge)), radius: 2.5 + 2.0 * charge, casts_shadow: false });
            }
        }
        // Words in flight: a head and a trail, and a light that goes with them.
        for (i, f) in snap.flights.iter().enumerate() {
            let Some(look) = self.look(&f.word).cloned() else { continue };
            let to = snap.targets.iter().find(|t| t.id == f.target).map_or(f.at, |t| t.centre);
            let dir = (to - f.from).normalize_or_zero();
            let gone = f.at.distance(f.from);
            // A typo goes visibly wrong halfway: its letters turn into the typo's, and its
            // light sours.
            let wrong = !f.typo.is_empty() && gone > to.distance(f.from) * 0.5;
            let colour = if wrong { [0.75, 0.5, 0.45] } else { look.colour };
            let text = if wrong { pack(&f.typo) } else { pack(&look.word) };
            self.emitters.push((
                [[f.at.x, f.at.y, f.at.z, 1.0], [dir.x, dir.y, dir.z, gone], [colour[0], colour[1], colour[2], 3.0 + 2.0 * f.charge], [90.0, 0.03 + 0.015 * f.charge, i as f32 * 7.31, f.charge]],
                text,
            ));
            self.lights.push(Light { key: AssetId(0), position: f.at.into(), colour: look.colour.map(|c| c * (3.5 + 3.0 * f.charge)), radius: 7.0, casts_shadow: false });
        }
        // Impacts: a burst, and a flash of light that dies quickly.
        for b in &self.bursts {
            let age = seconds - b.began;
            self.emitters.push((
                [[b.centre.x, b.centre.y, b.centre.z, 2.0], [0.0, 1.0, 0.0, b.began], [b.colour[0], b.colour[1], b.colour[2], 3.0 * b.strength], [120.0 * b.strength.min(1.5), 0.02, b.seed, b.shape]],
                b.text,
            ));
            let flash = (1.0 - age / 0.45).max(0.0);
            if flash > 0.0 {
                self.lights.push(Light { key: AssetId(0), position: b.centre.into(), colour: b.colour.map(|c| c * 10.0 * flash * b.strength), radius: 9.0, casts_shadow: false });
            }
        }
        // Burning things send up embers and give off flickering light; frozen ones
        // glitter. The nearest first, as many as there is room for.
        let eye = snap.views.first().map_or(Vec3::ZERO, |v| v.eye);
        let mut alight: Vec<(f32, Vec3, f32, bool)> = snap
            .targets
            .iter()
            .filter_map(|t| {
                let burning = t.effects.iter().any(|e| matches!(e.as_str(), "ignite" | "flare" | "blaze"));
                let frozen = t.effects.iter().any(|e| matches!(e.as_str(), "freeze" | "frost" | "rime"));
                (burning || frozen).then(|| (t.centre.distance(eye), t.centre, t.radius, burning))
            })
            .collect();
        alight.sort_by(|a, b| a.0.total_cmp(&b.0));
        for (i, (_, centre, radius, burning)) in alight.into_iter().take(10).enumerate() {
            if burning {
                self.emitters.push(([[centre.x, centre.y, centre.z, 3.0], [0.0, 1.0, 0.0, radius], [1.0, 0.45, 0.12, 7.0], [110.0, 0.065, i as f32 * 3.7, 0.0]], pack("^*',`^")));
                let flicker = 0.75 + 0.25 * (seconds * 11.0 + i as f32).sin() * (seconds * 7.3).cos();
                self.lights.push(Light { key: AssetId(0), position: (centre + Vec3::Y * radius * 0.5).into(), colour: [3.2 * flicker, 1.3 * flicker, 0.35 * flicker], radius: 3.0 + radius * 2.5, casts_shadow: false });
            } else {
                self.emitters.push(([[centre.x, centre.y, centre.z, 4.0], [0.0, 1.0, 0.0, radius], [0.75, 0.9, 1.0, 2.5], [60.0, 0.022, i as f32 * 5.1, 0.0]], pack("*.+")));
            }
        }
        self.emitters.truncate(MAX_EMITTERS);

        // What stays while words hold.
        self.lasting.clear();
        self.dark.clear();
        for t in snap.targets.iter() {
            if let Some(code) = t.effects.iter().map(|e| lasting(e)).filter(|&c| c > 0.0).reduce(f32::max) {
                self.lasting.push((t.id, code + 0.5));
            }
            if t.effects.iter().any(|e| e == "douse") {
                self.dark.push(t.id);
            }
            // A thing made to shine gives off light.
            if t.effects.iter().any(|e| e == "glow" || e == "light" || e == "blaze") && self.lights.len() < 24 {
                let colour = t.words.iter().rev().find_map(|w| self.look(w)).map_or([1.0, 0.9, 0.7], |l| l.colour);
                self.lights.push(Light { key: AssetId(0), position: t.centre.into(), colour: colour.map(|c| c * 2.2), radius: 3.0 + t.radius * 2.0, casts_shadow: false });
            }
        }
    }

    /// Writes this frame's waves and emitters into a view's uniform. `instance_of` gives
    /// a target's place in the instance buffer, counted from 1, if it has one.
    pub fn fill(&self, uniform: &mut ViewUniform, instance_of: &dyn Fn(u64) -> Option<u32>) {
        uniform.magic_counts = [self.waves.len() as u32, self.emitters.len() as u32, 0, 0];
        for (slot, w) in uniform.waves.iter_mut().zip(&self.waves) {
            let instance = instance_of(w.target).unwrap_or(0) as f32;
            *slot = [[w.centre.x, w.centre.y, w.centre.z, w.began], [w.colour[0], w.colour[1], w.colour[2], instance], [w.speed, w.reach, w.strength, 0.0]];
        }
        for (i, (e, text)) in self.emitters.iter().enumerate() {
            uniform.emitters[i] = *e;
            uniform.emitter_text[i] = *text;
        }
    }

    /// The scene's lights, less those words have put out, and the lights of magic.
    pub fn lights(&self, scene: &[Light]) -> Vec<Light> {
        let mut all: Vec<Light> = scene.iter().filter(|l| !self.dark.contains(&l.key.0)).cloned().collect();
        all.extend(self.lights.iter().cloned());
        all
    }

    pub fn emitter_count(&self) -> usize {
        self.emitters.len()
    }

    pub fn lasting(&self) -> &[(u64, f32)] {
        &self.lasting
    }

    /// Whether anything magical is under way, for tests.
    pub fn active(&self) -> (usize, usize) {
        (self.waves.len(), self.emitters.len())
    }
}
