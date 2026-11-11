//! The terrain of the Showcase, baked into tiles.
//!
//! The land is made in three nested grids of 513 × 513 samples, each a coarser level of
//! the tile layout: the 512 m playable area at a metre a sample (level 0), the far ring
//! out to 2 km at 8 m (level 3), and the horizon out to 8 km at 32 m (level 5). Each grid
//! is made the same way:
//!
//! 1. **Shape.** One function of position gives the land everywhere: the valley floor
//!    rising away from the river, noise, the hills, the rolling country and the
//!    highlands beyond, and the steep ground at the area's edges. A finer grid takes
//!    more octaves of the same noise, so the grids agree on everything both can show.
//! 2. **Erosion.** Water and slumping wear the land: hydraulic erosion by droplets,
//!    then thermal erosion, both deterministic.
//! 3. **Landmarks,** shaped by hand in `assets/terrain/valley.ron`: level ground for the
//!    hilltop, the green, and the yards; roads that follow the land without its bumps;
//!    rivers and ponds cut into it, with their water levels found from the ground.
//! 4. **Seams.** Near its border a grid eases into the coarser one, so that where they
//!    meet they are exactly the same ground. Then the finer grid is written into the
//!    coarser one wherever both have a sample, so a far view of near ground is true.
//!
//! Finally every sample is given its water and its ground, and each grid is cut into
//! 8 × 8 tiles of 65 × 65 samples.

use std::path::Path;

use serde::Deserialize;
use vellmark_content::{AssetId, Ground, TerrainTile, TileCoord, TERRAIN_LAYER, TILE_SAMPLES};

use crate::sha256::{hex, sha256};
use crate::{err, BakeError};

/// Changes whenever the generator does, so that the cache is not trusted across it.
const VERSION: &str = "terrain 1";
/// Samples along each side of a grid: eight tiles of 64 cells, and the last edge.
const GRID: usize = 513;
/// The levels baked, finest last, with the half-width of each grid in metres.
const LEVELS: [(u8, f32); 3] = [(5, 8192.0), (3, 2048.0), (0, 256.0)];

// ---- the authored data ----

#[derive(Debug, Deserialize)]
struct Authored {
    seed: u32,
    shape: Shape,
    edges: Edges,
    hills: Vec<Hill>,
    flats: Vec<Flat>,
    roads: Vec<Road>,
    rivers: Vec<River>,
    ponds: Vec<Pond>,
    fields: Vec<Area>,
    woods: Vec<Area>,
}

#[derive(Debug, Deserialize)]
struct Shape {
    floor: f32,
    from_river: f32,
    from_river_most: f32,
    relief: f32,
    wavelength: f32,
    rolling: Rolling,
    highlands: Highlands,
    forest: Forest,
    erasure: Erasure,
}

#[derive(Debug, Deserialize)]
struct Rolling {
    height: f32,
    wavelength: f32,
    beyond: f32,
}

#[derive(Debug, Deserialize)]
struct Highlands {
    from: f32,
    to: f32,
    height: f32,
    wavelength: f32,
}

#[derive(Debug, Deserialize)]
struct Forest {
    beyond: f32,
    cover: f32,
    wavelength: f32,
}

#[derive(Debug, Deserialize)]
struct Erasure {
    from: f32,
    ragged: f32,
}

#[derive(Debug, Deserialize)]
struct Edges {
    start: f32,
    width: f32,
    north: f32,
    south: f32,
    east: f32,
    west: f32,
}

#[derive(Debug, Deserialize)]
struct Hill {
    key: String,
    centre: (f32, f32),
    radius: f32,
    height: f32,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq)]
enum GroundName {
    Grass,
    Field,
    Road,
    Earth,
    Bank,
    Forest,
}

impl GroundName {
    fn ground(self) -> Ground {
        match self {
            Self::Grass => Ground::Grass,
            Self::Field => Ground::Field,
            Self::Road => Ground::Road,
            Self::Earth => Ground::Earth,
            Self::Bank => Ground::Bank,
            Self::Forest => Ground::Forest,
        }
    }
}

#[derive(Debug, Deserialize)]
struct Flat {
    key: String,
    centre: (f32, f32),
    radius: f32,
    blend: f32,
    /// The height to level to. If not given, the ground's own height at the centre.
    #[serde(default)]
    height: Option<f32>,
    #[serde(default)]
    ground: Option<GroundName>,
}

#[derive(Debug, Deserialize)]
struct Road {
    key: String,
    width: f32,
    /// The steepest the road may climb, rise over run. Where the land is steeper the
    /// road is cut into it or banked up, as roads are.
    #[serde(default)]
    grade: Option<f32>,
    points: Vec<(f32, f32)>,
}

#[derive(Debug, Deserialize)]
struct River {
    key: String,
    width: f32,
    depth: f32,
    bank: f32,
    points: Vec<(f32, f32)>,
}

#[derive(Debug, Deserialize)]
struct Pond {
    key: String,
    centre: (f32, f32),
    radius: f32,
    depth: f32,
    bank: f32,
}

#[derive(Debug, Deserialize)]
struct Area {
    key: String,
    points: Vec<(f32, f32)>,
}

// ---- noise ----

fn hash(mut x: u32) -> u32 {
    x ^= x >> 16;
    x = x.wrapping_mul(0x7feb_352d);
    x ^= x >> 15;
    x = x.wrapping_mul(0x846c_a68b);
    x ^ (x >> 16)
}

/// A unit gradient for a lattice point.
fn gradient(x: i32, y: i32, seed: u32) -> (f32, f32) {
    let h = hash((x as u32).wrapping_mul(0x9e37_79b1) ^ hash((y as u32).wrapping_mul(0x85eb_ca6b) ^ seed));
    let a = (h >> 8) as f32 / 16_777_216.0 * std::f32::consts::TAU;
    (a.cos(), a.sin())
}

/// Gradient noise, about -0.7 to 0.7.
fn noise(x: f32, y: f32, seed: u32) -> f32 {
    let (xi, yi) = (x.floor() as i32, y.floor() as i32);
    let (fx, fy) = (x - xi as f32, y - yi as f32);
    let fade = |t: f32| t * t * t * (t * (t * 6.0 - 15.0) + 10.0);
    let dot = |dx: i32, dy: i32| {
        let g = gradient(xi + dx, yi + dy, seed);
        g.0 * (fx - dx as f32) + g.1 * (fy - dy as f32)
    };
    let (u, v) = (fade(fx), fade(fy));
    let a = dot(0, 0) + (dot(1, 0) - dot(0, 0)) * u;
    let b = dot(0, 1) + (dot(1, 1) - dot(0, 1)) * u;
    a + (b - a) * v
}

/// Octaves of noise from `wavelength` down to the finest a grid of `spacing` can carry.
/// Every grid sums the same octaves as far as it goes, so the grids agree.
fn fbm(x: f32, z: f32, wavelength: f32, spacing: f32, seed: u32) -> f32 {
    let (mut sum, mut amplitude, mut w, mut o) = (0.0, 1.0, wavelength, 0u32);
    while w >= spacing * 4.0 && o < 12 {
        sum += noise(x / w, z / w, seed.wrapping_add(o * 1013)) * amplitude;
        amplitude *= 0.5;
        w *= 0.5;
        o += 1;
    }
    sum
}

fn smoothstep(edge0: f32, edge1: f32, x: f32) -> f32 {
    let t = ((x - edge0) / (edge1 - edge0)).clamp(0.0, 1.0);
    t * t * (3.0 - 2.0 * t)
}

/// A small deterministic random number source.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }

    fn unit(&mut self) -> f32 {
        (self.next() >> 40) as f32 / 16_777_216.0
    }
}

// ---- lines and shapes ----

/// The nearest point of a polyline: distance, and the distance along the line to it.
fn nearest_on(points: &[(f32, f32)], x: f32, z: f32) -> (f32, f32) {
    let (mut best, mut along_best, mut along) = (f32::INFINITY, 0.0, 0.0);
    for w in points.windows(2) {
        let (a, b) = (w[0], w[1]);
        let (dx, dz) = (b.0 - a.0, b.1 - a.1);
        let length = (dx * dx + dz * dz).sqrt();
        let t = (((x - a.0) * dx + (z - a.1) * dz) / (length * length)).clamp(0.0, 1.0);
        let (px, pz) = (a.0 + dx * t, a.1 + dz * t);
        let d = ((x - px).powi(2) + (z - pz).powi(2)).sqrt();
        if d < best {
            best = d;
            along_best = along + t * length;
        }
        along += length;
    }
    (best, along_best)
}

fn polyline_length(points: &[(f32, f32)]) -> f32 {
    points.windows(2).map(|w| ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt()).sum()
}

/// The point at a distance along a polyline.
fn point_along(points: &[(f32, f32)], s: f32) -> (f32, f32) {
    let mut left = s;
    for w in points.windows(2) {
        let length = ((w[1].0 - w[0].0).powi(2) + (w[1].1 - w[0].1).powi(2)).sqrt();
        if left <= length {
            let t = left / length;
            return (w[0].0 + (w[1].0 - w[0].0) * t, w[0].1 + (w[1].1 - w[0].1) * t);
        }
        left -= length;
    }
    *points.last().unwrap()
}

fn inside(points: &[(f32, f32)], x: f32, z: f32) -> bool {
    let mut odd = false;
    let mut j = points.len() - 1;
    for i in 0..points.len() {
        let (a, b) = (points[i], points[j]);
        if (a.1 > z) != (b.1 > z) && x < (b.0 - a.0) * (z - a.1) / (b.1 - a.1) + a.0 {
            odd = !odd;
        }
        j = i;
    }
    odd
}

/// A profile along a line, sampled every metre: a river's water level or a road's
/// height.
struct Profile {
    step: f32,
    values: Vec<f32>,
}

impl Profile {
    fn at(&self, s: f32) -> f32 {
        let x = (s / self.step).clamp(0.0, (self.values.len() - 1) as f32);
        let i = (x.floor() as usize).min(self.values.len().saturating_sub(2));
        let t = x - i as f32;
        match self.values.get(i + 1) {
            Some(next) => self.values[i] + (next - self.values[i]) * t,
            None => self.values[i],
        }
    }
}

// ---- the land ----

struct Land<'a> {
    a: &'a Authored,
    /// Each river's water level along it, found from the shaped land before erosion.
    river_levels: Vec<Profile>,
    /// Each pond's water level.
    pond_levels: Vec<f32>,
}

impl<'a> Land<'a> {
    fn new(a: &'a Authored) -> Self {
        let mut land = Self { a, river_levels: Vec::new(), pond_levels: Vec::new() };
        // Water finds its level from the land at the finest grid's detail: a running
        // minimum downstream, so that a river never flows uphill, a little below the
        // ground so that it sits in its bed.
        for river in &a.rivers {
            let length = polyline_length(&river.points);
            let step = 1.0f32.max(length / 20_000.0);
            let mut values = Vec::new();
            let mut lowest = f32::INFINITY;
            let mut s = 0.0;
            while s <= length {
                let (x, z) = point_along(&river.points, s);
                lowest = lowest.min(land.shape(x, z, 1.0) - river.depth * 0.35 - 0.25);
                values.push(lowest);
                s += step;
            }
            land.river_levels.push(Profile { step, values });
        }
        // A pond's level is a little below the lowest point of its rim, so it holds.
        for pond in &a.ponds {
            let rim = (0..48)
                .map(|k| {
                    let t = k as f32 / 48.0 * std::f32::consts::TAU;
                    let r = pond.radius + pond.bank;
                    land.shape(pond.centre.0 + r * t.cos(), pond.centre.1 + r * t.sin(), 1.0)
                })
                .fold(f32::INFINITY, f32::min);
            land.pond_levels.push(rim - 0.45);
        }
        land
    }

    fn river_distance(&self, x: f32, z: f32) -> f32 {
        self.a.rivers.first().map_or(f32::INFINITY, |r| nearest_on(&r.points, x, z).0)
    }

    /// Step 1: the land as one function of position, to the detail a grid of
    /// `spacing` can carry.
    fn shape(&self, x: f32, z: f32, spacing: f32) -> f32 {
        let s = &self.a.shape;
        let seed = self.a.seed;
        let mut h = s.floor + (self.river_distance(x, z) * s.from_river).min(s.from_river_most);
        h += fbm(x, z, s.wavelength, spacing, seed) * s.relief * 2.0;

        // Rolling country beyond the area.
        let away = x.abs().max(z.abs());
        let rolling = smoothstep(s.rolling.beyond, s.rolling.beyond * 2.2, away);
        h += rolling * s.rolling.height * (0.55 + fbm(x, z, s.rolling.wavelength, spacing, seed ^ 0x51) * 1.3).max(0.0);

        // The highlands to the north, ridged.
        let north = smoothstep(s.highlands.from, s.highlands.to, -z);
        if north > 0.0 {
            let ridge = 1.0 - (fbm(x, z, s.highlands.wavelength, spacing, seed ^ 0x77) * 1.6).abs();
            h += north * s.highlands.height * (0.55 + 0.45 * ridge);
        }
        // Where the Erasure has scraped the land back to paper, it is smoothed out.
        let erased = self.erased(x, z);
        if erased > 0.0 {
            let smooth = s.floor + north * s.highlands.height * 0.7;
            h += (smooth - h) * erased * 0.6;
        }

        // The hills.
        for hill in &self.a.hills {
            let d = ((x - hill.centre.0).powi(2) + (z - hill.centre.1).powi(2)).sqrt();
            // A ragged outline, so no hill is a perfect dome.
            let d = d * (1.0 + 0.18 * noise(x / 60.0, z / 60.0, seed ^ hash(hill.key.len() as u32)));
            if d < hill.radius {
                h += hill.height * 0.5 * (1.0 + (std::f32::consts::PI * d / hill.radius).cos());
            }
        }

        // Steep ground at the edges of the area, and the land beyond it stays raised.
        let e = &self.a.edges;
        let rise = |inward: f32| smoothstep(e.start, e.start + e.width, inward);
        let edge = (rise(-z) * e.north).max(rise(z) * e.south).max(rise(x) * e.east).max(rise(-x) * e.west);
        h + edge * (0.85 + 0.3 * noise(x / 23.0, z / 23.0, seed ^ 0xed6e))
    }

    /// 0 to 1: how far the Erasure has taken this place.
    fn erased(&self, x: f32, z: f32) -> f32 {
        let e = &self.a.shape.erasure;
        let ragged = noise(x / 900.0, 0.5, self.a.seed ^ 0xe5) * e.ragged * 2.0;
        smoothstep(e.from + ragged, e.from + ragged + 250.0, -z)
    }
}

/// One grid of heights.
#[derive(Clone)]
struct Grid {
    level: u8,
    half: f32,
    spacing: f32,
    h: Vec<f32>,
}

impl Grid {
    fn position(&self, i: usize, j: usize) -> (f32, f32) {
        (-self.half + i as f32 * self.spacing, -self.half + j as f32 * self.spacing)
    }

    fn contains(&self, x: f32, z: f32) -> bool {
        x.abs() <= self.half && z.abs() <= self.half
    }

    /// Height between samples, as the grid's triangles are drawn.
    fn height_at(&self, x: f32, z: f32) -> f32 {
        let last = (GRID - 1) as f32;
        let u = ((x + self.half) / self.spacing).clamp(0.0, last);
        let v = ((z + self.half) / self.spacing).clamp(0.0, last);
        let (i, j) = ((u.floor() as usize).min(GRID - 2), (v.floor() as usize).min(GRID - 2));
        let (fu, fv) = (u - i as f32, v - j as f32);
        let h = |a: usize, b: usize| self.h[b * GRID + a];
        if fu >= fv {
            h(i, j) + (h(i + 1, j) - h(i, j)) * fu + (h(i + 1, j + 1) - h(i + 1, j)) * fv
        } else {
            h(i, j) + (h(i + 1, j + 1) - h(i, j + 1)) * fu + (h(i, j + 1) - h(i, j)) * fv
        }
    }
}

// ---- step 2: erosion ----

/// Hydraulic erosion by droplets. Heights are worked in cells, so that the same
/// settings wear a metre grid and an eight-metre grid alike.
fn erode_hydraulic(grid: &mut Grid, droplets: usize, seed: u64) {
    const INERTIA: f32 = 0.05;
    const CAPACITY: f32 = 4.0;
    const MIN_SLOPE: f32 = 0.01;
    const ERODE: f32 = 0.3;
    const DEPOSIT: f32 = 0.3;
    const EVAPORATE: f32 = 0.015;
    const GRAVITY: f32 = 4.0;
    const LIFETIME: usize = 48;
    const RADIUS: i32 = 3;

    let n = GRID;
    let scale = grid.spacing;
    let mut h: Vec<f32> = grid.h.iter().map(|v| v / scale).collect();
    // The brush that spreads erosion over the cells around a droplet.
    let mut brush: Vec<(i32, i32, f32)> = Vec::new();
    for dy in -RADIUS..=RADIUS {
        for dx in -RADIUS..=RADIUS {
            let d = ((dx * dx + dy * dy) as f32).sqrt();
            if d <= RADIUS as f32 {
                brush.push((dx, dy, 1.0 - d / RADIUS as f32));
            }
        }
    }
    let total: f32 = brush.iter().map(|b| b.2).sum();
    for b in &mut brush {
        b.2 /= total;
    }

    let sample = |h: &[f32], x: f32, y: f32| -> (f32, f32, f32) {
        let (i, j) = (x.floor() as usize, y.floor() as usize);
        let (u, v) = (x - i as f32, y - j as f32);
        let k = j * n + i;
        let (a, b, c, d) = (h[k], h[k + 1], h[k + n], h[k + n + 1]);
        let gx = (b - a) * (1.0 - v) + (d - c) * v;
        let gy = (c - a) * (1.0 - u) + (d - b) * u;
        (a * (1.0 - u) * (1.0 - v) + b * u * (1.0 - v) + c * (1.0 - u) * v + d * u * v, gx, gy)
    };

    let mut rng = Rng(seed);
    let limit = (n - 2) as f32;
    for _ in 0..droplets {
        let (mut x, mut y) = (1.0 + rng.unit() * (limit - 1.0), 1.0 + rng.unit() * (limit - 1.0));
        let (mut dx, mut dy, mut speed, mut water, mut sediment) = (0.0f32, 0.0f32, 1.0f32, 1.0f32, 0.0f32);
        for _ in 0..LIFETIME {
            let (i, j) = (x.floor() as usize, y.floor() as usize);
            let (u, v) = (x - i as f32, y - j as f32);
            let (height, gx, gy) = sample(&h, x, y);
            dx = dx * INERTIA - gx * (1.0 - INERTIA);
            dy = dy * INERTIA - gy * (1.0 - INERTIA);
            let len = (dx * dx + dy * dy).sqrt();
            if len < 1e-6 {
                break;
            }
            dx /= len;
            dy /= len;
            x += dx;
            y += dy;
            if x < 1.0 || y < 1.0 || x >= limit || y >= limit {
                break;
            }
            let (new_height, _, _) = sample(&h, x, y);
            let fall = new_height - height;
            let capacity = (-fall).max(MIN_SLOPE) * speed * water * CAPACITY;
            if sediment > capacity || fall > 0.0 {
                // Drop sediment: into the pit if it ran uphill, else what it cannot carry.
                let amount = if fall > 0.0 { fall.min(sediment) } else { (sediment - capacity) * DEPOSIT };
                sediment -= amount;
                let k = j * n + i;
                h[k] += amount * (1.0 - u) * (1.0 - v);
                h[k + 1] += amount * u * (1.0 - v);
                h[k + n] += amount * (1.0 - u) * v;
                h[k + n + 1] += amount * u * v;
            } else {
                let amount = ((capacity - sediment) * ERODE).min(-fall);
                for &(bx, by, w) in &brush {
                    let (ci, cj) = (i as i32 + bx, j as i32 + by);
                    if ci < 0 || cj < 0 || ci >= n as i32 || cj >= n as i32 {
                        continue;
                    }
                    let k = cj as usize * n + ci as usize;
                    let take = (amount * w).min(h[k].max(0.0) + 1000.0);
                    h[k] -= take;
                    sediment += take;
                }
            }
            speed = (speed * speed + fall.abs() * GRAVITY).sqrt();
            water *= 1.0 - EVAPORATE;
        }
    }
    for (out, v) in grid.h.iter_mut().zip(h) {
        *out = v * scale;
    }
}

/// Thermal erosion: wherever the ground stands steeper than loose earth can, some of
/// it slumps to the lower neighbours.
fn erode_thermal(grid: &mut Grid, iterations: usize, talus: f32) {
    let n = GRID;
    let limit = talus * grid.spacing;
    let mut moved = vec![0.0f32; n * n];
    for _ in 0..iterations {
        moved.iter_mut().for_each(|m| *m = 0.0);
        for j in 1..n - 1 {
            for i in 1..n - 1 {
                let k = j * n + i;
                let here = grid.h[k];
                for nk in [k - 1, k + 1, k - n, k + n] {
                    let excess = here - grid.h[nk] - limit;
                    if excess > 0.0 {
                        let amount = excess * 0.12;
                        moved[k] -= amount;
                        moved[nk] += amount;
                    }
                }
            }
        }
        for (h, m) in grid.h.iter_mut().zip(&moved) {
            *h += m;
        }
    }
}

// ---- step 3: landmarks ----

impl Land<'_> {
    fn shape_landmarks(&self, grid: &mut Grid) {
        let a = self.a;
        // Level ground.
        for flat in &a.flats {
            let target = flat.height.unwrap_or_else(|| grid.height_at(flat.centre.0, flat.centre.1));
            for j in 0..GRID {
                for i in 0..GRID {
                    let (x, z) = grid.position(i, j);
                    let d = ((x - flat.centre.0).powi(2) + (z - flat.centre.1).powi(2)).sqrt();
                    let w = 1.0 - smoothstep(flat.radius, flat.radius + flat.blend, d);
                    if w > 0.0 {
                        let k = j * GRID + i;
                        grid.h[k] += (target - grid.h[k]) * w;
                    }
                }
            }
        }
        // Roads: the height along each is the land's, smoothed, and the road is laid
        // level across.
        for road in &a.roads {
            let length = polyline_length(&road.points);
            let mut values: Vec<f32> = (0..=length.ceil() as usize)
                .map(|s| {
                    let (x, z) = point_along(&road.points, s as f32);
                    grid.height_at(x, z)
                })
                .collect();
            for _ in 0..3 {
                let previous = values.clone();
                for (s, v) in values.iter_mut().enumerate() {
                    let window = 9usize;
                    let (lo, hi) = (s.saturating_sub(window), (s + window).min(previous.len() - 1));
                    *v = previous[lo..=hi].iter().sum::<f32>() / (hi - lo + 1) as f32;
                }
            }
            if let Some(grade) = road.grade {
                for s in 1..values.len() {
                    values[s] = values[s].clamp(values[s - 1] - grade, values[s - 1] + grade);
                }
                for s in (0..values.len() - 1).rev() {
                    values[s] = values[s].clamp(values[s + 1] - grade, values[s + 1] + grade);
                }
            }
            let profile = Profile { step: 1.0, values };
            let shoulder = 2.5f32.max(grid.spacing * 1.5);
            let half = (road.width * 0.5).max(grid.spacing * 0.6);
            for j in 0..GRID {
                for i in 0..GRID {
                    let (x, z) = grid.position(i, j);
                    let (d, s) = nearest_on(&road.points, x, z);
                    let w = 1.0 - smoothstep(half, half + shoulder, d);
                    if w > 0.0 {
                        let k = j * GRID + i;
                        grid.h[k] += (profile.at(s) - grid.h[k]) * w;
                    }
                }
            }
        }
        // Rivers and ponds: cut down to a bed below the water, with banks that slope up
        // to the land, and never lower than the water just beyond the channel, so the
        // water always stands in a hollow.
        for j in 0..GRID {
            for i in 0..GRID {
                let (x, z) = grid.position(i, j);
                let k = j * GRID + i;
                for (water, lip, d, half, depth, bank) in self.waters(x, z, grid.spacing) {
                    grid.h[k] = carve(grid.h[k], water, lip, d, half, depth, bank);
                }
            }
        }
    }

    /// Every body of water near a point: its level there, the level its bank must hold
    /// (the river's level a little upstream, which is never lower than the water beside
    /// the bank, even at a bend), the point's distance from its middle line or centre,
    /// its half-width, depth, and bank.
    fn waters(&self, x: f32, z: f32, spacing: f32) -> Vec<(f32, f32, f32, f32, f32, f32)> {
        let mut out = Vec::new();
        for (river, levels) in self.a.rivers.iter().zip(&self.river_levels) {
            // A river is never narrower than two samples of the grid, or the far ring
            // would lose it.
            let half = (river.width * 0.5).max(spacing);
            let (d, s) = nearest_on(&river.points, x, z);
            if d < half + river.bank.max(spacing * 1.5) + spacing * 2.0 + 30.0 {
                let bank = river.bank.max(spacing * 1.5);
                let upstream = levels.at((s - half - bank - spacing * 2.0).max(0.0));
                out.push((levels.at(s), upstream, d, half, river.depth, bank));
            }
        }
        for (pond, &level) in self.a.ponds.iter().zip(&self.pond_levels) {
            let d = ((x - pond.centre.0).powi(2) + (z - pond.centre.1).powi(2)).sqrt();
            let half = pond.radius.max(spacing);
            if d < half + pond.bank + 30.0 {
                out.push((level, level, d, half, pond.depth, pond.bank.max(spacing * 1.5)));
            }
        }
        out
    }

    /// Step 4b: water and ground for every sample of a finished grid.
    fn finish(&self, grid: &Grid) -> (Vec<f32>, Vec<Ground>) {
        let a = self.a;
        let mut water = vec![f32::NAN; GRID * GRID];
        let mut ground = vec![Ground::Grass; GRID * GRID];
        let s = &a.shape;
        for j in 0..GRID {
            for i in 0..GRID {
                let (x, z) = grid.position(i, j);
                let k = j * GRID + i;
                let h = grid.h[k];
                let mut g = Ground::Grass;
                let away = x.abs().max(z.abs());
                if away > s.forest.beyond {
                    let cover = fbm(x, z, s.forest.wavelength, grid.spacing.max(16.0), a.seed ^ 0xf0)
                        * 0.5
                        + 0.5
                        + smoothstep(s.forest.beyond, s.forest.beyond * 1.6, away) * 0.15;
                    if cover > 1.0 - s.forest.cover {
                        g = Ground::Forest;
                    }
                }
                if a.woods.iter().any(|w| inside(&w.points, x, z)) {
                    g = Ground::Forest;
                }
                if a.fields.iter().any(|f| inside(&f.points, x, z)) {
                    g = Ground::Field;
                }
                for flat in &a.flats {
                    if let Some(name) = flat.ground {
                        if ((x - flat.centre.0).powi(2) + (z - flat.centre.1).powi(2)).sqrt() < flat.radius {
                            g = name.ground();
                        }
                    }
                }
                for road in &a.roads {
                    if nearest_on(&road.points, x, z).0 < (road.width * 0.5).max(grid.spacing * 0.6) {
                        g = Ground::Road;
                    }
                }
                for (level, _, d, half, _, bank) in self.waters(x, z, grid.spacing) {
                    if d < half && level > h {
                        water[k] = if water[k].is_nan() { level } else { water[k].max(level) };
                    } else if d < half + bank && h < level + 1.2 {
                        g = Ground::Bank;
                    }
                }
                if !water[k].is_nan() {
                    g = Ground::Bed;
                }
                if self.erased(x, z) > 0.5 {
                    g = Ground::Erasure;
                }
                ground[k] = g;
            }
        }
        (water, ground)
    }
}

/// The ground at distance `d` from a body of water's middle: a rounded bed below the
/// water within the channel, then a bank sloping up to the land.
fn carve(h: f32, water: f32, lip: f32, d: f32, half: f32, depth: f32, bank: f32) -> f32 {
    if d < half {
        let t = d / half;
        return h.min(water - depth * (1.0 - t * t) - 0.05);
    }
    let lip = lip.max(water) + 0.15;
    // The bank: cut down to a slope rising from the lip, steepening beyond the bank so
    // that a brook makes a gully and not a valley, and raised to the lip where the land
    // would let the water out.
    let past = (d - half - bank).max(0.0);
    let slope = lip + (d - half).min(bank) * 0.4 + past * 1.2;
    let cut = h.min(slope);
    if d < half + bank {
        cut.max(lip)
    } else {
        cut
    }
}

// ---- the bake ----

pub struct BakedTerrain {
    pub tiles: Vec<TerrainTile>,
    /// Each level's whole grid, for the maps: level, half-width, heights, water, ground.
    pub grids: Vec<(u8, f32, Vec<f32>, Vec<f32>, Vec<Ground>)>,
}

fn read(path: &Path) -> Result<(Authored, Vec<u8>), BakeError> {
    let bytes = std::fs::read(path).map_err(|e| BakeError(format!("{}: {e}", path.display())))?;
    let text = String::from_utf8(bytes.clone()).map_err(|_| BakeError(format!("{}: not UTF-8", path.display())))?;
    let authored: Authored = ron::Options::default()
        .from_str(&text)
        .map_err(|e| BakeError(format!("{}: {e}", path.display())))?;
    let mut keys = std::collections::BTreeSet::new();
    let all = authored
        .hills
        .iter()
        .map(|h| &h.key)
        .chain(authored.flats.iter().map(|f| &f.key))
        .chain(authored.roads.iter().map(|r| &r.key))
        .chain(authored.rivers.iter().map(|r| &r.key))
        .chain(authored.ponds.iter().map(|p| &p.key))
        .chain(authored.fields.iter().map(|f| &f.key))
        .chain(authored.woods.iter().map(|w| &w.key));
    for key in all {
        if !keys.insert(key.clone()) {
            return err(format!("{}: the key `{key}` is used twice", path.display()));
        }
    }
    for line in authored.roads.iter().map(|r| (&r.key, r.points.len())).chain(authored.rivers.iter().map(|r| (&r.key, r.points.len()))) {
        if line.1 < 2 {
            return err(format!("{}: `{}` needs two points or more", path.display(), line.0));
        }
    }
    for area in authored.fields.iter().chain(&authored.woods) {
        if area.points.len() < 3 {
            return err(format!("{}: `{}` needs three points or more", path.display(), area.key));
        }
    }
    Ok((authored, bytes))
}

/// Makes the land. Deterministic: the same file always gives the same tiles.
pub fn generate(path: &Path) -> Result<BakedTerrain, BakeError> {
    let (authored, _) = read(path)?;
    let land = Land::new(&authored);
    let mut grids: Vec<Grid> = Vec::new();
    for (index, &(level, half)) in LEVELS.iter().enumerate() {
        let spacing = half * 2.0 / (GRID - 1) as f32;
        let mut grid = Grid { level, half, spacing, h: vec![0.0; GRID * GRID] };
        for j in 0..GRID {
            for i in 0..GRID {
                let (x, z) = grid.position(i, j);
                grid.h[j * GRID + i] = land.shape(x, z, spacing);
            }
        }
        if level < 5 {
            erode_hydraulic(&mut grid, GRID * GRID / 2, authored.seed as u64 ^ (level as u64) << 32);
            erode_thermal(&mut grid, 24, 1.1);
        }
        land.shape_landmarks(&mut grid);
        // Ease into the coarser grid near the border, so that the two are one ground.
        if let Some(coarser) = grids.last() {
            let band = spacing * 28.0;
            for j in 0..GRID {
                for i in 0..GRID {
                    let (x, z) = grid.position(i, j);
                    let inward = half - x.abs().max(z.abs());
                    let w = smoothstep(0.0, band, inward);
                    let k = j * GRID + i;
                    let outer = coarser.height_at(x, z);
                    grid.h[k] = outer + (grid.h[k] - outer) * w;
                }
            }
        }
        grids.push(grid);
        let _ = index;
    }
    // Write each finer grid into the coarser ones wherever they share a sample.
    for f in (1..grids.len()).rev() {
        let (coarse_part, fine_part) = grids.split_at_mut(f);
        let fine = &fine_part[0];
        for coarse in coarse_part.iter_mut() {
            let ratio = (coarse.spacing / fine.spacing).round() as usize;
            for j in 0..GRID {
                for i in 0..GRID {
                    let (x, z) = coarse.position(i, j);
                    if !fine.contains(x, z) {
                        continue;
                    }
                    let fi = ((x + fine.half) / fine.spacing).round() as usize;
                    let fj = ((z + fine.half) / fine.spacing).round() as usize;
                    if fi < GRID && fj < GRID && ratio > 0 {
                        coarse.h[j * GRID + i] = fine.h[fj * GRID + fi];
                    }
                }
            }
        }
    }

    // Water and ground for every grid; where a coarser grid has a finer one's sample,
    // it takes the finer one's water and ground with its height, so the two agree.
    let mut finished: Vec<(Vec<f32>, Vec<Ground>)> = grids.iter().map(|g| land.finish(g)).collect();
    for f in (1..grids.len()).rev() {
        for c in 0..f {
            let (fine, coarse) = (&grids[f], &grids[c]);
            for j in 0..GRID {
                for i in 0..GRID {
                    let (x, z) = coarse.position(i, j);
                    // Strictly inside: on the border both grids are the coarser's ground.
                    if x.abs().max(z.abs()) >= fine.half - fine.spacing * 0.5 {
                        continue;
                    }
                    let fi = ((x + fine.half) / fine.spacing).round() as usize;
                    let fj = ((z + fine.half) / fine.spacing).round() as usize;
                    let (w, g) = (finished[f].0[fj * GRID + fi], finished[f].1[fj * GRID + fi]);
                    finished[c].0[j * GRID + i] = w;
                    finished[c].1[j * GRID + i] = g;
                }
            }
        }
    }

    let mut tiles = Vec::new();
    let mut maps = Vec::new();
    for (grid, (water, ground)) in grids.iter().zip(finished) {
        let per = GRID / 8;
        for tz in 0..8 {
            for tx in 0..8 {
                let (x0, z0) = grid.position(tx * per, tz * per);
                let coord = TileCoord::containing(x0 + grid.spacing * 0.5, z0 + grid.spacing * 0.5, grid.level);
                let mut tile = TerrainTile::new(coord);
                for j in 0..TILE_SAMPLES {
                    for i in 0..TILE_SAMPLES {
                        let k = (tz * per + j) * GRID + tx * per + i;
                        let t = TerrainTile::index(i, j);
                        tile.heights[t] = grid.h[k];
                        tile.water[t] = water[k];
                        tile.ground[t] = ground[k];
                    }
                }
                tiles.push(tile);
            }
        }
        maps.push((grid.level, grid.half, grid.h.clone(), water, ground));
    }
    Ok(BakedTerrain { tiles, grids: maps })
}

/// Bakes the terrain into tiles, or returns the cached tiles if neither the file nor the
/// generator has changed.
pub fn bake_tiles(path: &Path, cache: Option<&Path>) -> Result<Vec<(AssetId, Vec<u8>)>, BakeError> {
    let (_, bytes) = read(path)?;
    let mut key = VERSION.as_bytes().to_vec();
    key.extend_from_slice(&sha256(&bytes));
    let cache_file = cache.map(|dir| dir.join(format!("terrain-{}.bin", &hex(&sha256(&key))[..16])));
    if let Some(file) = &cache_file {
        if let Ok(cached) = std::fs::read(file) {
            if let Some(tiles) = split_cached(&cached) {
                return Ok(tiles);
            }
        }
    }
    let baked = generate(path)?;
    let tiles: Vec<(AssetId, Vec<u8>)> = baked.tiles.iter().map(|t| (t.coord.asset_id(TERRAIN_LAYER), t.to_bytes())).collect();
    if let Some(file) = &cache_file {
        let mut out = Vec::new();
        for (id, bytes) in &tiles {
            out.extend_from_slice(&id.0.to_le_bytes());
            out.extend_from_slice(&(bytes.len() as u32).to_le_bytes());
            out.extend_from_slice(bytes);
        }
        if let Some(dir) = file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(file, out);
    }
    Ok(tiles)
}

fn split_cached(mut bytes: &[u8]) -> Option<Vec<(AssetId, Vec<u8>)>> {
    let mut out = Vec::new();
    while !bytes.is_empty() {
        let id = u64::from_le_bytes(bytes.get(..8)?.try_into().ok()?);
        let len = u32::from_le_bytes(bytes.get(8..12)?.try_into().ok()?) as usize;
        let tile = bytes.get(12..12 + len)?;
        TerrainTile::from_bytes(tile).ok()?;
        out.push((AssetId(id), tile.to_vec()));
        bytes = &bytes[12 + len..];
    }
    (!out.is_empty()).then_some(out)
}

/// A shaded map of one grid, to look at: hill-shaded, coloured by ground, with water.
/// Returns RGBA8, `GRID` pixels square, north up.
pub fn map_image(spacing: f32, heights: &[f32], water: &[f32], ground: &[Ground]) -> Vec<u8> {
    let mut rgba = vec![0u8; GRID * GRID * 4];
    let lowest = heights.iter().copied().fold(f32::INFINITY, f32::min);
    let highest = heights.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    for j in 0..GRID {
        for i in 0..GRID {
            let k = j * GRID + i;
            let at = |a: usize, b: usize| heights[b.min(GRID - 1) * GRID + a.min(GRID - 1)];
            let dx = (at(i + 1, j) - at(i.saturating_sub(1), j)) / (2.0 * spacing);
            let dz = (at(i, j + 1) - at(i, j.saturating_sub(1))) / (2.0 * spacing);
            // Light from the north-west, high.
            let n = [-dx, 1.0, -dz];
            let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
            let light = ((-n[0] * 0.5 - n[2] * 0.5 + n[1] * 0.7) / len / 1.0).clamp(0.0, 1.0);
            let base: [f32; 3] = match ground[k] {
                Ground::Grass => [0.42, 0.55, 0.28],
                Ground::Field => [0.80, 0.70, 0.38],
                Ground::Road => [0.62, 0.52, 0.40],
                Ground::Earth => [0.50, 0.40, 0.30],
                Ground::Bank => [0.45, 0.42, 0.34],
                Ground::Bed => [0.30, 0.30, 0.26],
                Ground::Forest => [0.20, 0.34, 0.18],
                Ground::Erasure => [0.96, 0.95, 0.92],
            };
            let tint = 0.85 + 0.3 * (heights[k] - lowest) / (highest - lowest).max(1.0);
            let mut c = base.map(|v| v * (0.35 + 0.75 * light) * tint);
            if !water[k].is_nan() {
                let depth = (water[k] - heights[k]).clamp(0.0, 3.0) / 3.0;
                c = [0.20 - depth * 0.08, 0.38 - depth * 0.1, 0.52 - depth * 0.1];
            }
            for (o, v) in rgba[k * 4..k * 4 + 3].iter_mut().zip(c) {
                *o = (v.clamp(0.0, 1.0).powf(1.0 / 2.2) * 255.0).round() as u8;
            }
            rgba[k * 4 + 3] = 255;
        }
    }
    rgba
}

pub const GRID_SAMPLES: usize = GRID;

#[cfg(test)]
mod tests {
    use super::*;

    fn valley() -> BakedTerrain {
        let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/terrain/valley.ron");
        generate(&root).expect("the valley bakes")
    }

    #[test]
    fn the_valley_is_tiles_that_meet_without_seams() {
        let baked = valley();
        assert_eq!(baked.tiles.len(), 3 * 64);
        let by_coord: std::collections::HashMap<TileCoord, &TerrainTile> = baked.tiles.iter().map(|t| (t.coord, t)).collect();
        for tile in &baked.tiles {
            assert!(tile.heights.iter().all(|h| h.is_finite()), "{:?} has a height that is not a number", tile.coord);
            // The east neighbour shares this tile's east edge, sample for sample.
            let east = TileCoord { x: tile.coord.x + 1, ..tile.coord };
            if let Some(next) = by_coord.get(&east) {
                for j in 0..TILE_SAMPLES {
                    assert_eq!(tile.heights[TerrainTile::index(TILE_SAMPLES - 1, j)], next.heights[TerrainTile::index(0, j)]);
                }
            }
            let south = TileCoord { z: tile.coord.z + 1, ..tile.coord };
            if let Some(next) = by_coord.get(&south) {
                for i in 0..TILE_SAMPLES {
                    assert_eq!(tile.heights[TerrainTile::index(i, TILE_SAMPLES - 1)], next.heights[TerrainTile::index(i, 0)]);
                }
            }
        }
        // Where the fine area meets the far ring, the ground is the same ground: along
        // the border the fine edge lies on the coarse triangles exactly.
        let (fine, coarse) = (&baked.grids[2], &baked.grids[1]);
        let coarse_grid = Grid { level: coarse.0, half: coarse.1, spacing: coarse.1 * 2.0 / 512.0, h: coarse.2.clone() };
        for i in 0..GRID {
            let x = -256.0 + i as f32;
            for (j, z) in [(0usize, -256.0f32), (GRID - 1, 256.0)] {
                let a = fine.2[j * GRID + i];
                let b = coarse_grid.height_at(x, z);
                assert!((a - b).abs() < 1.0e-3, "a seam at ({x}, {z}): {a} against {b}");
            }
        }
    }

    #[test]
    fn water_stands_in_hollows_and_rivers_run_downhill() {
        let baked = valley();
        for (level, _, heights, water, _) in &baked.grids {
            for j in 1..GRID - 1 {
                for i in 1..GRID - 1 {
                    let k = j * GRID + i;
                    if water[k].is_nan() {
                        continue;
                    }
                    // Each region is held to this on its own grid. Where a coarser grid
                    // carries a finer one's samples, its neighbours are several of the
                    // finer grid's samples apart, so it is not checked there, nor on the
                    // seam band where the two ease into each other and a river is carved
                    // at two widths.
                    let half = [8192.0f32, 2048.0, 256.0][[5u8, 3, 0].iter().position(|l| l == level).unwrap()];
                    let spacing = half / 256.0;
                    let (x, z) = (-half + i as f32 * spacing, -half + j as f32 * spacing);
                    let finer = match level {
                        5 => Some(2048.0f32),
                        3 => Some(256.0),
                        _ => None,
                    };
                    let away = x.abs().max(z.abs());
                    // This grid's own seam band at its outer border, where it eases into
                    // the coarser grid. In the playable area that is the last 28 m, past
                    // the steep edges; the far grids' are 2 km and more away.
                    if *level != 5 && away >= half - spacing * 30.0 {
                        continue;
                    }
                    if let Some(finer) = finer {
                        if away <= finer + spacing * 2.0 {
                            continue;
                        }
                    }
                    assert!(water[k] > heights[k], "level {level}: water below its bed");
                    // Every neighbour is wet too, or its ground stands above the water.
                    for nk in [k - 1, k + 1, k - GRID, k + GRID] {
                        assert!(
                            !water[nk].is_nan() || heights[nk] >= water[k] - 0.02,
                            "level {level}: water at {k} would spill over {nk}: {} against {}",
                            heights[nk],
                            water[k]
                        );
                    }
                }
            }
        }
        let (authored, _) = read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/terrain/valley.ron")).unwrap();
        let land = Land::new(&authored);
        for levels in &land.river_levels {
            assert!(levels.values.windows(2).all(|w| w[1] <= w[0]), "a river runs uphill");
        }
    }

    #[test]
    fn the_landmarks_are_where_the_data_puts_them() {
        let baked = valley();
        let (_, _, heights, water, ground) = &baked.grids[2];
        let at = |x: f32, z: f32| ((z + 256.0).round() as usize) * GRID + (x + 256.0).round() as usize;
        // Tithe Hill is the highest ground inside the edges.
        let top = heights[at(-112.0, 128.0)];
        for j in 40..GRID - 40 {
            for i in 40..GRID - 40 {
                assert!(heights[j * GRID + i] <= top + 0.5, "higher than the hilltop at ({i}, {j})");
            }
        }
        assert!(top > heights[at(42.0, -38.0)] + 20.0, "the hill stands well above the village");
        // The green is level.
        let green = heights[at(42.0, -38.0)];
        for (dx, dz) in [(20.0, 0.0), (-20.0, 0.0), (0.0, 20.0), (0.0, -20.0)] {
            assert!((heights[at(42.0 + dx, -38.0 + dz)] - green).abs() < 0.3);
        }
        assert!(!water[at(0.0, -118.0)].is_nan(), "the river runs where it is drawn");
        assert!(!water[at(100.0, -70.0)].is_nan(), "the mill pond holds water");
        assert_eq!(ground[at(-20.0, 8.0)], Ground::Road);
        assert_eq!(ground[at(-60.0, -20.0)], Ground::Field);
        // The road follows the land: no stretch of it steeper than the hill it comes down.
        let (authored, _) = read(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/terrain/valley.ron")).unwrap();
        let fine = Grid { level: 0, half: 256.0, spacing: 1.0, h: heights.clone() };
        let road = &authored.roads.iter().find(|r| r.key == "penance_road").unwrap().points;
        let length = polyline_length(road);
        let mut s = 0.0;
        while s + 4.0 < length {
            let (a, b) = (point_along(road, s), point_along(road, s + 4.0));
            let grade = (fine.height_at(b.0, b.1) - fine.height_at(a.0, a.1)).abs() / 4.0;
            // Where it crosses water a bridge will carry it.
            // The footbridge is built in S06; here the road dips to the brook.
            let near_water = [a, b].iter().any(|p| {
                let k = at(p.0, p.1);
                (k.saturating_sub(12 * GRID)..(k + 12 * GRID).min(GRID * GRID)).step_by(GRID).any(|r| (r.saturating_sub(12)..r + 12).any(|q| !water[q].is_nan()))
            });
            // It comes down off the hill as steep as the hill, about one in three.
            assert!(near_water || grade < 0.34, "the road is too steep at {s} m: {grade}");
            s += 2.0;
        }
        // The far ring has its woods and, to the north, the white band.
        let far: Vec<Ground> = baked.grids[0].4.clone();
        assert!(far.contains(&Ground::Forest));
        assert!(far.contains(&Ground::Erasure));
        assert!(baked.grids[1].4.contains(&Ground::Forest));
    }

    #[test]
    fn the_same_file_bakes_the_same_tiles() {
        let a = valley();
        let b = valley();
        for (x, y) in a.tiles.iter().zip(&b.tiles) {
            assert!(x.to_bytes() == y.to_bytes());
        }
    }
}
