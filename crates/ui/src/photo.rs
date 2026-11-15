//! Photo mode: a free camera within a few metres of the player, its field of view, tilt,
//! and exposure; the frame as it looks, as text, or split between the two by a divider
//! that can be moved and turned; the hands shown or hidden; and the picture saved. The
//! panel is a page of the interface kit, drawn beside the frame and never in the picture.

use glam::{Quat, Vec3};
use vellmark_content::{sid, Face, GlyphAtlas, StringId, StringTable};

use crate::kit::{self, Focus};
use crate::menu::{Entry, MenuInput, MenuRepeat, Value};
use crate::{Canvas, Shown, TextStyle};

/// How far the camera may go from the player's eye, in metres.
pub const REACH: f32 = 4.0;
/// Metres a second the camera flies.
const SPEED: f32 = 1.6;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum View {
    /// As the eye sees it.
    Real,
    /// As text, as close reading shows it.
    Text,
    /// Realistic on one side of the divider, text on the other.
    Split,
}

/// What a press in the panel asked for.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PhotoOutcome {
    None,
    Save,
    Leave,
}

pub struct Photo {
    /// Where the camera is, from the player's eye.
    pub offset: Vec3,
    pub yaw: f32,
    pub pitch: f32,
    /// Tilt, in degrees.
    pub roll: f32,
    /// Vertical field of view, in degrees.
    pub fov: f32,
    /// Exposure, in stops from the hour's own.
    pub stops: f32,
    pub view: View,
    /// The divider's turn in degrees, and its place across the frame, -0.45 to 0.45.
    pub divider_turn: f32,
    pub divider_place: f32,
    pub hands: bool,
    /// The panel is shown; it hides itself while the picture is taken.
    pub panel: bool,
    pub focus: Focus,
    pub repeat: MenuRepeat,
}

impl Photo {
    /// Opens at the player's own view.
    pub fn open(yaw: f32, pitch: f32) -> Self {
        Self {
            offset: Vec3::ZERO,
            yaw,
            pitch,
            roll: 0.0,
            fov: 65.0,
            stops: 0.0,
            view: View::Real,
            divider_turn: 0.0,
            divider_place: 0.0,
            hands: false,
            panel: true,
            focus: Focus::default(),
            repeat: MenuRepeat::default(),
        }
    }

    pub fn entries(&self) -> Vec<Entry> {
        let e = |label: StringId, value: Value| Entry { label, value };
        let view = match self.view {
            View::Real => sid!("ui.photo.view.real"),
            View::Text => sid!("ui.photo.view.text"),
            View::Split => sid!("ui.photo.view.split"),
        };
        let slider = |value: f32, min: f32, max: f32, shown: f32| Value::Slider { value, min, max, shown };
        let mut out = vec![
            e(sid!("ui.photo.view"), Value::Choice(view)),
            e(sid!("ui.photo.fov"), slider(self.fov, 20.0, 110.0, 1.0)),
            e(sid!("ui.photo.tilt"), slider(self.roll + 45.0, 0.0, 90.0, 1.0)),
            e(sid!("ui.photo.exposure"), slider(self.stops + 3.0, 0.0, 6.0, 1.0)),
        ];
        if self.view == View::Split {
            out.push(e(sid!("ui.photo.divider_turn"), slider(self.divider_turn + 90.0, 0.0, 180.0, 1.0)));
            out.push(e(sid!("ui.photo.divider_place"), slider(self.divider_place + 0.5, 0.0, 1.0, 100.0)));
        }
        out.push(e(sid!("ui.photo.hands"), Value::Switch(self.hands)));
        out.push(e(sid!("ui.photo.save"), Value::Button));
        out.push(e(sid!("ui.photo.leave"), Value::Button));
        out
    }

    /// Acts on the panel's input.
    pub fn update(&mut self, input: MenuInput) -> PhotoOutcome {
        if input.back {
            return PhotoOutcome::Leave;
        }
        let entries = self.entries();
        self.focus.step(input.step, entries.len(), entries.len());
        let label = entries[self.focus.at].label;
        let by = input.change as f32;
        if label == sid!("ui.photo.view") && (input.change != 0 || input.confirm) {
            let order = [View::Real, View::Text, View::Split];
            let at = order.iter().position(|v| *v == self.view).unwrap_or(0) as i32;
            let step = if input.change != 0 { input.change } else { 1 };
            self.view = order[(at + step).rem_euclid(3) as usize];
        } else if label == sid!("ui.photo.fov") {
            self.fov = (self.fov + by * 5.0).clamp(20.0, 110.0);
        } else if label == sid!("ui.photo.tilt") {
            self.roll = (self.roll + by * 3.0).clamp(-45.0, 45.0);
        } else if label == sid!("ui.photo.exposure") {
            self.stops = (self.stops + by * 0.25).clamp(-3.0, 3.0);
        } else if label == sid!("ui.photo.divider_turn") {
            self.divider_turn = (self.divider_turn + by * 5.0).clamp(-90.0, 90.0);
        } else if label == sid!("ui.photo.divider_place") {
            self.divider_place = (self.divider_place + by * 0.05).clamp(-0.45, 0.45);
        } else if label == sid!("ui.photo.hands") && (input.change != 0 || input.confirm) {
            self.hands = !self.hands;
        } else if label == sid!("ui.photo.save") && input.confirm {
            return PhotoOutcome::Save;
        } else if label == sid!("ui.photo.leave") && input.confirm {
            return PhotoOutcome::Leave;
        }
        PhotoOutcome::None
    }

    /// Flies the camera: `along` is strafe, rise, and advance, each -1 to 1; `look` turns
    /// it, in radians. It never goes further than `REACH` from the player's eye, nor into
    /// the ground, which `floor` gives under a point, as a height above the eye.
    pub fn fly(&mut self, along: Vec3, look: (f32, f32), dt: f32, floor: &dyn Fn(Vec3) -> f32) {
        self.yaw = (self.yaw + look.0).rem_euclid(std::f32::consts::TAU);
        self.pitch = (self.pitch + look.1).clamp(-1.5, 1.5);
        let (sy, cy) = self.yaw.sin_cos();
        let forward = Vec3::new(sy, 0.0, -cy);
        let right = Vec3::new(cy, 0.0, sy);
        let mut at = self.offset + (right * along.x + Vec3::Y * along.y + forward * along.z) * SPEED * dt;
        if at.length() > REACH {
            at = at.normalize() * REACH;
        }
        at.y = at.y.max(floor(at) + 0.15);
        self.offset = at;
    }

    /// The camera's rotation: turned, tilted up or down, and tilted to the side.
    pub fn rotation(&self) -> Quat {
        Quat::from_rotation_y(-self.yaw) * Quat::from_rotation_x(self.pitch) * Quat::from_rotation_z(-self.roll.to_radians())
    }

    /// The divider for the renderer, as its angle in radians and its offset in pixels from
    /// the middle, for a frame of `size`; none unless the frame is split.
    pub fn split(&self, size: (u32, u32)) -> Option<(f32, f32)> {
        (self.view == View::Split).then(|| {
            let angle = self.divider_turn.to_radians();
            let (w, h) = (size.0 as f32, size.1 as f32);
            let across = (angle.cos() * w).abs() + (angle.sin() * h).abs();
            (angle, self.divider_place * across)
        })
    }

    /// The panel, beside the frame, and the divider drawn in letters along its line.
    pub fn draw(&self, canvas: &mut Canvas, atlas: &GlyphAtlas, strings: &StringTable, size: (u32, u32), density: f32, hint: Option<Shown<'_>>) {
        let d = density.max(1.0);
        let (w, h) = (size.0 as f32, size.1 as f32);
        if let Some((angle, offset)) = self.split(size) {
            // The divider: a rule of characters along the line where the frame turns to text.
            let style = TextStyle { face: Face::Small, size: (12.0 * d).round(), colour: [1.0, 0.95, 0.8, 0.85] };
            let mark = Shown::from_table(strings, sid!("ui.photo.divider_mark"));
            let (n, t) = ((angle.cos(), angle.sin()), (-angle.sin(), angle.cos()));
            let centre = (w * 0.5 + n.0 * offset, h * 0.5 + n.1 * offset);
            let step = 14.0 * d;
            let reach = w.hypot(h);
            let mut s = -reach;
            while s < reach {
                let (x, y) = (centre.0 + t.0 * s, centre.1 + t.1 * s);
                if (0.0..w).contains(&x) && (0.0..h).contains(&y) {
                    canvas.text(atlas, x - 3.0 * d, y + 4.0 * d, &style, &mark);
                }
                s += step;
            }
        }
        if !self.panel {
            return;
        }
        let entries = self.entries();
        let pad = 22.0 * d;
        let row = 18.0 * d;
        let pw = (380.0 * d).min(w * 0.4);
        let style = TextStyle { face: Face::Large, size: row.round(), colour: kit::ink() };
        let line = Canvas::line_height(atlas, &style) * 1.15;
        let ph = entries.len() as f32 * line + pad * 2.0 + 60.0 * d;
        let panel = [w - pw - 24.0 * d, 24.0 * d, pw, ph];
        kit::panel(canvas, atlas, strings, panel, d);
        let heading = TextStyle { face: Face::Large, size: (22.0 * d).round(), colour: kit::ink() };
        canvas.text(atlas, panel[0] + pad, panel[1] + pad + 14.0 * d, &heading, &Shown::from_table(strings, sid!("ui.photo.title")));
        let list = [panel[0] + pad, panel[1] + pad + 30.0 * d, pw - pad * 2.0, entries.len() as f32 * line + 4.0];
        crate::menu::MenuView { atlas, strings, size, density }.rows_in(canvas, list, &entries, self.focus, 18.0);
        if let Some(hint) = hint {
            let small = TextStyle { face: Face::Small, size: (14.0 * d).round(), colour: kit::faint() };
            canvas.text(atlas, panel[0] + pad, panel[1] + ph - pad * 0.5, &small, &hint);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_camera_stays_within_reach_and_above_the_ground() {
        let mut p = Photo::open(0.0, 0.0);
        for _ in 0..600 {
            p.fly(Vec3::new(0.3, -1.0, 1.0), (0.0, 0.0), 1.0 / 60.0, &|_| -1.5);
        }
        assert!(p.offset.length() <= REACH + 1e-4);
        assert!(p.offset.y >= -1.35 - 1e-4, "kept above the ground: {}", p.offset.y);
    }

    #[test]
    fn the_divider_moves_and_turns_only_in_a_split_frame() {
        let mut p = Photo::open(0.0, 0.0);
        assert_eq!(p.split((1000, 500)), None);
        p.view = View::Split;
        p.divider_place = 0.2;
        let (angle, offset) = p.split((1000, 500)).unwrap();
        assert_eq!(angle, 0.0);
        assert!((offset - 200.0).abs() < 1e-3);
        // Through the panel: across to the divider's turn, and turn it.
        p.focus = Focus { at: 4, scroll: 0 };
        p.update(MenuInput { change: 1, ..Default::default() });
        assert_eq!(p.divider_turn, 5.0);
    }
}
