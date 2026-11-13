use crate::clock::STEP_SECONDS;
use crate::hash::Fnv64;
use crate::input::PlayerInput;
use crate::debris::{shatter, Debris, Solid};
use crate::snapshot::{LetterSnap, PlayerSnap, Snapshot, ViewSnap};
use glam::Vec3;
use std::sync::Arc;

/// Height of the eye above the feet, in metres.
pub const EYE_HEIGHT: f32 = 1.68;

const MAX_PITCH: f32 = 1.48;
/// A breakable thing burning longer than this, in seconds, falls to its letters.
const BURN_TO_BREAK: f64 = 15.0;

/// Identifies a player for the whole session. Never reused.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PlayerId(pub u32);

#[derive(Clone, Debug)]
pub struct WorldConfig {
    pub seed: u64,
    /// Hours since midnight at step 0.
    pub start_time_of_day: f32,
    /// In-game hours that pass in one real second. 0 freezes the sun.
    pub hours_per_second: f32,
}

impl Default for WorldConfig {
    fn default() -> Self {
        Self { seed: 1, start_time_of_day: 10.0, hours_per_second: 24.0 / (48.0 * 60.0) }
    }
}

/// A thing that can be broken into its letters: its permanent ID, where it stands, and
/// the boxes it is made of in its own space, each with its material.
#[derive(Clone, Debug)]
pub struct BreakableThing {
    pub id: u64,
    pub to_world: glam::Affine3A,
    pub boxes: Vec<(Vec3, Vec3, u32)>,
    /// In kilograms if it is loose, a prop that falls and can be carried; 0 if it is
    /// fixed in place.
    pub mass: f32,
}

#[derive(Clone, Debug)]
struct Player {
    id: PlayerId,
    body: crate::character::Body,
    yaw: f32,
    pitch: f32,
    reading: crate::reading::Reading,
    /// The text in focus, for prompts.
    in_focus: Option<usize>,
    /// How sharp a reader: hidden lines up to this are seen.
    perception: u32,
    caster: crate::casting::Caster,
}

/// The ground as the body asks about it.
struct Around<'a>(Option<&'a dyn crate::Ground>);

impl crate::character::Surroundings for Around<'_> {
    fn ground(&self, x: f32, z: f32) -> Option<f32> {
        self.0.and_then(|g| g.height(x, z))
    }
    fn water(&self, x: f32, z: f32) -> Option<f32> {
        self.0.and_then(|g| g.water(x, z))
    }
}

/// The whole mutable state of the game. Stepping it is deterministic: the same
/// configuration and the same inputs give the same state, bit for bit, on every run.
#[derive(Clone, Debug)]
pub struct World {
    config: WorldConfig,
    step: u64,
    rng: u64,
    time_of_day: f32,
    players: Vec<Player>,
    next_player: u32,
    debris: Debris,
    breakables: Vec<BreakableThing>,
    broken: Arc<Vec<u64>>,
    letters: Arc<Vec<LetterSnap>>,
    letters_step: u64,
    heat: Arc<Vec<(u64, f32)>>,
    ground: Option<Arc<dyn crate::Ground>>,
    weather: crate::weather::WeatherState,
    /// The scene's solid boxes and its trees' trunks, for the still world.
    solids: Vec<Solid>,
    trunks: Vec<crate::character::Trunk>,
    /// The still world the bodies move against, rebuilt when the ground or scene changes.
    statics: Option<Arc<crate::character::Statics>>,
    /// The texts that can be read.
    readables: Arc<Vec<crate::reading::ReadableText>>,
    /// The world's own clock, in seconds: it runs slow while time is slowed.
    seconds: f64,
    /// The share of a step's time that passed in the last step: 1, or less while slowed.
    pace: f32,
    /// The words, as the rules know them.
    rules: Arc<crate::emendation::WordRules>,
    /// Every thing with an Inscription.
    targets: Vec<crate::emendation::Target>,
    targets_snap: Arc<Vec<crate::snapshot::TargetSnap>>,
    flights: Vec<crate::emendation::Flight>,
    landings: Vec<crate::casting::CastEvent>,
    /// Breakable things that bodies cannot walk through while they are whole: bridges.
    solid_things: Vec<u64>,
    /// Things authored broken have been broken.
    authored_broken: bool,
    /// Solid things being mended, and which of their pieces stand again.
    standing: Vec<(u64, Vec<u32>)>,
    /// The loose things, as rigid bodies.
    props: Option<crate::props::Props>,
    /// Who carries what.
    carried: Vec<(PlayerId, u64)>,
    /// The props have come to rest on the ground once, at load.
    props_settled: bool,
    /// Where each loose thing stands now, for the snapshot.
    prop_poses: Arc<Vec<(u64, glam::Affine3A)>>,
}

impl World {
    pub fn new(config: WorldConfig) -> Self {
        let rng = config.seed ^ 0x9e37_79b9_7f4a_7c15;
        let time_of_day = config.start_time_of_day.rem_euclid(24.0);
        Self {
            config,
            step: 0,
            rng,
            time_of_day,
            players: Vec::new(),
            next_player: 0,
            debris: Debris::default(),
            breakables: Vec::new(),
            broken: Arc::default(),
            letters: Arc::default(),
            letters_step: 0,
            heat: Arc::default(),
            ground: None,
            weather: Default::default(),
            solids: Vec::new(),
            trunks: Vec::new(),
            statics: None,
            readables: Arc::default(),
            seconds: 0.0,
            pace: 1.0,
            rules: Arc::default(),
            targets: Vec::new(),
            targets_snap: Arc::default(),
            flights: Vec::new(),
            landings: Vec::new(),
            solid_things: Vec::new(),
            authored_broken: false,
            standing: Vec::new(),
            props: None,
            carried: Vec::new(),
            props_settled: false,
            prop_poses: Arc::default(),
        }
    }

    /// Adds a player and their view. The list may hold any number, though the Showcase
    /// draws one.
    pub fn add_player(&mut self, feet: Vec3, yaw: f32) -> PlayerId {
        let id = PlayerId(self.next_player);
        self.next_player += 1;
        self.players.push(Player { id, body: crate::character::Body::new(feet), yaw, pitch: 0.0, reading: Default::default(), in_focus: None, perception: 0, caster: Default::default() });
        id
    }

    /// Gives the world the scene's solid boxes and the things in it that can break.
    pub fn set_scene(&mut self, solids: Vec<Solid>, breakables: Vec<BreakableThing>) {
        self.solids = solids.clone();
        self.debris.set_solids(solids);
        self.breakables = breakables;
        self.rebuild_statics();
        self.rebuild_props();
    }

    /// The loose things become rigid bodies on the still things, where they stand now.
    fn rebuild_props(&mut self) {
        let mut loose: Vec<crate::props::LooseThing> = self
            .breakables
            .iter()
            .filter(|b| b.mass > 0.0 && !self.broken.contains(&b.id))
            .map(|b| crate::props::LooseThing {
                id: b.id,
                to_world: b.to_world,
                boxes: b.boxes.iter().map(|(min, max, _)| (*min, *max)).collect(),
                mass: b.mass,
            })
            .collect();
        if loose.is_empty() {
            self.props = None;
            return;
        }
        if let Some(old) = &self.props {
            for t in &mut loose {
                if let Some(p) = old.pose(t.id) {
                    t.to_world = p;
                }
            }
        }
        let still = crate::props::Still {
            patches: self
                .ground
                .as_ref()
                .map(|g| g.patches())
                .unwrap_or_default(),
            solids: self.solids.clone(),
            trunks: self.trunks.clone(),
        };
        let first = !self.props_settled && self.ground.is_some();
        let mut props = crate::props::Props::new(Arc::new(still), loose, STEP_SECONDS);
        // At load, props stand where they would have come to rest.
        if first {
            props.settle();
            self.props_settled = true;
        }
        self.props = Some(props);
        self.prop_poses = Arc::new(self.props.as_ref().map(|p| p.poses()).unwrap_or_default());
    }

    /// What a player carries, if anything.
    pub fn carrying(&self, id: PlayerId) -> Option<u64> {
        self.carried.iter().find(|(p, _)| *p == id).map(|(_, t)| *t)
    }

    /// Gives the world the words, as compiled from their sheets.
    pub fn set_word_rules(&mut self, rules: crate::emendation::WordRules) {
        self.rules = Arc::new(rules);
    }

    pub fn word_rules(&self) -> &crate::emendation::WordRules {
        &self.rules
    }

    /// Gives the world the things that carry Inscriptions.
    pub fn set_targets(&mut self, targets: Vec<crate::emendation::Target>) {
        self.targets = targets;
        self.refresh_targets();
        self.rebuild_statics();
    }

    /// A breakable thing that bodies stand on and cannot walk through while it is whole:
    /// a marked structure, or one named so.
    fn solid(&self, id: u64) -> bool {
        self.solid_things.contains(&id) || self.targets.iter().any(|t| t.id == id && self.rules.classes.get(t.class).is_some_and(|c| c == "structure"))
    }

    /// A blow lands on a player: a full charge in either hand smudges into its typo and
    /// flies at whatever is in the gaze; a lesser charge is lost.
    pub fn strike(&mut self, id: PlayerId) {
        let Some(p) = self.players.iter_mut().find(|p| p.id == id) else { return };
        let eye = p.body.feet + Vec3::Y * p.body.eye;
        let (sy, cy) = p.yaw.sin_cos();
        let (sp, cp) = p.pitch.sin_cos();
        let gaze = Vec3::new(sy * cp, sp, -cy * cp);
        let right = Vec3::new(cy, 0.0, sy);
        for hand in [crate::hands::Hand::Main, crate::hands::Hand::Off] {
            let full = p.caster.hands.charging(hand).is_some_and(|t| t >= crate::emendation::CHARGE_FULL);
            if let (true, crate::casting::Holding::Word(w)) = (full, p.caster.holds(hand)) {
                let stroke = crate::emendation::Stroke { charge: 1.0, typo: true, ..Default::default() };
                crate::casting::send(p.id.0, &mut p.caster, w, Some(hand), stroke, &self.rules, &self.targets, eye, gaze, right, &mut self.flights);
            }
        }
        p.caster.hands.interrupt();
        p.caster.typed.clear();
    }

    /// A player learns a word, by its spelling. Returns whether there is such a word.
    pub fn learn(&mut self, id: PlayerId, spelling: &str) -> bool {
        let Some(w) = self.rules.word(spelling) else { return false };
        if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
            p.caster.learn(w);
        }
        true
    }

    /// Puts a word, by its spelling, in a player's hand: 0 main, 1 off.
    pub fn hold_word(&mut self, id: PlayerId, hand: usize, spelling: &str) {
        let Some(w) = self.rules.word(spelling) else { return };
        if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
            p.caster.learn(w);
            p.caster.holding[hand.min(1)] = crate::casting::Holding::Word(w);
        }
    }

    fn refresh_targets(&mut self) {
        self.targets_snap = Arc::new(
            self.targets
                .iter()
                .map(|t| crate::snapshot::TargetSnap {
                    id: t.id,
                    centre: t.centre,
                    radius: t.radius,
                    words: t.words.iter().map(|w| w.spelling.clone()).collect(),
                    effects: t.effects.iter().map(|e| e.effect.clone()).collect(),
                })
                .collect(),
        );
        // Words that heat and cool make things hot and cold.
        let mut all: Vec<(u64, f32)> = self.heat.iter().copied().filter(|(id, _)| !self.targets.iter().any(|t| t.id == *id)).collect();
        for t in &self.targets {
            let h = crate::casting::heat_of(t);
            if h != 0.0 {
                all.push((t.id, h));
            }
        }
        self.heat = Arc::new(all);
    }

    /// Gives the world the texts that can be read.
    pub fn set_readables(&mut self, texts: Vec<crate::reading::ReadableText>) {
        self.readables = Arc::new(texts);
    }

    /// Sets how sharp a reader a player is.
    pub fn set_perception(&mut self, id: PlayerId, perception: u32) {
        if let Some(p) = self.players.iter_mut().find(|p| p.id == id) {
            p.perception = perception;
        }
    }

    /// Gives the world the trunks of the scene's trees, which bodies cannot walk through.
    pub fn set_trunks(&mut self, trunks: Vec<crate::character::Trunk>) {
        self.trunks = trunks;
        self.rebuild_statics();
        if self.props.is_some() {
            self.rebuild_props();
        }
    }

    /// Marks breakable things that bodies stand on and cannot walk through while whole.
    pub fn set_solid_things(&mut self, ids: Vec<u64>) {
        self.solid_things = ids;
        self.rebuild_statics();
    }

    fn rebuild_statics(&mut self) {
        let patches = self
            .ground
            .as_ref()
            .map(|g| g.patches())
            .unwrap_or_default();
        if patches.is_empty()
            && self.solids.is_empty()
            && self.trunks.is_empty()
            && self.breakables.is_empty()
        {
            self.statics = None;
            return;
        }
        let mut statics = crate::character::Statics::new(&patches, &self.solids, &self.trunks);
        let solid: Vec<u64> = self
            .breakables
            .iter()
            .map(|b| b.id)
            .filter(|id| self.solid(*id))
            .collect();
        for id in solid {
            statics.set_thing(id, &self.thing_solids(id));
        }
        self.statics = Some(Arc::new(statics));
    }

    /// The boxes a solid thing stands in now, in the world: all of them while it is whole,
    /// the pieces that stand again while it is mended, and none while it lies broken.
    fn thing_solids(&self, id: u64) -> Vec<Solid> {
        let Some(b) = self.breakables.iter().find(|b| b.id == id) else {
            return Vec::new();
        };
        let standing = self.standing.iter().find(|(t, _)| *t == id).map(|(_, p)| p);
        let broken = self.broken.contains(&id);
        b.boxes
            .iter()
            .enumerate()
            .filter(|(k, _)| !broken || standing.is_some_and(|p| p.contains(&(*k as u32))))
            .map(|(_, (min, max, _))| {
                let mut lo = Vec3::splat(f32::MAX);
                let mut hi = Vec3::splat(f32::MIN);
                for corner in 0..8 {
                    let p = Vec3::new(if corner & 1 == 0 { min.x } else { max.x }, if corner & 2 == 0 { min.y } else { max.y }, if corner & 4 == 0 { min.z } else { max.z });
                    let w = b.to_world.transform_point3(p);
                    lo = lo.min(w);
                    hi = hi.max(w);
                }
                Solid { min: lo, max: hi }
            })
            .collect()
    }

    /// The statics take up what a solid thing stands in now.
    fn refresh_thing(&mut self, id: u64) {
        if !self.solid(id) {
            return;
        }
        let solids = self.thing_solids(id);
        match self.statics.as_mut().and_then(Arc::get_mut) {
            Some(statics) => statics.set_thing(id, &solids),
            None => self.rebuild_statics(),
        }
    }

    /// Things authored broken are broken now, their letters already at rest, as though
    /// they fell long ago. Their state is their Inscription.
    fn break_authored(&mut self) {
        self.authored_broken = true;
        let ids: Vec<(u64, Vec3)> = self.targets.iter().filter(|t| t.words.iter().any(|w| w.spelling == "BROKEN")).map(|t| (t.id, t.centre)).collect();
        for (id, centre) in ids {
            self.break_thing_with(id, centre + Vec3::Y * 0.5, 0.25);
        }
        self.debris.settle(STEP_SECONDS);
        self.refresh_letters();
    }

    /// The thing's Inscription says it is broken, or whole.
    fn inscribe_state(&mut self, id: u64, spelling: &str) {
        if let Some(t) = self.targets.iter_mut().find(|t| t.id == id) {
            t.words.retain(|w| w.class != "Integrity");
            t.words.push(crate::emendation::InscribedWord { spelling: spelling.into(), class: "Integrity".into(), fades_at: None, replaced: None });
            t.effects.retain(|e| e.effect != "mend");
        }
    }

    /// Breaks a thing into its letters, thrown outwards from `blow`. Returns how many
    /// letters it made, or `None` if there is no such thing or it is already broken.
    pub fn break_thing(&mut self, id: u64, blow: Vec3) -> Option<usize> {
        self.break_thing_with(id, blow, 1.0)
    }

    /// Breaks a thing with a blow of the given strength: 1 throws its letters across
    /// the yard, and near 0 it falls in on itself.
    pub fn break_thing_with(&mut self, id: u64, blow: Vec3, strength: f32) -> Option<usize> {
        if self.broken.contains(&id) {
            return None;
        }
        let mut thing = self.breakables.iter().find(|b| b.id == id)?.clone();
        // A loose thing breaks where it lies now, and its body is gone.
        if let Some(props) = &mut self.props {
            if let Some(p) = props.pose(id) {
                thing.to_world = p;
                props.remove(id);
                self.carried.retain(|(_, t)| *t != id);
                self.prop_poses = Arc::new(props.poses());
            }
        }
        let mut made = 0;
        let keep = crate::debris::keep_for(thing.boxes.iter().map(|(min, max, _)| (*min, *max)));
        for (piece, (min, max, material)) in thing.boxes.iter().enumerate() {
            let mut rng = self.rng;
            let mut random = || {
                rng = rng.wrapping_add(0x9e37_79b9_7f4a_7c15);
                let mut z = rng;
                z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                z ^ (z >> 31)
            };
            made += shatter(
                &mut self.debris,
                *min,
                *max,
                thing.to_world,
                *material,
                blow,
                strength,
                id,
                piece as u32,
                keep,
                &mut random,
            );
            self.rng = rng;
        }
        let mut broken = (*self.broken).clone();
        broken.push(id);
        self.broken = Arc::new(broken);
        self.refresh_letters();
        if self.targets.iter().any(|t| t.id == id && !t.words.iter().any(|w| w.spelling == "BROKEN")) {
            self.inscribe_state(id, "BROKEN");
            self.refresh_targets();
        }
        self.standing.retain(|(t, _)| *t != id);
        self.refresh_thing(id);
        Some(made)
    }

    /// Sets how hot or cold a thing is, -1 (frozen) to 1 (as hot as it gets). 0 lets it
    /// be. The words of S05 will set this; until then the development tools do.
    pub fn set_heat(&mut self, id: u64, heat: f32) {
        let heat = heat.clamp(-1.0, 1.0);
        let mut all: Vec<(u64, f32)> = self.heat.iter().copied().filter(|(i, _)| *i != id).collect();
        if heat != 0.0 {
            all.push((id, heat));
        }
        self.heat = Arc::new(all);
    }

    /// Gives the world ground to walk on. Without it the players walk on the level of
    /// wherever they started, as in the yard.
    /// Asks for rain, 0 to 1. It comes on and eases over some seconds.
    pub fn set_rain(&mut self, rain: f32) {
        self.weather.rain_wanted = rain.clamp(0.0, 1.0);
    }

    /// Asks for fog, 0 to 1. It gathers and lifts over a minute or so.
    pub fn set_fog(&mut self, fog: f32) {
        self.weather.fog_wanted = fog.clamp(0.0, 1.0);
    }

    /// The weather of this step.
    pub fn weather(&self) -> crate::Weather {
        self.weather.now
    }

    pub fn set_ground(&mut self, ground: Arc<dyn crate::Ground>) {
        self.debris.set_ground(ground.clone());
        self.ground = Some(ground);
        for p in &mut self.players {
            if let Some(h) = self.ground.as_ref().and_then(|g| g.height(p.body.feet.x, p.body.feet.z)) {
                p.body.feet.y = h;
            }
        }
        self.rebuild_statics();
        if self.props.is_some() {
            self.rebuild_props();
        }
    }

    pub fn loose_letters(&self) -> usize {
        self.debris.letters.len()
    }

    pub fn letters_moving(&self) -> usize {
        self.debris.awake()
    }

    fn refresh_letters(&mut self) {
        self.letters_step = self.step + 1;
        self.letters = Arc::new(
            self.debris
                .letters
                .iter()
                .map(|l| LetterSnap { position: l.position, rotation: l.rotation, material: l.material, seed: l.seed })
                .collect(),
        );
    }

    pub fn remove_player(&mut self, id: PlayerId) {
        self.players.retain(|p| p.id != id);
    }

    pub fn step_index(&self) -> u64 {
        self.step
    }

    pub fn player_count(&self) -> usize {
        self.players.len()
    }

    /// Deterministic random numbers for the rules. SplitMix64.
    pub fn next_random(&mut self) -> u64 {
        self.rng = self.rng.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.rng;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    /// Advances one fixed step. `inputs[i]` belongs to the i-th player in the order they
    /// were added. A missing input is treated as no input.
    pub fn step(&mut self, inputs: &[PlayerInput]) {
        use crate::input::buttons;
        if !self.authored_broken {
            self.break_authored();
        }
        use crate::reading::{focus, Reading, CLOSE_READING_PACE};
        // What each player reads. A text in focus is read while Read is held; with none,
        // Read is close reading.
        for (i, p) in self.players.iter_mut().enumerate() {
            let input = inputs.get(i).copied().unwrap_or_default();
            let eye = p.body.feet + Vec3::Y * p.body.eye;
            let (sy, cy) = p.yaw.sin_cos();
            let (sp, cp) = p.pitch.sin_cos();
            let gaze = Vec3::new(sy * cp, sp, -cy * cp);
            p.in_focus = focus(eye, gaze, &self.readables, self.statics.as_deref());
            p.reading = if !input.held(buttons::READ) {
                Reading::None
            } else {
                match (&p.reading, p.in_focus) {
                    // Holding on a text keeps reading it, though the eye wanders.
                    (Reading::Text(t), _) => Reading::Text(*t),
                    (Reading::None, Some(t)) => Reading::Text(t),
                    _ => Reading::Close,
                }
            };
        }
        // In solo play, close reading slows time. In co-op the shared world never slows.
        self.pace = if self.players.len() == 1 && self.players[0].reading == Reading::Close { CLOSE_READING_PACE } else { 1.0 };
        let dt = STEP_SECONDS * self.pace;
        for (i, p) in self.players.iter_mut().enumerate() {
            let input = inputs.get(i).copied().unwrap_or_default();
            p.yaw = (p.yaw + input.look_yaw).rem_euclid(std::f32::consts::TAU);
            p.pitch = (p.pitch + input.look_pitch).clamp(-MAX_PITCH, MAX_PITCH);
            p.body.step(&input, p.yaw, self.statics.as_deref(), &Around(self.ground.as_deref()), dt);
        }
        // Casting: the hands' taps and charges become words in flight, which land.
        self.landings.clear();
        let mut hand_events = Vec::new();
        for (i, p) in self.players.iter_mut().enumerate() {
            let input = inputs.get(i).copied().unwrap_or_default();
            let pressed = input.buttons & !p.caster.last_buttons;
            p.caster.last_buttons = input.buttons;
            if pressed & buttons::NEXT_WORD != 0 {
                p.caster.cycle(1);
            }
            if pressed & buttons::PREVIOUS_WORD != 0 {
                p.caster.cycle(-1);
            }
            let eye = p.body.feet + Vec3::Y * p.body.eye;
            let (sy, cy) = p.yaw.sin_cos();
            let (sp, cp) = p.pitch.sin_cos();
            let gaze = Vec3::new(sy * cp, sp, -cy * cp);
            let right = Vec3::new(cy, 0.0, sy);
            // Loose things: Interact picks one up or puts it down, and the main hand
            // throws what is carried.
            let mut throwing = false;
            if let Some(props) = &mut self.props {
                let carrying = self.carried.iter().position(|(c, _)| *c == p.id);
                match carrying {
                    Some(k) if pressed & buttons::ATTACK_MAIN != 0 => {
                        props.throw(self.carried[k].1, gaze + Vec3::Y * 0.12);
                        self.carried.swap_remove(k);
                        throwing = true;
                    }
                    Some(k) if pressed & buttons::INTERACT != 0 => {
                        self.carried.swap_remove(k);
                    }
                    Some(_) => throwing = true,
                    None if pressed & buttons::INTERACT != 0 => {
                        if let Some(t) = props
                            .pick(eye, gaze)
                            .filter(|t| !self.carried.iter().any(|(_, c)| c == t))
                        {
                            self.carried.push((p.id, t));
                        }
                    }
                    None => {}
                }
            }
            crate::casting::take_aim(&mut p.caster, eye, gaze, &self.targets);
            // Typing while a word charges.
            let charging = p.caster.hands.charging(crate::hands::Hand::Main).is_some() || p.caster.hands.charging(crate::hands::Hand::Off).is_some() || p.caster.hands.both().is_some();
            if charging {
                for k in 0..4 {
                    let c = (input.typed >> (k * 8)) & 0xff;
                    if c != 0 && p.caster.typed.len() < 16 {
                        p.caster.typed.push((c as u8 as char).to_ascii_uppercase());
                    }
                }
            }
            // While reading, the hands are still.
            let reading = p.reading != Reading::None;
            // While carrying, the main hand is full.
            p.caster.hands.step(
                input.held(buttons::ATTACK_MAIN) && !reading && !throwing,
                input.held(buttons::ATTACK_OFF) && !reading,
                &mut hand_events,
            );
            crate::casting::release(
                p.id.0,
                &mut p.caster,
                &hand_events,
                &self.rules,
                &self.targets,
                eye,
                gaze,
                right,
                &mut self.flights,
            );
            crate::casting::refill(&mut p.caster, dt);
        }
        // Carried things are drawn before the eye; then the loose things move, and any
        // jolted hard enough break.
        if let Some(props) = &mut self.props {
            let mut lost = Vec::new();
            for (k, (who, t)) in self.carried.iter().enumerate() {
                let Some(p) = self.players.iter().find(|p| p.id == *who) else {
                    continue;
                };
                let eye = p.body.feet + Vec3::Y * p.body.eye;
                let (sy, cy) = p.yaw.sin_cos();
                let (sp, cp) = p.pitch.sin_cos();
                let gaze = Vec3::new(sy * cp, sp, -cy * cp);
                if !props.carry(*t, eye + gaze * crate::props::HOLD_AT) {
                    lost.push(k);
                }
            }
            for k in lost.into_iter().rev() {
                self.carried.swap_remove(k);
            }
            props.set_dt(dt);
            let held: Vec<u64> = self.carried.iter().map(|(_, t)| *t).collect();
            let jolted = props.step(&held);
            self.prop_poses = Arc::new(props.poses());
            for (id, at) in jolted {
                self.break_thing_with(id, at, 0.5);
            }
            // What carries an Inscription goes with the thing.
            for (id, pose) in self.prop_poses.iter() {
                if let (Some(t), Some(b)) = (
                    self.targets.iter_mut().find(|t| t.id == *id),
                    self.breakables.iter().find(|b| b.id == *id),
                ) {
                    let local = b.to_world.inverse().transform_point3(t.authored_centre);
                    t.centre = pose.transform_point3(local);
                }
            }
        }
        let landed_before = self.landings.len();
        crate::casting::fly(&mut self.flights, &mut self.targets, &self.rules, dt, self.seconds, &mut self.landings);
        // The chain: twice a second, fire spreads to what is near it.
        if self.step % 30 == 0 && !self.targets.is_empty() {
            let mut rng = self.rng;
            let mut roll = || {
                rng = rng.wrapping_add(0x9e37_79b9_7f4a_7c15);
                let mut z = rng;
                z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
                z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
                ((z ^ (z >> 31)) >> 40) as f32 / (1u64 << 24) as f32
            };
            crate::casting::spread(&mut self.targets, &self.rules, self.seconds, 0.35 * self.pace, &mut roll, &mut self.landings);
            self.rng = rng;
        }
        // A word that mends landing on a broken thing calls its letters home.
        let mending: Vec<(u64, Vec3)> = self.landings[landed_before..]
            .iter()
            .filter(|l| matches!(&l.landing, crate::emendation::Landing::Took { effect, .. } if effect == "mend"))
            .filter(|l| self.broken.contains(&l.target))
            .map(|l| (l.target, self.players.iter().find(|p| p.id == PlayerId(l.player)).map_or(l.at, |p| p.body.feet)))
            .collect();
        for (id, from) in mending {
            // It knits outward from the writer's feet.
            self.debris.mend(id, from);
            if !self.standing.iter().any(|(t, _)| *t == id) {
                self.standing.push((id, Vec::new()));
            }
        }
        // Pieces whose letters are all home stand again, and bear weight.
        for k in 0..self.standing.len() {
            let id = self.standing[k].0;
            let home = self.debris.pieces_home(id);
            if home.len() > self.standing[k].1.len() {
                self.standing[k].1 = home;
                self.refresh_thing(id);
            }
        }
        // Its typo knocks a hole: a whole breakable thing falls to its letters.
        let holed: Vec<(u64, Vec3)> = self.landings[landed_before..]
            .iter()
            .filter(|l| matches!(&l.landing, crate::emendation::Landing::Took { effect, .. } if effect == "gap"))
            .filter(|l| !self.broken.contains(&l.target) && self.breakables.iter().any(|b| b.id == l.target))
            .map(|l| (l.target, l.at))
            .collect();
        for (id, at) in holed {
            self.break_thing_with(id, at, 0.6);
        }
        // Things whose letters have all come home stand whole again.
        let mended = self.debris.take_mended();
        if !mended.is_empty() {
            self.broken = Arc::new(self.broken.iter().copied().filter(|b| !mended.contains(b)).collect());
            for id in &mended {
                self.inscribe_state(*id, "WHOLE");
            }
            self.standing.retain(|(t, _)| !mended.contains(t));
            self.refresh_letters();
            self.refresh_targets();
            for id in mended {
                self.refresh_thing(id);
            }
        }
        // A breakable thing that has burned long enough falls to its letters.
        if self.step % 30 == 0 {
            let burning_long: Vec<(u64, Vec3)> = self
                .targets
                .iter_mut()
                .filter_map(|t| {
                    let alight = t.effects.iter().any(|e| matches!(e.effect.as_str(), "ignite" | "flare" | "blaze"));
                    if !alight {
                        t.alight_since = None;
                        return None;
                    }
                    let since = *t.alight_since.get_or_insert(self.seconds);
                    (self.seconds - since > BURN_TO_BREAK).then_some((t.id, t.centre))
                })
                .collect();
            for (id, centre) in burning_long {
                if self.breakables.iter().any(|b| b.id == id) && !self.broken.contains(&id) {
                    self.break_thing_with(id, centre + Vec3::Y, 0.3);
                }
            }
        }
        let faded = crate::casting::fade_all(&mut self.targets, self.seconds);
        if faded || self.landings.len() > landed_before {
            self.refresh_targets();
        }
        if self.debris.awake() > 0 {
            self.debris.step(dt);
            self.refresh_letters();
        }
        self.seconds += dt as f64;
        self.weather.step(self.seconds);
        self.time_of_day = (self.time_of_day + self.config.hours_per_second * dt).rem_euclid(24.0);
        self.step += 1;
    }

    /// The immutable picture of this step.
    pub fn snapshot(&self) -> Arc<Snapshot> {
        let players: Vec<PlayerSnap> = self
            .players
            .iter()
            .map(|p| PlayerSnap {
                id: p.id,
                feet: p.body.feet,
                yaw: p.yaw,
                pitch: p.pitch,
                grounded: p.body.grounded,
                crouching: p.body.crouching,
                swimming: p.body.swimming,
                untouchable: p.body.untouchable(),
                reading: match &p.reading {
                    crate::reading::Reading::None => crate::reading::ReadingSnap::None,
                    crate::reading::Reading::Close => crate::reading::ReadingSnap::Close,
                    crate::reading::Reading::Text(t) => {
                        let r = &self.readables[*t];
                        crate::reading::ReadingSnap::Text {
                            id: r.id,
                            text: r.text,
                            hidden: r.hidden.iter().filter(|(_, need)| p.perception >= *need).map(|(s, _)| *s).collect(),
                        }
                    }
                },
                in_focus: p.in_focus.map(|t| self.readables[t].id),
                ink: p.caster.ink,
                holding: p.caster.holding,
                charge: [
                    p.caster.charge(Some(crate::hands::Hand::Main)),
                    p.caster.charge(Some(crate::hands::Hand::Off)),
                    p.caster.charge(None),
                ],
                preview: [
                    crate::casting::preview_for(&p.caster, &self.rules, &self.targets, crate::hands::Hand::Main),
                    crate::casting::preview_for(&p.caster, &self.rules, &self.targets, crate::hands::Hand::Off),
                ],
            })
            .collect();
        let views = self
            .players
            .iter()
            .map(|p| ViewSnap {
                player: p.id,
                eye: p.body.feet + Vec3::Y * p.body.eye,
                yaw: p.yaw,
                pitch: p.pitch,
            })
            .collect();
        Arc::new(Snapshot {
            step: self.step,
            time_of_day: self.time_of_day,
            players,
            views,
            letters: self.letters.clone(),
            letters_step: self.letters_step,
            broken: self.broken.clone(),
            props: self.prop_poses.clone(),
            heat: self.heat.clone(),
            weather: self.weather.now,
            targets: self.targets_snap.clone(),
            flights: self
                .flights
                .iter()
                .map(|f| crate::snapshot::FlightSnap {
                    word: self.rules.words[f.word].spelling.clone(),
                    from: f.from,
                    at: f.at,
                    target: f.target,
                    charge: f.charge,
                    typo: if f.typo { self.rules.words[f.word].typo.clone() } else { String::new() },
                })
                .collect(),
            landings: self.landings.clone(),
        })
    }

    /// A hash of every bit of state that the rules can read. Two worlds with equal hashes
    /// will behave identically from here on.
    pub fn state_hash(&self) -> u64 {
        let mut h = Fnv64::new();
        h.u64(self.step);
        h.u64(self.rng);
        h.u64(self.seconds.to_bits());
        h.f32(self.time_of_day);
        h.f32(self.weather.now.rain);
        h.f32(self.weather.now.wetness);
        h.f32(self.weather.rain_wanted);
        h.f32(self.weather.now.fog);
        h.f32(self.weather.fog_wanted);
        h.u32(self.next_player);
        h.u32(self.players.len() as u32);
        for p in &self.players {
            h.u32(p.id.0);
            p.body.hash(&mut h);
            h.f32(p.caster.ink);
            h.f32(p.yaw);
            h.f32(p.pitch);
        }
        for t in &self.targets {
            for w in &t.words {
                for b in w.spelling.bytes() {
                    h.u32(b as u32);
                }
            }
        }
        h.u32(self.flights.len() as u32);
        for f in &self.flights {
            h.f32(f.at.x);
            h.f32(f.at.y);
            h.f32(f.at.z);
        }
        h.u32(self.debris.letters.len() as u32);
        for l in &self.debris.letters {
            for v in l.position.to_array().into_iter().chain(l.velocity.to_array()) {
                h.f32(v);
            }
        }
        h.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::input::buttons;

    fn scripted_input(world: &mut World, step: u64, player: usize) -> PlayerInput {
        let r = world.next_random();
        let t = step as f32 * 0.013 + player as f32;
        PlayerInput {
            move_x: t.sin(),
            move_y: (t * 0.7).cos(),
            look_yaw: ((r & 0xff) as f32 / 255.0 - 0.5) * 0.05,
            look_pitch: (((r >> 8) & 0xff) as f32 / 255.0 - 0.5) * 0.02,
            buttons: if step % 90 < 30 { buttons::SPRINT } else { 0 },
            typed: 0,
        }
    }

    fn run(steps: u64) -> (u64, Vec<u64>) {
        let mut w = World::new(WorldConfig { seed: 7, ..Default::default() });
        w.add_player(Vec3::ZERO, 0.0);
        w.add_player(Vec3::new(3.0, 0.0, 1.0), 1.0);
        let mut trail = Vec::new();
        for s in 0..steps {
            let inputs = [scripted_input(&mut w, s, 0), scripted_input(&mut w, s, 1)];
            w.step(&inputs);
            if s % 100 == 99 {
                trail.push(w.state_hash());
            }
        }
        (w.state_hash(), trail)
    }

    #[test]
    fn a_thousand_steps_twice_are_identical() {
        let a = run(1000);
        let b = run(1000);
        assert_eq!(a, b);
        assert_eq!(a.1.len(), 10);
    }

    #[test]
    fn different_inputs_differ() {
        assert_ne!(run(1000).0, run(999).0);
    }

    #[test]
    fn a_view_for_each_player() {
        let mut w = World::new(WorldConfig::default());
        w.add_player(Vec3::ZERO, 0.0);
        w.add_player(Vec3::X, 0.0);
        let s = w.snapshot();
        assert_eq!(s.views.len(), 2);
        assert_eq!(s.views[1].eye, Vec3::new(1.0, EYE_HEIGHT, 0.0));
    }

    #[test]
    fn forward_is_minus_z_and_yaw_turns_right() {
        let mut w = World::new(WorldConfig::default());
        w.add_player(Vec3::ZERO, 0.0);
        w.step(&[PlayerInput { move_y: 1.0, ..Default::default() }]);
        let s = w.snapshot();
        assert!(s.players[0].feet.z < 0.0);
        assert!((s.views[0].forward() - Vec3::NEG_Z).length() < 1e-5);

        let mut w = World::new(WorldConfig::default());
        w.add_player(Vec3::ZERO, std::f32::consts::FRAC_PI_2);
        w.step(&[PlayerInput { move_y: 1.0, ..Default::default() }]);
        let s = w.snapshot();
        assert!(s.players[0].feet.x > 0.0);
        assert!((s.views[0].forward() - Vec3::X).length() < 1e-5);
    }

    #[test]
    fn snapshot_is_unchanged_by_later_steps() {
        let mut w = World::new(WorldConfig::default());
        w.add_player(Vec3::ZERO, 0.0);
        let before = w.snapshot();
        let copy = (*before).clone();
        w.step(&[PlayerInput { move_y: 1.0, ..Default::default() }]);
        assert_eq!(*before, copy);
        assert_ne!(*w.snapshot(), copy);
    }
}
