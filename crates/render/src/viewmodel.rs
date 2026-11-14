//! The view model: the first player's arms and hands, and what they hold, placed each
//! frame from the rules' snapshot. Nothing here is animation data. Each hand is put
//! where a curve says it should be for the swing, the block, the draw, or the writing
//! gesture, and each forearm is reached to it from its shoulder by two-bone IK.
//!
//! View space: +X right, +Y up, -Z ahead, metres from the eye.

use glam::{EulerRot, Mat4, Quat, Vec3};
use vellmark_content::AssetId;
use vellmark_sim::casting::Holding;
use vellmark_sim::melee::{SwingKind, WeaponKind};
use vellmark_sim::PlayerSnap;

/// Where the fingertip of a writing hand rests, main then off: the charge gathers here.
pub const WRITING_TIP: [Vec3; 2] = [Vec3::new(0.19, -0.19, -0.48), Vec3::new(-0.19, -0.19, -0.48)];

const SHOULDER: [Vec3; 2] = [Vec3::new(0.21, -0.36, 0.12), Vec3::new(-0.21, -0.36, 0.12)];
const UPPER_ARM: f32 = 0.31;
const FOREARM: f32 = 0.29;
/// The wrist in a fist's space: behind the fist.
const FIST_WRIST: Vec3 = Vec3::new(0.0, 0.0, 0.07);
const ARROW_POOL: usize = 21;

fn key(part: &str, which: &str) -> AssetId {
    AssetId::from_key(&format!("kit.{part}.{which}"))
}

/// What a placed thing of the kit is: 1 drawn in first person, 2 an arrow of the world, 0
/// not of the kit.
pub fn kit_kind(k: AssetId) -> u8 {
    thread_local! {
        static KEYS: (Vec<AssetId>, Vec<AssetId>) = {
            let mut first = Vec::new();
            for (part, which) in [
                ("forearm", "main"), ("forearm", "off"), ("fist", "main"), ("fist", "off"), ("hand_open", "main"), ("hand_open", "off"),
                ("sword", "main"), ("dagger", "main"), ("axe", "main"), ("mace", "main"), ("staff", "main"), ("shield", "off"), ("bow", "off"),
                ("string", "0"), ("string", "1"),
            ] {
                first.push(key(part, which));
            }
            let arrows = (0..ARROW_POOL).map(|i| key("arrow", &i.to_string())).collect();
            (first, arrows)
        };
    }
    KEYS.with(|(first, arrows)| if first.contains(&k) { 1 } else if arrows.contains(&k) { 2 } else { 0 })
}

/// A hand's place in view space: where its grip (or wrist, for an open hand) is, and how
/// it is turned.
#[derive(Clone, Copy, Debug)]
struct Grip {
    at: Vec3,
    turn: Quat,
}

impl Grip {
    fn new(at: Vec3, pitch: f32, yaw: f32, roll: f32) -> Self {
        Self { at, turn: Quat::from_euler(EulerRot::YXZ, yaw, pitch, roll) }
    }

    fn lerp(self, other: Grip, t: f32) -> Grip {
        Grip { at: self.at.lerp(other.at, t), turn: self.turn.slerp(other.turn, t) }
    }

    fn matrix(self) -> Mat4 {
        Mat4::from_rotation_translation(self.turn, self.at)
    }
}

fn smooth(t: f32) -> f32 {
    let t = t.clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// Through three poses: wind up, strike, come back.
fn three(t: f32, rest: Grip, back: Grip, through: Grip, wind: f32, strike: f32) -> Grip {
    if t < wind {
        rest.lerp(back, smooth(t / wind))
    } else if t < strike {
        // The strike is fast at first and slows as it comes through.
        let u = (t - wind) / (strike - wind);
        back.lerp(through, 1.0 - (1.0 - u).powi(3))
    } else {
        through.lerp(rest, smooth((t - strike) / (1.0 - strike)))
    }
}

/// What the view model remembers between frames.
#[derive(Clone, Debug, Default)]
pub struct ViewModel {
    last_feet: Option<Vec3>,
    stride: f32,
    speed: f32,
    /// The charge each hand held last frame, to see a word let go.
    charged: [f32; 2],
    /// Seconds since each hand let a word go.
    flick: [f32; 2],
}

/// Every part of the kit, and where it goes this frame: `None` hides it.
pub type Placing = Vec<(AssetId, Option<Mat4>)>;

impl ViewModel {
    /// Places the kit for `player`, seen from `camera`, `dt` seconds after the last frame.
    /// `arrows` are those in the world, placed from the pool.
    pub fn place(&mut self, player: &PlayerSnap, eye: Vec3, rotation: Quat, arrows: &[vellmark_sim::melee::Arrow], seconds: f32, dt: f32) -> Placing {
        let to_world = Mat4::from_rotation_translation(rotation, eye);
        let mut out: Placing = Vec::new();
        let mut shown: Vec<(AssetId, Mat4)> = Vec::new();

        // The walk: a stride's bob and sway, by how fast the feet move.
        if let Some(last) = self.last_feet {
            let moved = Vec3::new(player.feet.x - last.x, 0.0, player.feet.z - last.z).length();
            let speed = if dt > 0.0 { moved / dt } else { 0.0 };
            self.speed += (speed.min(8.0) - self.speed) * (dt * 8.0).min(1.0);
        }
        self.last_feet = Some(player.feet);
        self.stride += self.speed * dt * 1.15;
        let amount = (self.speed / 5.0).min(1.4) * if player.grounded { 1.0 } else { 0.3 };
        let bob = Vec3::new((self.stride * 0.5).sin() * 0.010, -(self.stride).sin().abs() * 0.014, 0.0) * amount;
        // Breathing, standing still.
        let breath = Vec3::new(0.0, (seconds * 1.6).sin() * 0.003, 0.0);
        let sway = bob + breath;

        // Letting a word go: a flick of the hand that wrote it.
        for h in 0..2 {
            let now = player.charge[h].max(player.charge[2]);
            if self.charged[h] > 0.0 && now == 0.0 {
                self.flick[h] = 0.0;
            } else {
                self.flick[h] += dt;
            }
            self.charged[h] = now;
        }

        let reading = player.reading != vellmark_sim::reading::ReadingSnap::None;
        let bow = player.weapon == WeaponKind::Bow && player.holding[0] == Holding::Weapon;
        let mut grips: [Option<(Grip, bool)>; 2] = [None, None];

        if !reading {
            // The main hand.
            match player.holding[0] {
                Holding::Weapon if bow => {
                    // The drawing hand: at the string, drawn back as the bow is.
                    let pull = player.drawing.map_or(0.0, |d| smooth(d / vellmark_sim::melee::DRAW_FULL));
                    let rest = Grip::new(Vec3::new(-0.02, -0.17, -0.47), 0.0, 0.2, 1.4);
                    let drawn = Grip::new(Vec3::new(0.05, -0.09, -0.05), 0.0, 0.3, 1.4);
                    grips[0] = Some((rest.lerp(drawn, pull), true));
                }
                Holding::Weapon => {
                    grips[0] = Some((self.weapon_grip(player, seconds), true));
                }
                Holding::Word(_) => grips[0] = Some((self.writing(player, 0, seconds), false)),
                _ => grips[0] = Some((Grip::new(Vec3::new(0.30, -0.44, -0.36), 0.3, 0.0, 0.2), true)),
            }
            // The off hand.
            match player.holding[1] {
                _ if bow => grips[1] = Some((Grip::new(Vec3::new(-0.08, -0.15, -0.55), 0.05, -0.08, -0.32), true)),
                Holding::Shield => grips[1] = Some((self.shield_grip(player), true)),
                Holding::Word(_) if player.holding[0] == Holding::Weapon && player.charge[2] > 0.0 => {
                    // Both hands, writing on the blade: the fingertip runs along it.
                    let blade = grips[0].map_or(Grip::new(Vec3::ZERO, 0.0, 0.0, 0.0), |g| g.0);
                    let along = blade.at + blade.turn * Vec3::new(0.0, 0.25 + 0.2 * (seconds * 2.5).sin().abs() * player.charge[2], 0.0);
                    let tip_to_wrist = Vec3::new(0.035, 0.0, 0.18);
                    grips[1] = Some((Grip { at: along + tip_to_wrist, turn: Quat::from_euler(EulerRot::YXZ, -0.3, -0.2, 0.1) }, false));
                }
                Holding::Word(_) => grips[1] = Some((self.writing(player, 1, seconds), false)),
                Holding::Weapon => grips[1] = Some((Grip::new(Vec3::new(-0.26, -0.30, -0.42), -0.55, 0.0, 0.25), true)),
                Holding::Empty if player.blocking => grips[1] = Some((Grip::new(Vec3::new(-0.16, -0.26, -0.40), 0.3, 0.3, -0.3), true)),
                Holding::Empty => grips[1] = Some((Grip::new(Vec3::new(-0.30, -0.44, -0.36), 0.3, 0.0, -0.2), true)),
            }
        }

        for (h, side) in [(0usize, "main"), (1, "off")] {
            let Some((grip, fist)) = grips[h] else { continue };
            let grip = Grip { at: grip.at + sway * if h == 0 { 1.0 } else { 0.8 }, ..grip };
            let hand = grip.matrix();
            let wrist = if fist { grip.at + grip.turn * FIST_WRIST } else { grip.at };
            shown.push((key(if fist { "fist" } else { "hand_open" }, side), hand));
            // The forearm, from the wrist toward an elbow reached from the shoulder.
            let elbow = reach(SHOULDER[h], wrist, if h == 0 { 1.0 } else { -1.0 });
            let along = (elbow - wrist).normalize_or(Vec3::Z);
            let up = grip.turn * Vec3::Y;
            let turn = look(along, up);
            shown.push((key("forearm", side), Mat4::from_rotation_translation(turn, wrist)));
            // What the hand holds.
            if h == 0 && player.holding[0] == Holding::Weapon && !bow {
                let part = match player.weapon {
                    WeaponKind::Sword => "sword",
                    WeaponKind::Axe => "axe",
                    WeaponKind::Mace => "mace",
                    WeaponKind::Dagger => "dagger",
                    WeaponKind::Greatsword => "sword",
                    WeaponKind::Staff => "staff",
                    WeaponKind::Bow => "bow",
                };
                shown.push((key(part, "main"), hand));
            }
            if h == 1 && player.holding[1] == Holding::Shield {
                shown.push((key("shield", "off"), hand));
            }
            if h == 1 && bow {
                shown.push((key("bow", "off"), hand));
                // The string, from each tip to the drawing hand's fingers, and the arrow on it.
                let nock = grips[0].map_or(grip.at, |g| g.0.at + sway + g.0.turn * Vec3::new(0.0, 0.0, -0.02));
                for (i, tip) in [Vec3::new(0.0, 0.555, 0.04), Vec3::new(0.0, -0.555, 0.04)].into_iter().enumerate() {
                    let tip = hand.transform_point3(tip);
                    let d = nock - tip;
                    let len = d.length().max(1e-3);
                    let turn = Quat::from_rotation_arc(Vec3::Y, d / len);
                    shown.push((key("string", &i.to_string()), Mat4::from_scale_rotation_translation(Vec3::new(1.0, len, 1.0), turn, tip)));
                }
                if player.quiver > 0 {
                    let rest = hand.transform_point3(Vec3::new(-0.012, 0.0, -0.05));
                    let dir = (rest - nock).normalize_or(Vec3::NEG_Z);
                    let turn = Quat::from_rotation_arc(Vec3::Y, dir);
                    shown.push((key("arrow", &(ARROW_POOL - 1).to_string()), Mat4::from_rotation_translation(turn, nock)));
                }
            }
        }

        // Every part of the kit: shown where placed, in the world, or hidden.
        for (k, m) in &shown {
            out.push((*k, Some(to_world * *m)));
        }
        for (part, which) in [
            ("forearm", "main"), ("forearm", "off"), ("fist", "main"), ("fist", "off"), ("hand_open", "main"), ("hand_open", "off"),
            ("sword", "main"), ("dagger", "main"), ("axe", "main"), ("mace", "main"), ("staff", "main"), ("shield", "off"), ("bow", "off"),
            ("string", "0"), ("string", "1"),
        ] {
            let k = key(part, which);
            if !shown.iter().any(|(s, _)| *s == k) {
                out.push((k, None));
            }
        }
        // Arrows in the world, from the pool; the last of the pool is the one on the string.
        let nocked = key("arrow", &(ARROW_POOL - 1).to_string());
        for i in 0..ARROW_POOL - 1 {
            let k = key("arrow", &i.to_string());
            out.push((k, arrows.get(i).map(|a| {
                let dir = a.velocity.normalize_or(Vec3::NEG_Z);
                // Its tip where it is: stuck, a hand's breadth of it in.
                let tail = a.at - dir * if a.stuck { 0.64 } else { 0.76 };
                Mat4::from_rotation_translation(Quat::from_rotation_arc(Vec3::Y, dir), tail)
            })));
        }
        if !shown.iter().any(|(s, _)| *s == nocked) {
            out.push((nocked, None));
        }
        out
    }

    /// The weapon hand through its swings, its heavy draw, and its block.
    fn weapon_grip(&self, player: &PlayerSnap, seconds: f32) -> Grip {
        let rest = Grip::new(Vec3::new(0.27, -0.33, -0.46), -0.75, 0.1, 0.42);
        if player.staggered {
            return rest.lerp(Grip::new(Vec3::new(0.32, -0.46, -0.30), 0.2, 0.3, -0.6), 0.8);
        }
        if player.blocking && player.holding[1] != Holding::Shield {
            // The blade held across, edge out.
            return Grip::new(Vec3::new(0.10, -0.16, -0.44), -0.2, 0.0, 1.35);
        }
        if let Some((kind, t)) = player.swing {
            return match kind {
                SwingKind::Light { from_left: false } => three(
                    t,
                    rest,
                    Grip::new(Vec3::new(0.40, -0.10, -0.32), 0.25, 0.9, -1.15),
                    Grip::new(Vec3::new(-0.26, -0.34, -0.56), -0.9, -0.95, 1.35),
                    0.32,
                    0.6,
                ),
                SwingKind::Light { from_left: true } => three(
                    t,
                    rest,
                    Grip::new(Vec3::new(-0.12, -0.12, -0.36), 0.2, -0.8, 1.2),
                    Grip::new(Vec3::new(0.42, -0.36, -0.52), -0.9, 0.9, -1.35),
                    0.32,
                    0.6,
                ),
                SwingKind::Heavy => three(
                    t,
                    Grip::new(Vec3::new(0.26, -0.02, -0.38), 1.0, 0.2, -0.35),
                    Grip::new(Vec3::new(0.25, 0.02, -0.34), 1.15, 0.2, -0.35),
                    Grip::new(Vec3::new(0.04, -0.46, -0.62), -1.45, 0.0, 0.0),
                    0.3,
                    0.58,
                ),
                SwingKind::Bash => rest,
            };
        }
        if let Some(d) = player.drawing {
            // The heavy attack drawn back over the shoulder, and held, trembling a little.
            let up = smooth(d / 0.25);
            let shake = Vec3::new((seconds * 31.0).sin(), (seconds * 37.0).sin(), 0.0) * 0.002 * smooth((d - 0.4) / 0.6);
            let drawn = Grip::new(Vec3::new(0.26, -0.02, -0.38) + shake, 1.0, 0.2, -0.35);
            return rest.lerp(drawn, up);
        }
        rest
    }

    /// The shield: carried low, raised square to block, and thrust out to bash.
    fn shield_grip(&self, player: &PlayerSnap) -> Grip {
        let low = Grip::new(Vec3::new(-0.30, -0.36, -0.42), 0.15, 0.45, -0.1);
        let up = Grip::new(Vec3::new(-0.15, -0.24, -0.46), 0.05, 0.12, 0.0);
        if let Some((SwingKind::Bash, t)) = player.swing {
            let out = Grip::new(Vec3::new(-0.04, -0.15, -0.62), 0.0, 0.0, 0.0);
            return three(t, up, up, out, 0.1, 0.45);
        }
        if player.blocking {
            up
        } else {
            low
        }
    }

    /// A hand that holds a word: open, the forefinger ready. Charging, it writes small
    /// strokes, faster and wider as the charge builds; letting go, it flicks forward.
    fn writing(&self, player: &PlayerSnap, h: usize, seconds: f32) -> Grip {
        let charge = player.charge[h].max(player.charge[2]);
        let side = if h == 0 { 1.0 } else { -1.0 };
        let tip = WRITING_TIP[h];
        let mut at = tip + Vec3::new(0.035 * side, 0.0, 0.18);
        if charge > 0.0 {
            let r = 0.008 + 0.014 * charge;
            let w = 9.0 + 6.0 * charge;
            at += Vec3::new((seconds * w).cos() * r, (seconds * w * 2.0).sin() * r * 0.6 + 0.02 * charge, 0.0);
        } else {
            // Resting lower, until it is used.
            at += Vec3::new(0.02 * side, -0.05, 0.03);
        }
        let flick = self.flick[h];
        if flick < 0.18 {
            at += Vec3::new(0.0, 0.02, -0.08) * (1.0 - flick / 0.18) * (flick / 0.04).min(1.0);
        }
        Grip::new(at, -0.25 - 0.2 * charge, -0.25 * side, 0.15 * side)
    }
}

/// The elbow of an arm from `shoulder` to `wrist`, bent outward and down.
fn reach(shoulder: Vec3, wrist: Vec3, side: f32) -> Vec3 {
    let d = wrist - shoulder;
    let len = d.length().clamp(0.05, UPPER_ARM + FOREARM - 0.001);
    let dir = d.normalize_or(Vec3::NEG_Z);
    // The law of cosines: how far along, and how far out, the elbow sits.
    let a = (UPPER_ARM * UPPER_ARM - FOREARM * FOREARM + len * len) / (2.0 * len);
    let h = (UPPER_ARM * UPPER_ARM - a * a).max(0.0).sqrt();
    let pole = Vec3::new(side * 0.7, -1.0, 0.2).normalize();
    let bend = (pole - dir * pole.dot(dir)).normalize_or(Vec3::NEG_Y);
    shoulder + dir * a + bend * h
}

/// A turn that points +Z along `forward`, with +Y as near `up` as it can be.
fn look(forward: Vec3, up: Vec3) -> Quat {
    let z = forward.normalize_or(Vec3::Z);
    let x = up.cross(z).normalize_or(Vec3::X);
    let y = z.cross(x);
    Quat::from_mat3(&glam::Mat3::from_cols(x, y, z))
}
