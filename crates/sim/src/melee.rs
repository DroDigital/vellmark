//! Steel: the tap chain, one heavy attack, blocking, the shield bash, Stamina, and what a
//! blow does to a creature. Shallow on purpose: tap to swing in a short chain of
//! alternating swings, hold for one heavy attack that costs Stamina and can stagger, hold
//! the free hand to block, and tap from a shield's block to bash. The depth is in the
//! words and in which weapon is swung.
//!
//! Every weapon of a moveset swings the same way and differs in damage, speed, reach, and
//! its one trait. Hits give Ink.

use glam::Vec3;

use crate::hands::{Hand, HandEvent};

/// Stamina, full.
pub const STAMINA_MAX: f32 = 100.0;
/// Stamina comes back this fast, a second, once nothing has spent it for a moment.
pub const STAMINA_REFILL: f32 = 28.0;
/// The moment, in seconds.
pub const STAMINA_PAUSE: f32 = 0.8;
/// Health, full.
pub const HEALTH_MAX: f32 = 100.0;
/// What a sidestep costs.
pub const SIDESTEP_COST: f32 = 18.0;
/// What a shield bash costs.
pub const BASH_COST: f32 = 16.0;
/// Close reading drains this much a second, in solo play; half in co-op.
pub const READING_DRAIN: f32 = 9.0;
/// A tap made within this long after a swing ends carries the chain on.
pub const CHAIN_GRACE: f32 = 0.45;
/// The longest chain of light swings before it starts again.
pub const CHAIN_LENGTH: u8 = 3;
/// How long a player is staggered when a block is broken.
pub const GUARD_BROKEN: f32 = 0.8;
/// How long a bashed creature is staggered.
pub const BASH_STAGGER: f32 = 1.0;
/// How long a creature is staggered when its poise gives.
pub const STAGGER: f32 = 1.3;
/// How long hit stop holds a view, for a light hit and for a heavy one.
pub const HIT_STOP: f32 = 0.055;
pub const HIT_STOP_HEAVY: f32 = 0.11;

/// The eight weapon types of the canon that are swung. A bow is not swung.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum WeaponKind {
    Sword,
    Axe,
    Mace,
    Dagger,
    Greatsword,
    Staff,
    /// Drawn, not swung: it takes both hands.
    Bow,
}

/// What a type of weapon does in the hand.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct WeaponStats {
    pub damage: f32,
    pub heavy_damage: f32,
    /// Metres from the eye.
    pub reach: f32,
    /// The width of a swing, in radians.
    pub arc: f32,
    /// Seconds a light swing takes; a heavy one takes half again.
    pub swing: f32,
    pub heavy_cost: f32,
    /// What a blow takes from a creature's poise.
    pub stagger: f32,
    pub heavy_stagger: f32,
    pub ink: f32,
    pub heavy_ink: f32,
    /// What a blow on an unaware creature is multiplied by.
    pub sneak: f32,
    /// Hits everything in its arc, rather than the nearest.
    pub sweeps: bool,
    /// Takes both hands.
    pub two_handed: bool,
}

impl WeaponKind {
    pub fn stats(self) -> WeaponStats {
        let base = WeaponStats {
            damage: 12.0,
            heavy_damage: 28.0,
            reach: 2.3,
            arc: 1.6,
            swing: 0.46,
            heavy_cost: 24.0,
            stagger: 18.0,
            heavy_stagger: 55.0,
            ink: 3.0,
            heavy_ink: 8.0,
            sneak: 2.5,
            sweeps: false,
            two_handed: false,
        };
        match self {
            // Balanced: each hit returns more Ink.
            Self::Sword => WeaponStats { ink: 4.5, heavy_ink: 11.0, ..base },
            // Bleeds: its blows go on hurting (the bleeding is the creature's to keep).
            Self::Axe => WeaponStats { damage: 13.0, heavy_damage: 30.0, swing: 0.5, ..base },
            // Crushes: ignores part of the armour, and staggers.
            Self::Mace => WeaponStats { damage: 13.0, heavy_damage: 32.0, swing: 0.52, stagger: 26.0, heavy_stagger: 70.0, ..base },
            // Quick: the fastest swing and the highest sneak multiplier.
            Self::Dagger => WeaponStats { damage: 8.0, heavy_damage: 18.0, reach: 1.8, arc: 1.2, swing: 0.32, heavy_cost: 16.0, sneak: 5.0, ..base },
            // Sweeps: hits everything in its arc.
            Self::Greatsword => WeaponStats { damage: 20.0, heavy_damage: 46.0, reach: 2.7, arc: 2.2, swing: 0.68, heavy_cost: 32.0, stagger: 30.0, heavy_stagger: 80.0, sweeps: true, two_handed: true, ..base },
            // Holds a word and writes it from its own charge; swung, it is a stick.
            Self::Staff => WeaponStats { damage: 9.0, heavy_damage: 20.0, reach: 2.5, swing: 0.55, two_handed: true, ..base },
            // Its numbers are its arrows'; struck with, it is a stick.
            Self::Bow => WeaponStats { damage: 4.0, heavy_damage: 30.0, reach: 1.6, swing: 0.5, heavy_cost: 0.0, ink: 3.0, heavy_ink: 6.0, sneak: 3.0, two_handed: true, ..base },
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SwingKind {
    /// One of the chain, from the right (`false`) or the left.
    Light { from_left: bool },
    Heavy,
    Bash,
}

/// A swing under way.
#[derive(Clone, Debug, PartialEq)]
pub struct Swing {
    pub kind: SwingKind,
    /// Seconds since it began.
    pub t: f32,
    /// How long it takes.
    pub length: f32,
    /// The creatures it has struck, so that each is struck once.
    pub struck: Vec<u64>,
}

impl Swing {
    /// The part of the swing in which it strikes.
    fn striking(&self) -> bool {
        let p = self.t / self.length;
        (0.32..=0.6).contains(&p)
    }
}

/// One player's steel, step by step.
#[derive(Clone, Debug, PartialEq)]
pub struct Melee {
    pub weapon: WeaponKind,
    pub health: f32,
    pub stamina: f32,
    /// Seconds since Stamina was last spent.
    pub rested: f32,
    pub swing: Option<Swing>,
    /// Swings made in the chain so far.
    pub chain: u8,
    /// A tap made during a swing, to follow it.
    pub queued: bool,
    /// Seconds since the last swing ended.
    pub since: f32,
    /// The heavy attack is drawn back, since this long.
    pub drawing: Option<f32>,
    pub blocking: bool,
    /// A block may bash once.
    pub bashed: bool,
    /// Staggered, for this long yet: a broken guard.
    pub staggered: f32,
    /// Words written on the held weapon: the effect, how strong, and until when, in the
    /// rules' seconds.
    pub on_weapon: Vec<(String, f32, f64)>,
    /// The bow drawn, since this long.
    pub draw: Option<f32>,
    pub arrows: u32,
    /// An arrow let go this step: the world takes it.
    pub loosed: Option<Arrow>,
}

impl Default for Melee {
    fn default() -> Self {
        Self {
            weapon: WeaponKind::Sword,
            health: HEALTH_MAX,
            stamina: STAMINA_MAX,
            rested: STAMINA_PAUSE,
            swing: None,
            chain: 0,
            queued: false,
            since: CHAIN_GRACE,
            drawing: None,
            blocking: false,
            bashed: false,
            staggered: 0.0,
            on_weapon: Vec::new(),
            draw: None,
            arrows: QUIVER,
            loosed: None,
        }
    }
}

/// What burning does to a creature, a second, at full strength.
pub const BURN_DAMAGE: f32 = 4.0;
/// Arrows in a full quiver.
pub const QUIVER: u32 = 20;
/// A bow is fully drawn after this long.
pub const DRAW_FULL: f32 = 0.9;
/// A held draw drains Stamina this fast, a second.
pub const DRAW_DRAIN: f32 = 7.0;
/// How long a word written on a held weapon lasts, in seconds, uncharged; a full charge
/// doubles it.
pub const ON_WEAPON_SECONDS: f64 = 30.0;

/// An arrow, in flight or where it stuck.
#[derive(Clone, Debug, PartialEq)]
pub struct Arrow {
    pub player: u32,
    pub at: Vec3,
    pub velocity: Vec3,
    pub damage: f32,
    /// Where it stuck, if it has: in the ground or a wall it can be picked up.
    pub stuck: bool,
    /// The words on the bow when it was loosed: they go with it.
    pub carries: Vec<(String, f32)>,
}

/// A blow landing on a creature, as the rules report it.
#[derive(Clone, Debug, PartialEq)]
pub struct Hit {
    pub player: u32,
    pub creature: u64,
    pub at: Vec3,
    pub damage: f32,
    pub kind: SwingKind,
    pub sneak: bool,
    /// The blow broke its poise.
    pub staggered: bool,
    pub killed: bool,
    /// How long the striker's view holds.
    pub hit_stop: f32,
    /// A heavy blow shattered the frost on it.
    pub shattered: bool,
    /// The words on the weapon that struck, which pass to what it struck.
    pub carries: Vec<(String, f32)>,
}

/// A creature as steel meets it.
#[derive(Clone, Debug, PartialEq)]
pub struct Creature {
    pub id: u64,
    pub feet: Vec3,
    pub yaw: f32,
    pub radius: f32,
    pub height: f32,
    pub health: f32,
    pub max_health: f32,
    pub poise: f32,
    pub max_poise: f32,
    /// Staggered, for this long yet.
    pub staggered: f32,
    /// It knows the player is there. A blow on an unaware creature is a sneak attack.
    pub aware: bool,
    /// Its blows and their timing are its own (S03-T12); steel needs only this much.
    pub armour: f32,
    /// What the words on it do to a fight, step by step: see [`marks`].
    pub marks: u32,
    /// Its target class's key: `creature`, `inkling`.
    pub class: String,
}

/// What words written on a creature mean to steel.
pub mod marks {
    /// `OPEN`: its jaw hangs open, and its guard with it. Blows do half again.
    pub const GAPING: u32 = 1 << 0;
    /// `COLD`: slowed until brittle. A heavy blow shatters the frost for more.
    pub const SLOWED: u32 = 1 << 1;
    /// `BURNING`: it burns.
    pub const BURNING: u32 = 1 << 2;
    /// `CALM`: it will not fight, for a while.
    pub const PACIFIED: u32 = 1 << 3;
    /// `LIT`: dazzled; and a thing of ink is driven back.
    pub const DAZZLED: u32 = 1 << 4;

    /// The mark an effect makes, if any.
    pub fn of(effect: &str) -> u32 {
        match effect {
            "gape" => GAPING,
            "slow" | "rime" => SLOWED,
            "ignite" | "flare" | "blaze" | "bake" => BURNING,
            "pacify" | "rapt" => PACIFIED,
            "glow" | "repel" | "glint" => DAZZLED,
            _ => 0,
        }
    }
}

impl Creature {
    pub fn new(id: u64, feet: Vec3, radius: f32, height: f32, health: f32, poise: f32) -> Self {
        Self { id, feet, yaw: 0.0, radius, height, health, max_health: health, poise, max_poise: poise, staggered: 0.0, aware: false, armour: 0.0, marks: 0, class: "creature".into() }
    }

    pub fn alive(&self) -> bool {
        self.health > 0.0
    }

    pub fn centre(&self) -> Vec3 {
        self.feet + Vec3::Y * (self.height * 0.5)
    }
}

impl Melee {
    pub fn can_spend(&self, cost: f32) -> bool {
        self.stamina >= cost.min(STAMINA_MAX * 0.25) && self.staggered <= 0.0
    }

    /// Spends Stamina; it may go no lower than nothing.
    pub fn spend(&mut self, cost: f32) {
        self.stamina = (self.stamina - cost).max(0.0);
        self.rested = 0.0;
    }

    /// Whether a weapon in the main hand can block: a one-handed one when the other hand
    /// is empty, a two-handed one, or a shield in the other hand.
    pub fn guards(&self, off: crate::casting::Holding) -> bool {
        use crate::casting::Holding;
        matches!(off, Holding::Empty | Holding::Shield) || self.weapon.stats().two_handed
    }

    /// One step of a player's steel. `main_weapon` says the main hand holds the weapon;
    /// `off` is what the other hand holds, and `off_held` whether its button is down.
    /// Hits go to `hits`.
    #[allow(clippy::too_many_arguments)]
    pub fn step(
        &mut self,
        player: u32,
        events: &[HandEvent],
        main_weapon: bool,
        off: crate::casting::Holding,
        off_held: bool,
        eye: Vec3,
        gaze: Vec3,
        creatures: &mut [Creature],
        dt: f32,
        hits: &mut Vec<Hit>,
    ) {
        self.rested += dt;
        if self.rested >= STAMINA_PAUSE && !self.blocking && self.drawing.is_none() {
            self.stamina = (self.stamina + STAMINA_REFILL * dt).min(STAMINA_MAX);
        }
        if self.staggered > 0.0 {
            self.staggered = (self.staggered - dt).max(0.0);
            self.swing = None;
            self.drawing = None;
            self.blocking = false;
            self.queued = false;
            return;
        }
        let stats = self.weapon.stats();
        if self.weapon == WeaponKind::Bow {
            self.bow(player, events, main_weapon, eye, gaze, dt);
            return;
        }
        // The free hand guards while its button is held.
        let blocking = off_held && main_weapon && self.guards(off) && self.drawing.is_none();
        if blocking && !self.blocking {
            self.bashed = false;
        }
        self.blocking = blocking && self.swing.as_ref().is_none_or(|s| s.kind == SwingKind::Bash);
        if main_weapon {
            for e in events {
                match *e {
                    HandEvent::Tap(Hand::Main) => {
                        if self.blocking {
                            // From a shield's block, a bash, once.
                            if off == crate::casting::Holding::Shield && !self.bashed && self.swing.is_none() && self.can_spend(BASH_COST) {
                                self.spend(BASH_COST);
                                self.bashed = true;
                                self.swing = Some(Swing { kind: SwingKind::Bash, t: 0.0, length: 0.4, struck: Vec::new() });
                            }
                        } else if self.swing.is_some() {
                            self.queued = true;
                        } else {
                            self.light(stats);
                        }
                    }
                    HandEvent::HoldBegin(Hand::Main) if !self.blocking => {
                        self.drawing = Some(0.0);
                        self.queued = false;
                    }
                    HandEvent::HoldEnd(Hand::Main, _) => {
                        if self.drawing.take().is_some() && self.swing.is_none() {
                            if self.can_spend(stats.heavy_cost) {
                                self.spend(stats.heavy_cost);
                                self.swing = Some(Swing { kind: SwingKind::Heavy, t: 0.0, length: stats.swing * 1.5, struck: Vec::new() });
                                self.chain = 0;
                            } else {
                                // Too tired for a heavy blow: a light one.
                                self.light(stats);
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        if let Some(d) = &mut self.drawing {
            *d += dt;
        }
        // The swing goes on, and strikes in its middle.
        let mut ended = false;
        if let Some(swing) = &mut self.swing {
            swing.t += dt;
            if swing.striking() {
                let carries: Vec<(String, f32)> = self.on_weapon.iter().map(|(e, s, _)| (e.clone(), *s)).collect();
                strike(player, swing, &stats, &carries, eye, gaze, creatures, hits);
            }
            ended = swing.t >= swing.length;
        }
        if ended {
            self.swing = None;
            self.since = 0.0;
            if std::mem::take(&mut self.queued) {
                self.light(stats);
            }
        } else if self.swing.is_none() {
            self.since += dt;
            if self.since > CHAIN_GRACE {
                self.chain = 0;
            }
        }
    }

    /// The bow: hold to draw, let go to loose; a tap looses a weak shot. A held draw
    /// drains Stamina, and with none left the arrow goes as it is.
    fn bow(&mut self, player: u32, events: &[HandEvent], main_weapon: bool, eye: Vec3, gaze: Vec3, dt: f32) {
        self.blocking = false;
        if !main_weapon {
            self.draw = None;
            return;
        }
        let carries: Vec<(String, f32)> = self.on_weapon.iter().map(|(e, s, _)| (e.clone(), *s)).collect();
        for e in events {
            match *e {
                HandEvent::Tap(Hand::Main) if self.arrows > 0 => {
                    self.arrows -= 1;
                    self.loosed = Some(loose(player, 0.0, eye, gaze, carries.clone()));
                }
                HandEvent::HoldBegin(Hand::Main) if self.arrows > 0 => self.draw = Some(crate::hands::HOLD_SECONDS),
                HandEvent::HoldEnd(Hand::Main, _) => {
                    if let Some(drawn) = self.draw.take() {
                        self.arrows -= 1;
                        self.loosed = Some(loose(player, drawn, eye, gaze, carries.clone()));
                    }
                }
                _ => {}
            }
        }
        if let Some(d) = &mut self.draw {
            *d += dt;
            self.spend(DRAW_DRAIN * dt);
            if self.stamina <= 0.0 {
                let drawn = self.draw.take().unwrap_or(0.0);
                self.arrows -= 1;
                self.loosed = Some(loose(player, drawn, eye, gaze, carries));
            }
        }
    }

    fn light(&mut self, stats: WeaponStats) {
        if self.chain >= CHAIN_LENGTH || self.since > CHAIN_GRACE {
            self.chain = 0;
        }
        let from_left = self.chain % 2 == 1;
        self.chain += 1;
        self.swing = Some(Swing { kind: SwingKind::Light { from_left }, t: 0.0, length: stats.swing, struck: Vec::new() });
    }

    /// A blow lands on this player, from `from`, for `damage`. Returns what got through.
    /// A block faced toward the blow takes most of it, at a cost in Stamina; a guard with
    /// no Stamina left breaks, and the player staggers.
    pub fn take(&mut self, damage: f32, from: Vec3, feet: Vec3, yaw: f32, shield: bool) -> f32 {
        let mut through = damage;
        if self.blocking {
            let to = Vec3::new(from.x - feet.x, 0.0, from.z - feet.z).normalize_or_zero();
            let facing = Vec3::new(yaw.sin(), 0.0, -yaw.cos());
            if to.dot(facing) > 0.35 {
                let kept = if shield { 0.85 } else { 0.6 };
                let cost = damage * kept * 0.9;
                if self.stamina >= cost {
                    self.spend(cost);
                    through = damage * (1.0 - kept);
                } else {
                    // The guard breaks.
                    self.spend(cost);
                    self.blocking = false;
                    self.staggered = GUARD_BROKEN;
                    through = damage * (1.0 - kept) * 2.0;
                }
            }
        }
        self.health = (self.health - through).max(0.0);
        through
    }
}

/// A swing in its striking part meets what is before the eye within reach.
#[allow(clippy::too_many_arguments)]
fn strike(player: u32, swing: &mut Swing, stats: &WeaponStats, carries: &[(String, f32)], eye: Vec3, gaze: Vec3, creatures: &mut [Creature], hits: &mut Vec<Hit>) {
    let flat = Vec3::new(gaze.x, 0.0, gaze.z).normalize_or(Vec3::NEG_Z);
    let reach = if swing.kind == SwingKind::Bash { 1.7 } else { stats.reach };
    let half_arc = (stats.arc * 0.5).cos();
    let mut candidates: Vec<(f32, usize)> = creatures
        .iter()
        .enumerate()
        .filter(|(_, c)| c.alive() && !swing.struck.contains(&c.id))
        .filter_map(|(i, c)| {
            // The nearest point of its upright body to the eye.
            let y = eye.y.clamp(c.feet.y, c.feet.y + c.height);
            let near = Vec3::new(c.feet.x, y, c.feet.z);
            let to = near - eye;
            let flat_to = Vec3::new(to.x, 0.0, to.z);
            let d = flat_to.length();
            let ok = d - c.radius <= reach && (d < c.radius + 0.2 || flat_to.normalize().dot(flat) >= half_arc) && (to.y.abs() < 1.6);
            ok.then_some((d, i))
        })
        .collect();
    candidates.sort_by(|a, b| a.0.total_cmp(&b.0));
    if !stats.sweeps && swing.kind != SwingKind::Bash {
        candidates.truncate(1);
    }
    for (_, i) in candidates {
        let c = &mut creatures[i];
        swing.struck.push(c.id);
        let (damage, stagger, heavy) = match swing.kind {
            SwingKind::Light { .. } => (stats.damage, stats.stagger, false),
            SwingKind::Heavy => (stats.heavy_damage, stats.heavy_stagger, true),
            SwingKind::Bash => (2.0, 0.0, false),
        };
        let sneak = !c.aware && swing.kind != SwingKind::Bash;
        let (dealt, shattered) = wound(c, damage, if sneak { stats.sneak } else { 1.0 }, heavy, carries);
        c.aware = true;
        let mut staggered = false;
        if swing.kind == SwingKind::Bash {
            c.staggered = c.staggered.max(BASH_STAGGER);
            staggered = true;
        } else {
            c.poise -= stagger;
            if c.poise <= 0.0 {
                c.poise = c.max_poise;
                c.staggered = STAGGER;
                staggered = true;
            }
        }
        let at = c.centre();
        hits.push(Hit {
            player,
            creature: c.id,
            at,
            damage: dealt,
            kind: swing.kind,
            sneak,
            staggered,
            killed: !c.alive(),
            hit_stop: if heavy || sneak || shattered { HIT_STOP_HEAVY } else { HIT_STOP },
            shattered,
            carries: if swing.kind == SwingKind::Bash { Vec::new() } else { carries.to_vec() },
        });
    }
}

/// What a blow of `damage` does to a creature, after its sneak multiplier, the words on
/// the creature, and the words on the weapon. Returns the damage dealt, and whether a
/// heavy blow shattered its frost.
fn wound(c: &mut Creature, damage: f32, multiplier: f32, heavy: bool, carries: &[(String, f32)]) -> (f32, bool) {
    let mut dealt = damage * multiplier;
    // A staggered creature takes a quarter more.
    if c.staggered > 0.0 {
        dealt *= 1.25;
    }
    // Words open, steel finishes.
    if c.marks & marks::GAPING != 0 {
        dealt *= 1.5;
    }
    let shattered = heavy && c.marks & marks::SLOWED != 0;
    if shattered {
        dealt *= 1.6;
        c.marks &= !marks::SLOWED;
    }
    let mut armour = c.armour.clamp(0.0, 0.9);
    for (effect, strength) in carries {
        match effect.as_str() {
            // A sharp edge cuts through armour, and deeper.
            "sharpen" | "edge" => {
                armour = 0.0;
                dealt *= 1.0 + 0.15 * strength;
            }
            "bite" => dealt *= 1.0 + 0.2 * strength,
            "heat" => dealt += 3.0 * strength,
            "gilt" => dealt *= 1.05,
            _ => {}
        }
    }
    dealt *= 1.0 - armour;
    c.health = (c.health - dealt).max(0.0);
    (dealt, shattered)
}

/// Looses an arrow drawn for `drawn` seconds from the eye along the gaze.
pub fn loose(player: u32, drawn: f32, eye: Vec3, gaze: Vec3, carries: Vec<(String, f32)>) -> Arrow {
    let pull = (drawn / DRAW_FULL).clamp(0.15, 1.0);
    Arrow { player, at: eye + gaze * 0.6, velocity: gaze * (18.0 + 42.0 * pull), damage: 6.0 + 26.0 * pull, stuck: false, carries }
}

/// Arrows fly, drop, and strike: a creature takes the blow, still things keep the arrow
/// where it stuck. `blocked` gives how far along a segment something still is in the way.
pub fn fly_arrows(arrows: &mut Vec<Arrow>, creatures: &mut [Creature], blocked: &dyn Fn(Vec3, Vec3) -> Option<f32>, dt: f32, hits: &mut Vec<Hit>) {
    arrows.retain_mut(|a| {
        if a.stuck {
            return true;
        }
        a.velocity.y -= 9.81 * dt;
        let from = a.at;
        let to = from + a.velocity * dt;
        // The first creature on the segment.
        let seg = to - from;
        let mut first: Option<(f32, usize)> = None;
        for (i, c) in creatures.iter().enumerate().filter(|(_, c)| c.alive()) {
            // Against its upright axis, as a capsule.
            let steps = 8;
            for k in 0..=steps {
                let t = k as f32 / steps as f32;
                let p = from + seg * t;
                let flat = Vec3::new(p.x - c.feet.x, 0.0, p.z - c.feet.z).length();
                if flat <= c.radius && p.y >= c.feet.y && p.y <= c.feet.y + c.height {
                    if first.is_none_or(|f| t < f.0) {
                        first = Some((t, i));
                    }
                    break;
                }
            }
        }
        let wall = blocked(from, to);
        match (first, wall) {
            (Some((t, i)), w) if w.is_none_or(|w| t <= w) => {
                let c = &mut creatures[i];
                let sneak = !c.aware;
                let (dealt, shattered) = wound(c, a.damage, if sneak { WeaponKind::Bow.stats().sneak } else { 1.0 }, false, &a.carries);
                c.aware = true;
                hits.push(Hit {
                    player: a.player,
                    creature: c.id,
                    at: from + seg * t,
                    damage: dealt,
                    kind: SwingKind::Light { from_left: false },
                    sneak,
                    staggered: false,
                    killed: !c.alive(),
                    hit_stop: 0.0,
                    shattered,
                    carries: a.carries.clone(),
                });
                false
            }
            (_, Some(w)) => {
                a.at = from + seg * w;
                a.stuck = true;
                true
            }
            _ => {
                a.at = to;
                // An arrow that has fallen far below the world is lost.
                a.at.y > -200.0
            }
        }
    });
}

/// Ink a hit returns to the striker.
pub fn ink_for(hit: &Hit, weapon: WeaponKind) -> f32 {
    let s = weapon.stats();
    match hit.kind {
        SwingKind::Bash => 0.0,
        SwingKind::Heavy => s.heavy_ink,
        SwingKind::Light { .. } if hit.sneak => s.heavy_ink,
        SwingKind::Light { .. } => s.ink,
    }
}

/// Creatures recover their poise and come out of a stagger.
pub fn recover(creatures: &mut [Creature], dt: f32) {
    for c in creatures {
        c.staggered = (c.staggered - dt).max(0.0);
        if c.staggered <= 0.0 {
            c.poise = (c.poise + c.max_poise * 0.25 * dt).min(c.max_poise);
        }
    }
}
