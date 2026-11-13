//! Trees, grown in the bake from `assets/trees/kinds.ron`.
//!
//! A tree is a trunk that branches, each branch a tube of bark whose page runs round it
//! and up it, and at the ends of the finest branches, cards of leaves. Each kind is grown
//! from its seed the same way every time. Three levels of detail are made from the one
//! tree, each with fewer, coarser tubes and fewer, larger cards so the crown keeps its
//! cover; and a far form, two crossed cards carrying the tree's own silhouette, projected
//! from the finest level into an impostor material, so the far form is the same tree.

use crate::BakeError;
use glam::{Vec2, Vec3};
use serde::Deserialize;
use std::path::Path;
use vellmark_content::{AssetId, Mesh, Vertex};

/// One kind of tree, as authored.
#[derive(Clone, Debug, Deserialize)]
pub struct Kind {
    pub key: String,
    pub height: f32,
    pub trunk_radius: f32,
    pub crown: f32,
    pub spread: f32,
    pub branches: u32,
    pub depth: u32,
    pub leaf_card: f32,
    pub cards_per_twig: u32,
    pub lean: f32,
    pub seed: u32,
    /// How many years a tree of this kind has stood when it has grown to its size at a
    /// scale of 1. Generated truth: its bark records it.
    pub age: f32,
}

#[derive(Deserialize)]
struct Kinds {
    kinds: Vec<Kind>,
}

pub fn read_kinds(path: &Path) -> Result<Vec<Kind>, BakeError> {
    let text =
        std::fs::read_to_string(path).map_err(|e| BakeError(format!("{}: {e}", path.display())))?;
    let kinds: Kinds =
        ron::from_str(&text).map_err(|e| BakeError(format!("{}: {e}", path.display())))?;
    Ok(kinds.kinds)
}

/// The levels of detail, finest first, then the far form.
pub const LEVELS: [&str; 4] = ["lod0", "lod1", "lod2", "far"];

/// The asset ID of a kind's mesh at a level.
pub fn mesh_id(kind: &str, level: &str) -> AssetId {
    AssetId::from_key(&format!("tree.{kind}.{level}"))
}

/// The key of a kind's impostor material, which the far form is drawn in.
pub fn impostor_key(kind: &str) -> String {
    format!("impostor.{kind}")
}

struct Rng(u32);

impl Rng {
    fn next(&mut self) -> f32 {
        self.0 = self.0.wrapping_mul(747_796_405).wrapping_add(2_891_336_453);
        let mut x = self.0;
        x ^= x >> 16;
        x = x.wrapping_mul(0x7feb_352d);
        x ^= x >> 15;
        (x & 0x00ff_ffff) as f32 / 16_777_216.0
    }

    fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.next()
    }
}

/// One branch: its centre line, its radius at each point, and how many times removed
/// from the trunk it is.
struct Branch {
    points: Vec<Vec3>,
    radii: Vec<f32>,
    depth: u32,
}

impl Branch {
    fn length(&self) -> f32 {
        self.points.windows(2).map(|w| w[0].distance(w[1])).sum()
    }

    /// The point, direction, and radius a share `t` of the way along.
    fn at(&self, t: f32) -> (Vec3, Vec3, f32) {
        let n = self.points.len() - 1;
        let x = (t.clamp(0.0, 1.0) * n as f32).min(n as f32 - 1.0e-4);
        let i = x.floor() as usize;
        let f = x - i as f32;
        let p = self.points[i].lerp(self.points[i + 1], f);
        let d = (self.points[i + 1] - self.points[i]).normalize_or(Vec3::Y);
        (
            p,
            d,
            self.radii[i] + (self.radii[i + 1] - self.radii[i]) * f,
        )
    }
}

/// One card of leaves: its middle, the way it faces, its up, and its side.
struct Card {
    centre: Vec3,
    normal: Vec3,
    up: Vec3,
    size: f32,
}

struct Tree {
    branches: Vec<Branch>,
    cards: Vec<Card>,
}

/// A direction at an angle `tilt` from `d`, turned `azimuth` around it.
fn turned(d: Vec3, azimuth: f32, tilt: f32) -> Vec3 {
    let side = if d.y.abs() < 0.95 {
        d.cross(Vec3::Y).normalize()
    } else {
        d.cross(Vec3::X).normalize()
    };
    let other = d.cross(side);
    let around = side * azimuth.cos() + other * azimuth.sin();
    (d * tilt.cos() + around * tilt.sin()).normalize()
}

/// A branch from `start` along `dir`, bending under its own weight and towards the light.
fn limb(
    start: Vec3,
    dir: Vec3,
    length: f32,
    r0: f32,
    r1: f32,
    segments: usize,
    depth: u32,
    rng: &mut Rng,
) -> Branch {
    let mut points = vec![start];
    let mut radii = vec![r0];
    let mut d = dir;
    let step = length / segments as f32;
    for s in 1..=segments {
        let t = s as f32 / segments as f32;
        // Long low branches droop; all reach a little up at their ends.
        d = (d + Vec3::new(
            rng.range(-0.12, 0.12),
            -0.05 * depth as f32 + 0.08 * t,
            rng.range(-0.12, 0.12),
        ))
        .normalize();
        points.push(points[s - 1] + d * step);
        radii.push(r0 + (r1 - r0) * t);
    }
    Branch {
        points,
        radii,
        depth,
    }
}

fn grow(kind: &Kind) -> Tree {
    let mut rng = Rng(kind.seed.wrapping_mul(0x9e37_79b9) ^ 0x51ed);
    let lean_dir = rng.range(0.0, std::f32::consts::TAU);
    let up = (Vec3::Y + Vec3::new(lean_dir.cos(), 0.0, lean_dir.sin()) * kind.lean).normalize();
    let trunk = limb(
        Vec3::ZERO,
        up,
        kind.height * 0.92,
        kind.trunk_radius,
        kind.trunk_radius * 0.18,
        7,
        0,
        &mut rng,
    );
    let mut branches = vec![trunk];
    let mut cards = Vec::new();
    let mut parents = vec![0usize];
    let golden = 137.508f32.to_radians();
    for depth in 1..=kind.depth {
        let mut children = Vec::new();
        for &p in &parents {
            let (count, from) = if depth == 1 {
                (kind.branches, kind.crown)
            } else {
                ((kind.branches / 2).max(2), 0.35)
            };
            let parent_length = branches[p].length();
            for k in 0..count {
                let t = from + (0.97 - from) * (k as f32 + rng.range(0.2, 0.8)) / count as f32;
                let (at, dir, radius) = branches[p].at(t);
                let azimuth = k as f32 * golden + rng.range(-0.3, 0.3);
                let tilt = (kind.spread + rng.range(-12.0, 12.0)).to_radians();
                let mut d = turned(dir, azimuth, tilt);
                d = (d + Vec3::Y * 0.2).normalize();
                // Branches high up are shorter, so the crown rounds off.
                let length = parent_length
                    * if depth == 1 { 0.48 } else { 0.5 }
                    * (1.2 - 0.55 * t)
                    * rng.range(0.8, 1.15);
                let r0 = (radius * 0.62).max(0.01);
                let segments = if depth == 1 { 4 } else { 3 };
                children.push(branches.len());
                branches.push(limb(
                    at,
                    d,
                    length,
                    r0,
                    r0 * 0.25,
                    segments,
                    depth,
                    &mut rng,
                ));
            }
        }
        parents = children;
    }
    // Leaves at the ends of the finest branches, and along their outer part.
    let crown_centre = Vec3::Y * kind.height * (0.5 + kind.crown * 0.5);
    for &b in &parents {
        for c in 0..kind.cards_per_twig {
            let t = 0.45 + 0.55 * (c as f32 + rng.next()) / kind.cards_per_twig as f32;
            let (at, _, _) = branches[b].at(t);
            let outward = (at - crown_centre).normalize_or(Vec3::Y);
            let jitter = Vec3::new(
                rng.range(-1.0, 1.0),
                rng.range(-0.6, 1.0),
                rng.range(-1.0, 1.0),
            );
            let normal = (outward + jitter * 0.8).normalize_or(Vec3::Y);
            let up = turned(
                normal,
                rng.range(0.0, std::f32::consts::TAU),
                std::f32::consts::FRAC_PI_2,
            );
            let size = kind.leaf_card * rng.range(0.8, 1.2);
            cards.push(Card {
                centre: at + jitter * size * 0.15,
                normal,
                up,
                size,
            });
        }
    }
    Tree { branches, cards }
}

fn vertex(
    position: Vec3,
    normal: Vec3,
    tangent: Vec3,
    w: f32,
    anchor: Vec2,
    material: u32,
    thickness: f32,
    sky: f32,
) -> Vertex {
    Vertex {
        position: position.to_array(),
        normal: normal.to_array(),
        tangent: [tangent.x, tangent.y, tangent.z, w],
        anchor: anchor.to_array(),
        material,
        page: 0,
        thickness,
        sky,
    }
}

/// The sky a point of the tree sees: less low down and deep inside the crown.
fn sky_of(p: Vec3, kind: &Kind) -> f32 {
    let height = (p.y / kind.height).clamp(0.0, 1.0);
    (0.35 + 0.65 * height).clamp(0.3, 1.0)
}

fn tube(mesh: &mut Mesh, b: &Branch, sides: usize, bark: u32, kind: &Kind) {
    let base = mesh.vertices.len() as u32;
    let mut along = 0.0;
    for (i, &p) in b.points.iter().enumerate() {
        if i > 0 {
            along += p.distance(b.points[i - 1]);
        }
        let d = if i + 1 < b.points.len() {
            b.points[i + 1] - p
        } else {
            p - b.points[i - 1]
        }
        .normalize_or(Vec3::Y);
        let side = if d.y.abs() < 0.95 {
            d.cross(Vec3::Y).normalize()
        } else {
            d.cross(Vec3::X).normalize()
        };
        let other = d.cross(side);
        let r = b.radii[i];
        // The page runs round the branch, then up it; one seam where the ring closes.
        for j in 0..=sides {
            let a = j as f32 / sides as f32 * std::f32::consts::TAU;
            let out = side * a.cos() + other * a.sin();
            let round = (-side * a.sin() + other * a.cos()).normalize();
            let w = if out.cross(round).dot(d) >= 0.0 {
                1.0
            } else {
                -1.0
            };
            let position = p + out * r;
            mesh.vertices.push(vertex(
                position,
                out,
                round,
                w,
                Vec2::new(a * b.radii[0], along),
                bark,
                r * 2.0,
                sky_of(position, kind),
            ));
        }
    }
    let ring = sides as u32 + 1;
    for i in 0..b.points.len() as u32 - 1 {
        for j in 0..sides as u32 {
            let a = base + i * ring + j;
            let c = a + ring;
            mesh.indices
                .extend_from_slice(&[a, a + 1, c + 1, a, c + 1, c]);
        }
    }
}

/// Each card's page is set this far along from the last, so that the shader can tell
/// the cards apart and cut each into its own spray of leaves. Matches `CARD_STRIDE` in
/// `shaders/opaque.wgsl`.
pub const CARD_STRIDE: f32 = 100.0;

fn card(mesh: &mut Mesh, c: &Card, scale: f32, leaves: u32, kind: &Kind, index: usize) {
    let base = mesh.vertices.len() as u32;
    let right = c.up.cross(c.normal).normalize();
    let half = c.size * scale * 0.5;
    for (x, y) in [(-1.0, -1.0), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
        let position = c.centre + right * x * half + c.up * y * half;
        let anchor = Vec2::new(
            (x + 1.0) * half + (index % 64) as f32 * CARD_STRIDE,
            (y + 1.0) * half,
        );
        // The card's side, which the shaders need to find their place on it, is carried
        // as its thickness.
        mesh.vertices.push(vertex(
            position,
            c.normal,
            right,
            1.0,
            anchor,
            leaves,
            half * 2.0,
            sky_of(position, kind),
        ));
    }
    mesh.indices
        .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
}

/// The crossed cards of the far form: as wide as the crown, as tall as the tree.
fn far_form(mesh: &mut Mesh, impostor: u32, half_width: f32, top: f32) {
    for axis in [Vec3::X, Vec3::Z] {
        let normal = axis.cross(Vec3::Y).normalize();
        let base = mesh.vertices.len() as u32;
        // The impostor's page is one tile of the material: a side of one metre.
        for (x, y) in [(-1.0, 0.0), (1.0, 0.0), (1.0, 1.0), (-1.0, 1.0)] {
            let position = axis * x * half_width + Vec3::Y * y * top;
            let anchor = Vec2::new((x + 1.0) * 0.5, y);
            mesh.vertices.push(vertex(
                position, normal, axis, 1.0, anchor, impostor, half_width, 0.8,
            ));
        }
        mesh.indices
            .extend_from_slice(&[base, base + 1, base + 2, base, base + 2, base + 3]);
    }
}

/// How high the tree reaches, leaves and all.
fn top_of(tree: &Tree) -> f32 {
    let branches = tree
        .branches
        .iter()
        .flat_map(|b| b.points.iter())
        .map(|p| p.y);
    let cards = tree.cards.iter().map(|c| c.centre.y + c.size * 0.6);
    branches.chain(cards).fold(1.0, f32::max)
}

/// How far the crown reaches from the trunk's line, at most.
fn crown_half_width(tree: &Tree) -> f32 {
    let reach = |p: Vec3| Vec2::new(p.x, p.z).length();
    let branches = tree
        .branches
        .iter()
        .flat_map(|b| b.points.iter())
        .map(|p| reach(*p));
    let cards = tree.cards.iter().map(|c| reach(c.centre) + c.size * 0.5);
    branches.chain(cards).fold(0.5, f32::max)
}

/// Every level of a kind: its mesh ID and the mesh. `bark`, `leaves`, and `impostor` are
/// material indices in the set.
pub fn meshes(kind: &Kind, bark: u32, leaves: u32, impostor: u32) -> Vec<(AssetId, Mesh)> {
    let tree = grow(kind);
    let mut out = Vec::new();
    // Sides of the trunk, of a branch, and of the finest; the deepest branches drawn; and
    // one card in how many, made how much larger.
    let levels: [(usize, usize, usize, u32, usize, f32); 3] = [
        (9, 6, 4, kind.depth, 1, 1.0),
        (6, 4, 3, kind.depth.saturating_sub(1).max(1), 2, 1.38),
        (4, 3, 3, 1, 5, 2.1),
    ];
    for (level, &(trunk_sides, branch_sides, twig_sides, deepest, every, scale)) in
        levels.iter().enumerate()
    {
        let mut mesh = Mesh::default();
        for b in tree.branches.iter().filter(|b| b.depth <= deepest) {
            let sides = match b.depth {
                0 => trunk_sides,
                1 => branch_sides,
                _ => twig_sides,
            };
            tube(&mut mesh, b, sides, bark, kind);
        }
        for (i, c) in tree.cards.iter().enumerate().step_by(every) {
            card(&mut mesh, c, scale, leaves, kind, i);
        }
        out.push((mesh_id(&kind.key, LEVELS[level]), mesh));
    }
    let mut far = Mesh::default();
    far_form(&mut far, impostor, crown_half_width(&tree), top_of(&tree));
    out.push((mesh_id(&kind.key, LEVELS[3]), far));
    out
}

/// Round the trunk at its foot, in the metres of the bark's page: the page runs round
/// each branch at the radius of its foot.
pub fn girth(kind: &Kind) -> f32 {
    std::f32::consts::TAU * grow(kind).branches[0].radii[0]
}

/// How many years a placed tree has stood: its kind's age at full size, older as it is
/// larger, and a little older or younger by where it stands.
pub fn years(kind: &Kind, scale: f32, x: f32, z: f32) -> u32 {
    let mut rng = Rng((x.round() as i32 as u32).wrapping_mul(0x9e37_79b9)
        ^ (z.round() as i32 as u32).wrapping_mul(0x85eb_ca6b)
        ^ kind.seed);
    (kind.age * scale.powf(1.6) * rng.range(0.85, 1.15))
        .round()
        .max(3.0) as u32
}

/// Whether a point of a card, in the card's own square from 0 to 1, is leaf: a few
/// overlapping ovals, so a card is a spray of leaves and not a square.
pub fn leafy(u: f32, v: f32, seed: u32) -> bool {
    let mut rng = Rng(seed ^ 0xa5a5);
    for _ in 0..7 {
        let (cx, cy) = (rng.range(0.2, 0.8), rng.range(0.2, 0.8));
        let (rx, ry) = (rng.range(0.12, 0.24), rng.range(0.07, 0.14));
        let a = rng.range(0.0, std::f32::consts::PI);
        let (s, c) = a.sin_cos();
        let (dx, dy) = (u - cx, v - cy);
        let (x, y) = (dx * c + dy * s, -dx * s + dy * c);
        if (x / rx).powi(2) + (y / ry).powi(2) < 1.0 {
            return true;
        }
    }
    false
}

/// The far form's picture: the finest level seen from the side, `size` texels square,
/// stretched to the far form's card. Each texel: colour, and 1 where the tree covers it.
pub fn impostor(kind: &Kind, size: usize) -> Vec<([f32; 3], f32)> {
    let tree = grow(kind);
    let half = crown_half_width(&tree);
    let top = top_of(&tree);
    let mut out = vec![([0.0f32; 3], 0.0f32); size * size];
    let to_texel = |p: Vec3| {
        Vec2::new(
            (p.x / half * 0.5 + 0.5) * size as f32,
            (1.0 - p.y / top) * size as f32,
        )
    };
    let bark = [0.24, 0.19, 0.14];
    let leaf = match kind.key.as_str() {
        "apple" => [0.24, 0.36, 0.12],
        "hawthorn" => [0.17, 0.27, 0.09],
        "ash" => [0.2, 0.31, 0.12],
        _ => [0.15, 0.25, 0.08],
    };
    // Bark first, leaves over it: from the side, leaves hide most of the wood.
    let mut fill = |a: Vec2, b: Vec2, c: Vec2, paint: &dyn Fn(Vec2) -> Option<[f32; 3]>| {
        let lo = a.min(b).min(c).floor().max(Vec2::ZERO);
        let hi = a.max(b).max(c).ceil().min(Vec2::splat(size as f32 - 1.0));
        let area = (b - a).perp_dot(c - a);
        if area.abs() < 1.0e-6 {
            return;
        }
        for y in lo.y as usize..=hi.y as usize {
            for x in lo.x as usize..=hi.x as usize {
                let p = Vec2::new(x as f32 + 0.5, y as f32 + 0.5);
                let w0 = (b - p).perp_dot(c - p) / area;
                let w1 = (c - p).perp_dot(a - p) / area;
                let w2 = 1.0 - w0 - w1;
                if w0 >= 0.0 && w1 >= 0.0 && w2 >= 0.0 {
                    if let Some(colour) = paint(Vec2::new(w1, w2)) {
                        out[y * size + x] = (colour, 1.0);
                    }
                }
            }
        }
    };
    for b in &tree.branches {
        for w in b.points.windows(2).zip(b.radii.windows(2)) {
            let ((p0, p1), (r0, r1)) = ((w.0[0], w.0[1]), (w.1[0], w.1[1]));
            let side = Vec3::new(-(p1 - p0).y, (p1 - p0).x, 0.0).normalize_or(Vec3::X);
            let (a, bb, c, d) = (
                to_texel(p0 - side * r0),
                to_texel(p0 + side * r0),
                to_texel(p1 + side * r1),
                to_texel(p1 - side * r1),
            );
            fill(a, bb, c, &|_| Some(bark));
            fill(a, c, d, &|_| Some(bark));
        }
    }
    for (i, c) in tree.cards.iter().enumerate() {
        let right = c.up.cross(c.normal).normalize();
        let h = c.size * 0.5;
        let corners = [
            c.centre - right * h - c.up * h,
            c.centre + right * h - c.up * h,
            c.centre + right * h + c.up * h,
            c.centre - right * h + c.up * h,
        ];
        let t = corners.map(to_texel);
        // A little lighter where a card faces the light, from above and the side.
        let lit = 0.8 + 0.4 * c.normal.dot(Vec3::new(0.3, 0.8, 0.5).normalize()).max(0.0);
        let colour = [leaf[0] * lit, leaf[1] * lit, leaf[2] * lit];
        // The same spray the near levels cut from this card: see `leaves.wgsl`.
        let seed = (i % 64) as u32 * 7919;
        // Barycentric (w1, w2) on each triangle maps back to the card's own square.
        fill(t[0], t[1], t[2], &|w| {
            leafy(w.x + w.y, w.y, seed).then_some(colour)
        });
        fill(t[0], t[2], t[3], &|w| {
            leafy(w.x, w.x + w.y, seed).then_some(colour)
        });
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds() -> Vec<Kind> {
        read_kinds(&Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/trees/kinds.ron"))
            .unwrap()
    }

    #[test]
    fn every_kind_grows_the_same_every_time_and_each_level_is_lighter() {
        for kind in kinds() {
            let a = meshes(&kind, 1, 2, 3);
            let b = meshes(&kind, 1, 2, 3);
            assert_eq!(a.len(), 4);
            for ((ia, ma), (ib, mb)) in a.iter().zip(&b) {
                assert_eq!(ia, ib);
                assert_eq!(ma.vertices, mb.vertices, "{} grows the same", kind.key);
            }
            let triangles: Vec<usize> = a.iter().map(|(_, m)| m.indices.len() / 3).collect();
            assert!(
                triangles.windows(2).all(|w| w[1] < w[0]),
                "{}: each level lighter: {triangles:?}",
                kind.key
            );
            assert_eq!(triangles[3], 4, "the far form is two crossed cards");
            for (_, m) in &a {
                for v in &m.vertices {
                    let n = Vec3::from(v.normal);
                    assert!((n.length() - 1.0).abs() < 1.0e-3);
                    assert!(
                        v.position[1] > -0.5 && v.position[1] < kind.height * 1.6,
                        "{}: within its height",
                        kind.key
                    );
                }
                assert!(m.indices.iter().all(|&i| (i as usize) < m.vertices.len()));
            }
        }
    }

    #[test]
    fn the_impostor_is_the_tree_seen_from_the_side() {
        for kind in kinds() {
            let picture = impostor(&kind, 64);
            let covered =
                picture.iter().filter(|t| t.1 > 0.5).count() as f32 / picture.len() as f32;
            assert!(
                covered > 0.15 && covered < 0.85,
                "{}: the tree covers part of its card: {covered}",
                kind.key
            );
            // The trunk stands at the bottom middle.
            assert!(
                picture[63 * 64 + 32].1 > 0.5 || picture[62 * 64 + 31].1 > 0.5,
                "{}: the trunk reaches the ground",
                kind.key
            );
            // And the top corners are sky.
            assert!(picture[0].1 < 0.5 && picture[63].1 < 0.5);
        }
    }

    #[test]
    fn a_card_is_a_spray_of_leaves_not_a_square() {
        let n = 32;
        let leaf = (0..n * n)
            .filter(|i| leafy((i % n) as f32 / n as f32, (i / n) as f32 / n as f32, 5))
            .count() as f32
            / (n * n) as f32;
        assert!(leaf > 0.2 && leaf < 0.75, "{leaf}");
    }

    #[test]
    #[ignore]
    fn write_impostors() {
        let dir = std::env::var("TREES_OUT").unwrap();
        for kind in kinds() {
            let n = 128;
            let picture = impostor(&kind, n);
            let mut bytes = Vec::with_capacity(n * n * 3);
            for (c, a) in picture {
                let bg = [0.75, 0.85, 0.95];
                for k in 0..3 {
                    bytes.push(((c[k] * a + bg[k] * (1.0 - a)).clamp(0.0, 1.0) * 255.0) as u8);
                }
            }
            let file = std::fs::File::create(format!("{dir}/{}.png", kind.key)).unwrap();
            let mut e = png::Encoder::new(std::io::BufWriter::new(file), n as u32, n as u32);
            e.set_color(png::ColorType::Rgb);
            e.write_header().unwrap().write_image_data(&bytes).unwrap();
        }
    }
}
