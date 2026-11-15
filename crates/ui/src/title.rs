//! The title sequence, from the script: black; one enormous character fills the screen,
//! so large that it reads as landscape; the camera pushes in, and its strokes are made of
//! smaller characters, made of smaller still; the ink thins to dawn sky; the letters part
//! around the valley, which is real (the world drawn behind, from the title's camera
//! path); and the title assembles from rising letters, with "Press any button." under it.
//! With reduced motion it is a plain fade. All of it is quads on the interface's canvas.

use vellmark_content::{sid, Face, GlyphAtlas, StringTable};

use crate::{Canvas, Quad, Shown, TextStyle};

/// When each part begins and ends, in seconds from the start.
const ZOOM: (f32, f32) = (1.2, 15.0);
const THIN: (f32, f32) = (11.5, 16.5);
const PART: (f32, f32) = (13.0, 19.5);
const ASSEMBLE: (f32, f32) = (18.5, 24.5);
/// When "Press any button." shows, and how long leaving takes.
pub const READY: f32 = 25.0;
const LEAVE: f32 = 1.2;
/// The same, with reduced motion.
const CALM_READY: f32 = 6.0;

/// Cells across the enormous character, and across each letter below that.
const FIRST_DIVISION: f32 = 160.0;
const DIVISION: f32 = 32.0;
/// How many times larger the view grows over the push.
const PUSH: f32 = 4096.0;

const PAPER: [f32; 4] = [0.92, 0.89, 0.82, 1.0];

fn smooth(a: f32, b: f32, x: f32) -> f32 {
    let t = ((x - a) / (b - a)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A small, fixed hash: the same letters every time.
fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TitleState {
    Running,
    /// A button was pressed with the title shown: it is fading away.
    Leaving,
    Done,
}

pub struct Title {
    t: f32,
    leaving: Option<f32>,
    reduced_motion: bool,
    /// The point of the enormous character the push heads for, in its cell, 0 to 1:
    /// always on ink, at every level.
    focus: [f32; 2],
    /// The letters everything is written in: the title's own.
    weave: Vec<char>,
}

impl Title {
    pub fn new(atlas: &GlyphAtlas, strings: &StringTable, reduced_motion: bool) -> Self {
        let weave: Vec<char> = strings.text(sid!("title.name")).chars().filter(|c| !c.is_whitespace()).collect();
        let weave = if weave.is_empty() { vec!['V'] } else { weave };
        let mut title = Self { t: 0.0, leaving: None, reduced_motion, focus: [0.5, 0.5], weave };
        title.focus = title.find_focus(atlas);
        title
    }

    /// The character at a cell of a level, from the weave.
    fn letter(&self, level: u32, i: i64, j: i64) -> char {
        let h = hash((i as u32).wrapping_mul(73_856_093) ^ (j as u32).wrapping_mul(19_349_663) ^ level.wrapping_mul(83_492_791));
        self.weave[h as usize % self.weave.len()]
    }

    /// A point inside the enormous character's ink, inside its first-level letter's ink,
    /// inside its second's, and so on, near the middle of a stroke, so that the push stays
    /// on ink all the way down.
    fn find_focus(&self, atlas: &GlyphAtlas) -> [f32; 2] {
        let top = self.weave[0];
        let mut best = [0.5, 0.5];
        let mut best_score = -1;
        // The first-level cells inside the character's ink, nearest its middle first.
        for k in 0..400u32 {
            let (u, v) = (0.3 + (hash(k) % 1000) as f32 / 1000.0 * 0.4, 0.3 + (hash(k ^ 0xa5a5) % 1000) as f32 / 1000.0 * 0.4);
            let (ci, cj) = ((u * FIRST_DIVISION).floor(), (v * FIRST_DIVISION).floor());
            let centre = [(ci + 0.5) / FIRST_DIVISION, (cj + 0.5) / FIRST_DIVISION];
            if !atlas.covers(Face::Large, top, centre[0], centre[1]) {
                continue;
            }
            // Within that letter, a cell of the next level on its ink; and within that,
            // one on the next.
            let l1 = self.letter(1, ci as i64, cj as i64);
            for a in 0..DIVISION as u32 {
                for b in 0..DIVISION as u32 {
                    let (su, sv) = ((a as f32 + 0.5) / DIVISION, (b as f32 + 0.5) / DIVISION);
                    if !atlas.covers(Face::Large, l1, su, sv) {
                        continue;
                    }
                    let gi = ci as i64 * DIVISION as i64 + a as i64;
                    let gj = cj as i64 * DIVISION as i64 + b as i64;
                    let l2 = self.letter(2, gi, gj);
                    // How much ink the deeper letter has round its middle.
                    let score = (0..5).flat_map(|x| (0..5).map(move |y| (x, y))).filter(|(x, y)| atlas.covers(Face::Large, l2, 0.3 + *x as f32 * 0.1, 0.3 + *y as f32 * 0.1)).count() as i32
                        + (0..3).flat_map(|x| (0..3).map(move |y| (x, y))).filter(|(x, y)| atlas.covers(Face::Large, l1, su + (*x as f32 - 1.0) * 0.05, sv + (*y as f32 - 1.0) * 0.05)).count() as i32;
                    if score > best_score {
                        best_score = score;
                        best = [(ci + su) / FIRST_DIVISION, (cj + sv) / FIRST_DIVISION];
                    }
                }
            }
            if best_score >= 30 {
                break;
            }
        }
        best
    }

    /// A button was pressed: before the title is up it comes at once; once it is up, the
    /// sequence leaves.
    pub fn press(&mut self) {
        let ready = if self.reduced_motion { CALM_READY } else { READY };
        if self.t < ready {
            self.t = self.t.max(if self.reduced_motion { CALM_READY - 0.5 } else { ASSEMBLE.1 - 0.6 });
        } else if self.leaving.is_none() {
            self.leaving = Some(self.t);
        }
    }

    pub fn update(&mut self, dt: f32) -> TitleState {
        self.t += dt;
        match self.leaving {
            Some(at) if self.t - at >= LEAVE => TitleState::Done,
            Some(_) => TitleState::Leaving,
            None => TitleState::Running,
        }
    }

    /// Seconds since the title began: the time along its camera path.
    pub fn seconds(&self) -> f32 {
        self.t
    }

    /// How much of the world shows through: 0 black, 1 all of it.
    pub fn reveal(&self) -> f32 {
        if self.reduced_motion {
            smooth(0.5, 4.0, self.t)
        } else {
            smooth(THIN.0, THIN.1, self.t)
        }
    }

    pub fn draw(&self, canvas: &mut Canvas, atlas: &GlyphAtlas, strings: &StringTable, size: (u32, u32), density: f32) {
        let (w, h) = (size.0 as f32, size.1 as f32);
        let d = density.max(1.0);
        let fade_out = self.leaving.map_or(1.0, |at| 1.0 - smooth(0.0, LEAVE, self.t - at));
        // The black, thinning to the sky.
        let black = 1.0 - self.reveal();
        if black > 0.0 {
            canvas.rect([0.0, 0.0, w, h], [0.0, 0.0, 0.0, black]);
        }
        if !self.reduced_motion {
            self.draw_dive(canvas, atlas, w, h);
        }
        self.draw_name(canvas, atlas, strings, w, h, d, fade_out);
    }

    /// The enormous character and the letters within letters, pushed into.
    fn draw_dive(&self, canvas: &mut Canvas, atlas: &GlyphAtlas, w: f32, h: f32) {
        let fade_in = smooth(0.0, ZOOM.0, self.t);
        let gone = smooth(PART.0 + 2.0, PART.1, self.t);
        if fade_in <= 0.0 || gone >= 1.0 {
            return;
        }
        // The enormous character's cell on screen, growing as the camera pushes in.
        let push = PUSH.powf(smooth(ZOOM.0, ZOOM.1, self.t).powf(0.85));
        let p0 = h * 2.6 * push;
        let (cx, cy) = (w * 0.5, h * 0.5);
        let origin = [cx - self.focus[0] * p0, cy - self.focus[1] * p0];
        // The parting: a hole round the middle that grows, the letters at its edge pushed
        // aside.
        let hole = smooth(PART.0, PART.1, self.t) * (w.hypot(h) * 0.62);
        let top = self.weave[0];
        let mut level_size = p0 / FIRST_DIVISION;
        let mut divisions = FIRST_DIVISION;
        for level in 1..=4u32 {
            let alpha = smooth(3.0, 9.0, level_size) * (1.0 - smooth(170.0, 340.0, level_size));
            if level_size > 2.5 && alpha > 0.01 {
                // The cells of this level on screen, in this level's grid.
                let to_cell = |x: f32, o: f32| ((x - o) / level_size).floor() as i64;
                let (i0, i1) = (to_cell(0.0, origin[0]).max(0), to_cell(w, origin[0]).min(divisions as i64 - 1));
                let (j0, j1) = (to_cell(0.0, origin[1]).max(0), to_cell(h, origin[1]).min(divisions as i64 - 1));
                for j in j0..=j1 {
                    for i in i0..=i1 {
                        // Its middle, as a point of the enormous character, 0 to 1.
                        let (u, v) = ((i as f32 + 0.5) / divisions, (j as f32 + 0.5) / divisions);
                        if !self.inked(atlas, top, level, u, v) {
                            continue;
                        }
                        let mut x = origin[0] + i as f32 * level_size;
                        let mut y = origin[1] + j as f32 * level_size;
                        let (dx, dy) = (x + level_size * 0.5 - cx, y + level_size * 0.5 - cy);
                        let r = dx.hypot(dy);
                        if r < hole {
                            continue;
                        }
                        // Pushed aside by the parting, more the nearer the edge.
                        if hole > 0.0 {
                            let push = (hole * 0.35) * (-(r - hole) / (hole * 0.5 + 1.0)).exp();
                            x += dx / r.max(1.0) * push;
                            y += dy / r.max(1.0) * push;
                        }
                        let c = self.letter(level, i, j);
                        let cell = level_size;
                        canvas.quads.push(Quad {
                            rect: [x, y, cell, cell],
                            colour: [PAPER[0], PAPER[1], PAPER[2], alpha * fade_in * (1.0 - gone)],
                            layer: GlyphAtlas::layer(Face::Large, c),
                        });
                    }
                }
            }
            level_size /= DIVISION;
            divisions *= DIVISION;
        }
    }

    /// Whether a point of the enormous character, 0 to 1, is on the ink of every letter
    /// above `level` that holds it.
    fn inked(&self, atlas: &GlyphAtlas, top: char, level: u32, u: f32, v: f32) -> bool {
        if !atlas.covers(Face::Large, top, u, v) {
            return false;
        }
        let mut divisions = FIRST_DIVISION;
        for l in 1..level {
            let (gi, gj) = ((u * divisions).floor(), (v * divisions).floor());
            let parent = self.letter(l, gi as i64, gj as i64);
            if !atlas.covers(Face::Large, parent, u * divisions - gi, v * divisions - gj) {
                return false;
            }
            divisions *= DIVISION;
        }
        true
    }

    /// The title, assembled from letters rising as smoke rises, and the call to begin.
    #[allow(clippy::too_many_arguments)]
    fn draw_name(&self, canvas: &mut Canvas, atlas: &GlyphAtlas, strings: &StringTable, w: f32, h: f32, d: f32, fade: f32) {
        let name = Shown::from_table(strings, sid!("title.name"));
        let big = TextStyle { face: Face::Large, size: (h * 0.13).round(), colour: PAPER };
        let width = Canvas::measure(atlas, &big, &name);
        let (x0, base) = ((w - width) * 0.5, h * 0.36);
        let (start, end) = if self.reduced_motion { (CALM_READY - 2.5, CALM_READY - 0.5) } else { ASSEMBLE };
        // Each letter of the title is whole once its smoke has gathered.
        let face = atlas.face(Face::Large);
        let cell = big.size * face.cell_ems;
        let mut pen = x0;
        for (k, c) in name.as_str().chars().enumerate() {
            let glyph = atlas.glyph(Face::Large, c);
            let left = pen + glyph.centre * big.size - cell * 0.5;
            let top = base - cell * face.baseline;
            pen += glyph.advance * big.size;
            if c == ' ' {
                continue;
            }
            let span = end - start;
            let arrive = start + span * (0.45 + 0.07 * k as f32);
            let whole = smooth(arrive - 0.6, arrive + 0.5, self.t);
            if whole > 0.0 {
                canvas.quads.push(Quad { rect: [left, top, cell, cell], colour: [PAPER[0], PAPER[1], PAPER[2], whole * fade], layer: GlyphAtlas::layer(Face::Large, c) });
            }
            if self.reduced_motion {
                continue;
            }
            // Its smoke: small letters rising from below in a swaying column, drawn in to
            // points of the letter's ink.
            let small = (11.0 * d).max(h * 0.016);
            let small_cell = small * atlas.face(Face::Small).cell_ems;
            for n in 0..48u32 {
                let hsh = hash(n.wrapping_mul(2_654_435_761) ^ (k as u32) << 20);
                let (tu, tv) = ((hsh & 0xffff) as f32 / 65535.0, (hsh >> 16) as f32 / 65535.0);
                if !atlas.covers(Face::Large, c, tu, tv) {
                    continue;
                }
                let leaves = start + span * (k as f32 * 0.07 + (hsh % 97) as f32 / 97.0 * 0.3);
                let s = smooth(leaves, arrive, self.t);
                if s <= 0.0 || whole >= 1.0 {
                    continue;
                }
                let (tx, ty) = (left + tu * cell, top + tv * cell);
                let from_x = w * 0.5 + (k as f32 - 3.5) * w * 0.02;
                let sway = (s * 6.0 + n as f32).sin() * w * 0.02 * (1.0 - s);
                let x = from_x + (tx - from_x) * s + sway;
                let y = h * 0.98 + (ty - h * 0.98) * s;
                let letter = self.weave[(hsh as usize >> 3) % self.weave.len()];
                canvas.quads.push(Quad {
                    rect: [x - small_cell * 0.5, y - small_cell * 0.5, small_cell, small_cell],
                    colour: [PAPER[0], PAPER[1], PAPER[2], s.min(1.0 - whole) * 0.9 * fade],
                    layer: GlyphAtlas::layer(Face::Small, letter),
                });
            }
        }
        let ready = if self.reduced_motion { CALM_READY } else { READY };
        let call = smooth(ready - 0.5, ready + 0.8, self.t) * (0.75 + 0.25 * (self.t * 2.2).sin()) * fade;
        if call > 0.0 {
            let small = TextStyle { face: Face::Large, size: (22.0 * d).round(), colour: [PAPER[0], PAPER[1], PAPER[2], call] };
            let text = Shown::from_table(strings, sid!("title.press"));
            let tw = Canvas::measure(atlas, &small, &text);
            canvas.text(atlas, (w - tw) * 0.5, base + h * 0.12, &small, &text);
        }
    }
}
