//! Materials: colour, normal, roughness, and height maps for every surface, and what
//! each surface is written in.
//!
//! Maps come from one of two places. The approved texture sources are CC0 photographs,
//! fetched once the owner has approved the list in `third_party/DOWNLOADS.md`. Until
//! then, and for anything those sources lack, maps are generated here from noise. Both
//! paths end in the same asset, so the renderer never knows which was used.

use crate::{err, BakeError};
use serde::Deserialize;
use std::path::Path;
use vellmark_content::{AssetId, Material, MaterialSet, Lexicon};

#[derive(Deserialize, Clone, Copy, Debug)]
enum Generator {
    Masonry,
    Timber,
    Cloth,
    Iron,
    Earth,
    Glass,
    Grass,
    Straw,
    Gravel,
    Mud,
    Moss,
    Rock,
    Paper,
    Water,
}

#[derive(Deserialize)]
struct Authored {
    key: String,
    generator: Generator,
    tile_metres: f32,
    family: String,
    contrast: f32,
    relief: f32,
    #[serde(default)]
    emissive: f32,
}

/// One material's words, from `lexicon.ron`.
#[derive(Deserialize, Clone)]
pub struct AuthoredLexicon {
    pub key: String,
    pub name: String,
    pub elements: Vec<String>,
    pub hot: Vec<String>,
    pub cold: Vec<String>,
}

/// Reads every material's lexicon, the one place the words are kept.
pub fn read_lexicons(file: &Path) -> Result<Vec<AuthoredLexicon>, BakeError> {
    let text = std::fs::read_to_string(file).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
    let all: Vec<AuthoredLexicon> = ron::from_str(&text).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
    let mut seen = std::collections::BTreeSet::new();
    for l in &all {
        if !seen.insert(l.key.clone()) {
            return err(format!("{}: `{}` is listed twice", file.display(), l.key));
        }
    }
    Ok(all)
}

#[derive(Deserialize)]
struct AuthoredSet {
    size: u32,
    materials: Vec<Authored>,
}

/// One texel of a generated material, before it is packed.
#[derive(Clone, Copy)]
struct Texel {
    /// Linear colour.
    colour: [f32; 3],
    /// 0 to 1, where 1 is the surface's highest point.
    height: f32,
    roughness: f32,
    metal: f32,
}

// ---- noise, all of it periodic so that maps tile ----

fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

fn hash2(x: i32, y: i32, seed: u32) -> f32 {
    let h = hash((x as u32).wrapping_mul(0x9e37_79b1) ^ hash((y as u32).wrapping_add(seed.wrapping_mul(0x85eb_ca6b))));
    (h >> 8) as f32 / 16_777_216.0
}

fn smooth(t: f32) -> f32 {
    t * t * (3.0 - 2.0 * t)
}

/// Several octaves of noise over the unit square, 0 to 1. `fx` and `fy` are the cells
/// across the square at the first octave.
fn fbm(u: f32, v: f32, fx: i32, fy: i32, octaves: u32, seed: u32) -> f32 {
    let (mut sum, mut weight, mut total) = (0.0, 1.0, 0.0);
    for o in 0..octaves {
        let s = 1 << o;
        // Each octave has its own period so that the whole stays periodic over the square.
        let n = noise_xy(u * (fx * s) as f32, v * (fy * s) as f32, fx * s, fy * s, seed + o * 31);
        sum += n * weight;
        total += weight;
        weight *= 0.5;
    }
    sum / total
}

fn noise_xy(x: f32, y: f32, period_x: i32, period_y: i32, seed: u32) -> f32 {
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (smooth(x - xi as f32), smooth(y - yi as f32));
    let at = |dx: i32, dy: i32| hash2((xi + dx).rem_euclid(period_x), (yi + dy).rem_euclid(period_y), seed);
    let top = at(0, 0) + (at(1, 0) - at(0, 0)) * fx;
    let bottom = at(0, 1) + (at(1, 1) - at(0, 1)) * fx;
    top + (bottom - top) * fy
}

/// Distance to the nearest of a periodic scatter of points, and that point's own
/// random number. `cells` points lie across the unit square each way.
fn cellular(u: f32, v: f32, cells: i32, seed: u32) -> (f32, f32) {
    let (x, y) = (u * cells as f32, v * cells as f32);
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let mut best = (f32::MAX, 0.0);
    for dy in -1..=1 {
        for dx in -1..=1 {
            let (cx, cy) = (xi + dx, yi + dy);
            let (wx, wy) = (cx.rem_euclid(cells), cy.rem_euclid(cells));
            let px = cx as f32 + hash2(wx, wy, seed);
            let py = cy as f32 + hash2(wx, wy, seed + 7);
            let d = ((px - x).powi(2) + (py - y).powi(2)).sqrt();
            if d < best.0 {
                best = (d, hash2(wx, wy, seed + 13));
            }
        }
    }
    best
}

fn mix3(a: [f32; 3], b: [f32; 3], t: f32) -> [f32; 3] {
    [a[0] + (b[0] - a[0]) * t, a[1] + (b[1] - a[1]) * t, a[2] + (b[2] - a[2]) * t]
}

fn scale3(a: [f32; 3], s: f32) -> [f32; 3] {
    [a[0] * s, a[1] * s, a[2] * s]
}

fn step(edge0: f32, edge1: f32, x: f32) -> f32 {
    smooth(((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0))
}

// ---- the generators ----

/// Coursed stone: eight courses to the tile, four stones to a course, laid half over
/// half, with recessed mortar and worn edges.
fn masonry(u: f32, v: f32) -> Texel {
    const COURSES: i32 = 8;
    const STONES: i32 = 4;
    let row = (v * COURSES as f32).floor() as i32;
    let rv = v * COURSES as f32 - row as f32;
    let shift = if row % 2 == 0 { 0.0 } else { 0.5 };
    let x = u * STONES as f32 + shift;
    let col = x.floor() as i32;
    // Each joint is nudged a little, the same for the stone on either side of it.
    let joint = |c: i32| c as f32 + (hash2(c.rem_euclid(STONES), row, 5) - 0.5) * 0.22;
    let (left, right) = (joint(col), joint(col + 1));
    let (col, left, right) = if x < left {
        (col - 1, joint(col - 1), left)
    } else if x > right {
        (col + 1, right, joint(col + 2))
    } else {
        (col, left, right)
    };
    // Distance to the stone's edge, in metres (a tile is 2 m: a course is 0.25 m).
    let dx = (x - left).min(right - x) * 0.5;
    let dy = rv.min(1.0 - rv) * 0.25;
    let wobble = (fbm(u, v, 16, 16, 3, 11) - 0.5) * 0.012;
    let edge = dx.min(dy) + wobble;
    let stone = step(0.006, 0.016, edge);

    let id = hash2(col.rem_euclid(STONES), row, 21);
    let mottle = fbm(u, v, 24, 24, 4, 3);
    let pits = fbm(u, v, 96, 96, 2, 9);
    let tone = 0.80 + id * 0.34 + (mottle - 0.5) * 0.30;
    let warm = hash2(col.rem_euclid(STONES), row, 33);
    let base = mix3([0.33, 0.32, 0.30], [0.40, 0.35, 0.28], warm);
    let stone_colour = scale3(base, tone * (0.92 + pits * 0.16));
    let mortar_colour = scale3([0.46, 0.43, 0.38], 0.85 + mottle * 0.3);

    let rounded = step(0.006, 0.05, edge);
    let face = 0.62 + rounded * 0.30 + (mottle - 0.5) * 0.10 + (pits - 0.5) * 0.05;
    Texel {
        colour: mix3(mortar_colour, stone_colour, stone),
        height: 0.22 + (face - 0.22) * stone,
        roughness: 0.94 - stone * 0.08 + (pits - 0.5) * 0.06,
        metal: 0.0,
    }
}

/// Upright planks, ten to the tile, with grain running up them.
fn timber(u: f32, v: f32) -> Texel {
    const PLANKS: i32 = 10;
    let x = u * PLANKS as f32;
    let plank = x.floor() as i32;
    let px = x - plank as f32;
    let id = hash2(plank, 0, 41);
    let gap = step(0.0, 0.035, px.min(1.0 - px));
    // Grain: noise stretched along the plank, shifted for each plank so none match.
    let streak = fbm(u + id * 0.37, v, 160, 3, 3, 43);
    let rings = ((px + (fbm(u, v + id, 6, 2, 2, 47) - 0.5) * 0.6) * 14.0 + id * 9.0).sin() * 0.5 + 0.5;
    let grain = streak * 0.65 + rings * 0.35;
    let (knot, _) = cellular(u + id, v, 3, 51 + plank as u32);
    let knot = 1.0 - step(0.02, 0.09, knot);
    let weather = fbm(u, v, 5, 5, 3, 53);
    let base = mix3([0.20, 0.13, 0.08], [0.36, 0.25, 0.15], grain);
    let colour = scale3(mix3(base, [0.10, 0.06, 0.04], knot * 0.8), 0.75 + id * 0.35 + (weather - 0.5) * 0.25);
    Texel {
        colour: scale3(colour, 0.35 + 0.65 * gap),
        height: (0.55 + grain * 0.25 - knot * 0.10) * (0.15 + 0.85 * gap),
        roughness: 0.78 + grain * 0.12,
        metal: 0.0,
    }
}

/// A plain weave: threads over and under, a hundred each way to the tile.
fn cloth(u: f32, v: f32) -> Texel {
    const THREADS: f32 = 100.0;
    let (x, y) = (u * THREADS, v * THREADS);
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let over = (xi + yi) % 2 == 0;
    let (fx, fy) = (x - xi as f32 - 0.5, y - yi as f32 - 0.5);
    // The thread on top runs across (warp) or up (weft), and is round in section.
    let across = if over { fy } else { fx };
    let along = if over { fx } else { fy };
    let round = (1.0 - (across * 2.0).powi(2)).max(0.0).sqrt();
    let dip = 0.85 + 0.15 * (along * std::f32::consts::PI).cos();
    let slub = fbm(u, v, 12, 60, 3, 61);
    let fade = fbm(u, v, 4, 4, 3, 67);
    let dye = mix3([0.36, 0.09, 0.07], [0.48, 0.15, 0.10], slub);
    Texel {
        colour: scale3(dye, (0.55 + 0.45 * round * dip) * (0.8 + fade * 0.4)),
        height: 0.3 + 0.6 * round * dip,
        roughness: 0.96,
        metal: 0.0,
    }
}

/// Wrought iron: dark, hammered, with rust where water sits.
fn iron(u: f32, v: f32) -> Texel {
    let (d, id) = cellular(u, v, 14, 71);
    let dimple = step(0.0, 0.7, d);
    let scale = fbm(u, v, 40, 40, 3, 73);
    let rust = step(0.56, 0.72, fbm(u, v, 7, 7, 4, 79));
    let metal_colour = scale3([0.20, 0.19, 0.18], 0.75 + id * 0.3 + scale * 0.25);
    let rust_colour = scale3([0.27, 0.12, 0.06], 0.7 + scale * 0.6);
    Texel {
        colour: mix3(metal_colour, rust_colour, rust),
        height: 0.5 + (dimple - 0.5) * 0.25 + (scale - 0.5) * 0.12 + rust * 0.06,
        roughness: 0.42 + scale * 0.14 + rust * 0.36,
        metal: 1.0 - rust * 0.85,
    }
}

/// Packed earth with small stones trodden into it.
fn earth(u: f32, v: f32) -> Texel {
    let broad = fbm(u, v, 5, 5, 4, 83);
    let fine = fbm(u, v, 48, 48, 3, 89);
    let (d, id) = cellular(u, v, 22, 97);
    // Only some points are stones, and they differ in size.
    let size = 0.10 + id * 0.22;
    let is_stone = hash((id * 4096.0) as u32) % 9 < 2;
    let stone = if is_stone { 1.0 - step(size * 0.6, size, d) } else { 0.0 };
    let damp = step(0.55, 0.8, fbm(u, v, 3, 3, 3, 101));
    let soil = scale3(mix3([0.19, 0.14, 0.09], [0.31, 0.25, 0.17], broad), 0.8 + fine * 0.4);
    let soil = scale3(soil, 1.0 - damp * 0.3);
    let pebble = scale3(mix3([0.22, 0.21, 0.19], [0.30, 0.28, 0.25], id), 0.85 + fine * 0.3);
    Texel {
        colour: mix3(soil, pebble, stone),
        height: 0.35 + broad * 0.2 + fine * 0.12 + stone * 0.3 * (1.0 - d / size.max(0.01) * 0.5),
        roughness: 0.95 - stone * 0.15 - damp * 0.25,
        metal: 0.0,
    }
}

/// Lantern glass: thick, slightly rippled, lit from within.
fn glass(u: f32, v: f32) -> Texel {
    let ripple = fbm(u, v, 6, 10, 3, 107);
    Texel {
        colour: scale3([1.0, 0.62, 0.26], 0.85 + ripple * 0.3),
        height: 0.5 + (ripple - 0.5) * 0.3,
        roughness: 0.25,
        metal: 0.0,
    }
}

/// Turf seen from a step back: green blades in clumps over soil, with dry tips.
fn grass(u: f32, v: f32) -> Texel {
    let clump = fbm(u, v, 6, 6, 4, 131);
    let blade = fbm(u, v, 96, 24, 3, 137);
    let dry = step(0.55, 0.85, fbm(u, v, 4, 4, 3, 139));
    let green = mix3([0.10, 0.17, 0.05], [0.20, 0.30, 0.09], clump);
    let colour = mix3(green, [0.32, 0.30, 0.14], dry * 0.6);
    let gap = step(0.25, 0.45, blade);
    Texel {
        colour: mix3(scale3([0.16, 0.12, 0.08], 0.9), scale3(colour, 0.8 + blade * 0.45), gap),
        height: 0.3 + clump * 0.3 + blade * 0.3,
        roughness: 0.9,
        metal: 0.0,
    }
}

/// Barley stubble and straw on dry earth, in rows.
fn straw(u: f32, v: f32) -> Texel {
    let row = ((v * 40.0).fract() - 0.5).abs() * 2.0;
    let stalk = fbm(u, v, 128, 12, 3, 149);
    let broad = fbm(u, v, 5, 5, 3, 151);
    let gold = mix3([0.46, 0.38, 0.18], [0.62, 0.52, 0.27], broad);
    let ground = [0.24, 0.19, 0.12];
    let on = 1.0 - step(0.45, 0.8, row);
    Texel {
        colour: mix3(ground, scale3(gold, 0.8 + stalk * 0.4), on * (0.6 + stalk * 0.4)),
        height: 0.3 + on * 0.4 + stalk * 0.2,
        roughness: 0.85,
        metal: 0.0,
    }
}

/// A beaten road: packed earth and small stones, ruts along it.
fn gravel(u: f32, v: f32) -> Texel {
    let (d, id) = cellular(u, v, 64, 157);
    let fine = fbm(u, v, 40, 40, 3, 163);
    let broad = fbm(u, v, 4, 4, 3, 167);
    let size = 0.25 + id * 0.25;
    let stone = 1.0 - step(size * 0.5, size, d);
    let packed = mix3([0.30, 0.26, 0.20], [0.42, 0.37, 0.29], broad);
    let pebble = mix3([0.36, 0.34, 0.30], [0.52, 0.49, 0.44], id);
    Texel {
        colour: scale3(mix3(packed, pebble, stone * 0.8), 0.85 + fine * 0.3),
        height: 0.3 + stone * 0.45 + fine * 0.1,
        roughness: 0.9 - stone * 0.15,
        metal: 0.0,
    }
}

/// Wet mud at the water's edge, darker and shinier where it is wettest.
fn mud(u: f32, v: f32) -> Texel {
    let broad = fbm(u, v, 5, 5, 4, 173);
    let fine = fbm(u, v, 48, 48, 3, 179);
    let wet = step(0.45, 0.75, fbm(u, v, 3, 3, 3, 181));
    let colour = mix3([0.20, 0.16, 0.11], [0.11, 0.09, 0.07], wet);
    Texel {
        colour: scale3(colour, 0.85 + broad * 0.25 + fine * 0.1),
        height: 0.4 + broad * 0.2 - wet * 0.15 + fine * 0.08,
        roughness: 0.9 - wet * 0.55,
        metal: 0.0,
    }
}

/// Water: what is seen of its body, for its letters and its tint. Its surface, its light,
/// and its depth are drawn by the water pass, not from these maps.
fn water(u: f32, v: f32) -> Texel {
    let broad = fbm(u, v, 4, 4, 4, 211);
    let fine = fbm(u, v, 32, 32, 3, 223);
    Texel {
        colour: scale3([0.05, 0.09, 0.08], 0.9 + broad * 0.2),
        height: 0.5 + (broad - 0.5) * 0.3 + (fine - 0.5) * 0.2,
        roughness: 0.04,
        metal: 0.0,
    }
}

/// The floor of a wood: moss and fallen leaves.
fn moss(u: f32, v: f32) -> Texel {
    let broad = fbm(u, v, 6, 6, 4, 191);
    let (d, id) = cellular(u, v, 40, 193);
    let leaf = (1.0 - step(0.15, 0.35, d)) * if hash((id * 4096.0) as u32) % 3 == 0 { 1.0 } else { 0.0 };
    let green = mix3([0.07, 0.11, 0.04], [0.14, 0.20, 0.06], broad);
    let brown = mix3([0.28, 0.16, 0.07], [0.40, 0.26, 0.10], id);
    Texel {
        colour: mix3(green, brown, leaf),
        height: 0.35 + broad * 0.3 + leaf * 0.15,
        roughness: 0.95,
        metal: 0.0,
    }
}

/// Bare rock where the ground is too steep to hold soil: layered, cracked, lichened.
fn rock(u: f32, v: f32) -> Texel {
    let broad = fbm(u, v, 4, 4, 5, 197);
    let layer = ((v * 14.0 + broad * 3.0).fract() - 0.5).abs() * 2.0;
    let (d, _) = cellular(u, v, 10, 199);
    let crack = 1.0 - step(0.0, 0.06, d);
    let lichen = step(0.62, 0.8, fbm(u, v, 12, 12, 3, 211));
    let stone = mix3([0.30, 0.29, 0.27], [0.46, 0.44, 0.41], broad);
    let colour = mix3(scale3(stone, 0.9 + layer * 0.15), [0.38, 0.40, 0.24], lichen * 0.5);
    Texel {
        colour: scale3(colour, 1.0 - crack * 0.5),
        height: 0.5 + broad * 0.3 - crack * 0.3 + layer * 0.06,
        roughness: 0.85,
        metal: 0.0,
    }
}

/// The Erasure: bare paper, very faintly fibrous.
fn paper(u: f32, v: f32) -> Texel {
    let fibre = fbm(u, v, 64, 64, 3, 223);
    Texel { colour: scale3([0.86, 0.85, 0.81], 0.97 + fibre * 0.05), height: 0.5 + fibre * 0.05, roughness: 0.95, metal: 0.0 }
}

fn to_srgb(v: f32) -> u8 {
    let v = v.clamp(0.0, 1.0);
    let s = if v <= 0.003_130_8 { v * 12.92 } else { 1.055 * v.powf(1.0 / 2.4) - 0.055 };
    (s * 255.0 + 0.5) as u8
}

fn from_srgb(b: u8) -> f32 {
    let s = b as f32 / 255.0;
    if s <= 0.040_45 { s / 12.92 } else { ((s + 0.055) / 1.055).powf(2.4) }
}

/// The two maps of one material at the largest level.
struct Maps {
    colour: Vec<u8>,
    normal: Vec<u8>,
}

/// How far the surface's highest point stands above its lowest, in metres.
fn relief_depth(generator: Generator) -> f32 {
    match generator {
        Generator::Masonry => 0.012,
        Generator::Timber => 0.004,
        Generator::Cloth => 0.0008,
        Generator::Iron => 0.002,
        Generator::Earth => 0.015,
        Generator::Glass => 0.001,
        Generator::Grass => 0.02,
        Generator::Straw => 0.015,
        Generator::Gravel => 0.01,
        Generator::Mud => 0.008,
        Generator::Moss => 0.015,
        Generator::Rock => 0.03,
        Generator::Paper => 0.0005,
        Generator::Water => 0.002,
    }
}

fn generate(generator: Generator, size: u32, tile_metres: f32) -> Maps {
    use rayon::prelude::*;
    let n = size as usize;
    let texels: Vec<Texel> = (0..n * n)
        .into_par_iter()
        .map(|i| {
            let (u, v) = (((i % n) as f32 + 0.5) / n as f32, ((i / n) as f32 + 0.5) / n as f32);
            match generator {
                Generator::Masonry => masonry(u, v),
                Generator::Timber => timber(u, v),
                Generator::Cloth => cloth(u, v),
                Generator::Iron => iron(u, v),
                Generator::Earth => earth(u, v),
                Generator::Glass => glass(u, v),
                Generator::Grass => grass(u, v),
                Generator::Straw => straw(u, v),
                Generator::Gravel => gravel(u, v),
                Generator::Mud => mud(u, v),
                Generator::Moss => moss(u, v),
                Generator::Rock => rock(u, v),
                Generator::Paper => paper(u, v),
                Generator::Water => water(u, v),
            }
        })
        .collect();

    let texel_metres = tile_metres / size as f32;
    let depth = relief_depth(generator);
    let mut colour = Vec::with_capacity(n * n * 4);
    let mut normal = Vec::with_capacity(n * n * 4);
    for y in 0..n {
        for x in 0..n {
            let t = texels[y * n + x];
            colour.extend_from_slice(&[to_srgb(t.colour[0]), to_srgb(t.colour[1]), to_srgb(t.colour[2]), (t.metal.clamp(0.0, 1.0) * 255.0 + 0.5) as u8]);
            let h = |dx: isize, dy: isize| {
                let xx = (x as isize + dx).rem_euclid(n as isize) as usize;
                let yy = (y as isize + dy).rem_euclid(n as isize) as usize;
                texels[yy * n + xx].height
            };
            // Slope of the height in metres per metre. The map's v runs down the image,
            // and the anchor's second coordinate runs up, so the second slope is flipped.
            let sx = (h(1, 0) - h(-1, 0)) * depth / (2.0 * texel_metres);
            let sy = (h(0, -1) - h(0, 1)) * depth / (2.0 * texel_metres);
            let len = (sx * sx + sy * sy + 1.0).sqrt();
            let (nx, ny) = (-sx / len, -sy / len);
            normal.extend_from_slice(&[
                ((nx * 0.5 + 0.5) * 255.0 + 0.5) as u8,
                ((ny * 0.5 + 0.5) * 255.0 + 0.5) as u8,
                (t.roughness.clamp(0.04, 1.0) * 255.0 + 0.5) as u8,
                (t.height.clamp(0.0, 1.0) * 255.0 + 0.5) as u8,
            ]);
        }
    }
    Maps { colour, normal }
}

fn halve(texels: &[u8], side: u32, is_colour: bool) -> Vec<u8> {
    let half = (side / 2) as usize;
    let side = side as usize;
    let mut out = Vec::with_capacity(half * half * 4);
    for y in 0..half {
        for x in 0..half {
            let at = |dx: usize, dy: usize, c: usize| texels[((y * 2 + dy) * side + x * 2 + dx) * 4 + c];
            for c in 0..4 {
                let four = [at(0, 0, c), at(1, 0, c), at(0, 1, c), at(1, 1, c)];
                if is_colour && c < 3 {
                    // Colours are averaged as light, not as stored bytes.
                    out.push(to_srgb(four.iter().map(|&b| from_srgb(b)).sum::<f32>() / 4.0));
                } else {
                    out.push(((four.iter().map(|&b| b as u32).sum::<u32>() + 2) / 4) as u8);
                }
            }
        }
    }
    out
}

pub const MATERIAL_SET_KEY: &str = "materials.set";

/// The key of a material named in authored data.
pub fn material_id(name: &str) -> AssetId {
    AssetId::from_key(&format!("material.{name}"))
}

pub fn bake_set(file: &Path) -> Result<MaterialSet, BakeError> {
    let text = std::fs::read_to_string(file).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
    let authored: AuthoredSet = ron::from_str(&text).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
    if !authored.size.is_power_of_two() || authored.size < 16 {
        return err(format!("{}: size must be a power of two", file.display()));
    }
    let mips = authored.size.trailing_zeros() + 1;
    let lexicons = read_lexicons(&file.with_file_name("lexicon.ron"))?;
    let mut materials = Vec::new();
    let mut levels: Vec<(Vec<u8>, Vec<u8>)> = Vec::new();
    for m in &authored.materials {
        let family = m.family.as_bytes();
        if family.is_empty() || family.len() > MaterialSet::MAX_FAMILY || !family.iter().all(|b| (0x20..=0x7e).contains(b)) {
            return err(format!("{}: `{}` needs 1 to 8 printable ASCII characters in its family", file.display(), m.key));
        }
        let word = |w: &String| -> Result<Vec<u8>, BakeError> {
            if w.is_empty() || w.len() > Lexicon::WORD || !w.bytes().all(|b| (0x21..=0x7e).contains(&b)) {
                return err(format!("{}: `{}`: the word `{w}` must be 1 to 16 printable ASCII characters, no spaces", file.display(), m.key));
            }
            Ok(w.as_bytes().to_vec())
        };
        let list = |words: &Vec<String>, most: usize, what: &str| -> Result<Vec<Vec<u8>>, BakeError> {
            if words.len() > most {
                return err(format!("{}: `{}` has more than {most} {what} words", file.display(), m.key));
            }
            words.iter().map(word).collect()
        };
        let Some(words) = lexicons.iter().find(|l| l.key == m.key) else {
            return err(format!("{}: `{}` has no entry in lexicon.ron", file.display(), m.key));
        };
        let lexicon = Lexicon {
            name: if words.name.is_empty() { Vec::new() } else { word(&words.name)? },
            elements: list(&words.elements, Lexicon::ELEMENTS, "element")?,
            hot: list(&words.hot, Lexicon::HOT, "hot")?,
            cold: list(&words.cold, Lexicon::COLD, "cold")?,
        };
        materials.push(Material {
            lexicon,
            key: material_id(&m.key),
            tile_metres: m.tile_metres,
            contrast: m.contrast,
            relief: m.relief,
            emissive: m.emissive,
            family: family.to_vec(),
        });
        let maps = generate(m.generator, authored.size, m.tile_metres);
        levels.push((maps.colour, maps.normal));
    }
    let mut texels = Vec::new();
    for level in 0..mips {
        for (colour, _) in &levels {
            texels.extend_from_slice(colour);
        }
        for (_, normal) in &levels {
            texels.extend_from_slice(normal);
        }
        let side = authored.size >> level;
        if side > 1 {
            for (colour, normal) in &mut levels {
                *colour = halve(colour, side, true);
                *normal = halve(normal, side, false);
            }
        }
    }
    Ok(MaterialSet { size: authored.size, mips, materials, texels })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_is_periodic() {
        for (x, y) in [(0.3, 0.7), (3.9, 2.2)] {
            assert_eq!(noise_xy(x, y, 4, 4, 1), noise_xy(x + 4.0, y + 4.0, 4, 4, 1));
        }
        assert!((fbm(0.001, 0.5, 8, 8, 3, 2) - fbm(1.001, 0.5, 8, 8, 3, 2)).abs() < 1e-4);
        let (a, _) = cellular(0.02, 0.5, 6, 3);
        let (b, _) = cellular(1.02, 0.5, 6, 3);
        assert!((a - b).abs() < 1e-4);
    }

    #[test]
    fn the_brief_table_is_the_lexicon_file_written_out() {
        // The table in the art direction brief and `lexicon.ron` must say the same.
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
        let lexicons = read_lexicons(&root.join("assets/materials/lexicon.ron")).unwrap();
        let brief = std::fs::read_to_string(root.join("full-game/docs/brief/2-art-direction.md")).unwrap();
        let table = brief.split("<!-- lexicon:start -->").nth(1).and_then(|t| t.split("<!-- lexicon:end -->").next()).expect("the brief has the table");
        let words = |cell: &str| -> Vec<String> {
            if cell == "none" {
                return Vec::new();
            }
            cell.split(", ").map(|w| w.trim_matches('`').to_owned()).collect()
        };
        let rows: Vec<Vec<String>> = table
            .lines()
            .filter(|l| l.starts_with("| ") && !l.starts_with("| Material"))
            .map(|l| l.trim().trim_matches('|').split(" | ").map(|c| c.trim().to_owned()).collect())
            .collect();
        assert_eq!(rows.len(), lexicons.len(), "the brief and lexicon.ron list different materials");
        for (row, l) in rows.iter().zip(&lexicons) {
            assert_eq!(row[0], l.key);
            assert_eq!(words(&row[1]), if l.name.is_empty() { Vec::new() } else { vec![l.name.clone()] }, "{}", l.key);
            assert_eq!(words(&row[2]), l.elements, "{}", l.key);
            assert_eq!(words(&row[3]), l.hot, "{}", l.key);
            assert_eq!(words(&row[4]), l.cold, "{}", l.key);
        }
    }

    #[test]
    fn generated_maps_tile_and_point_outwards() {
        let maps = generate(Generator::Masonry, 64, 2.0);
        assert_eq!(maps.colour.len(), 64 * 64 * 4);
        // A normal's third part is implied; the first two must leave room for it.
        for t in maps.normal.chunks_exact(4) {
            let (x, y) = (t[0] as f32 / 127.5 - 1.0, t[1] as f32 / 127.5 - 1.0);
            assert!(x * x + y * y < 1.0);
        }
        // Left and right edges agree closely: the map tiles.
        let edge_difference: u32 = (0..64)
            .map(|y| (maps.colour[(y * 64) * 4] as i32 - maps.colour[(y * 64 + 63) * 4] as i32).unsigned_abs())
            .sum();
        assert!(edge_difference / 64 < 40, "{edge_difference}");
    }

    #[test]
    fn srgb_round_trip() {
        for b in [0u8, 1, 30, 128, 200, 255] {
            assert_eq!(to_srgb(from_srgb(b)), b);
        }
    }
}
