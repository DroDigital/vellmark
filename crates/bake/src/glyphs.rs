//! The glyph atlas bake: both typefaces, 95 characters each, as distance fields with a
//! chain of smaller levels, and each glyph's lookup from tone to threshold.

use crate::sha256::{hex, sha256};
use crate::{err, BakeError};
use fdsm::bezier::scanline::FillRule;
use fdsm::bezier::{Point, Segment};
use fdsm::generate::generate_mtsdf;
use fdsm::render::correct_sign_mtsdf;
use fdsm::shape::{Contour, Shape};
use image::RgbaImage;
use std::path::Path;
use vellmark_content::{FaceInfo, GlyphAtlas, GlyphInfo, TONE_STEPS};

/// Side of a glyph's cell in texels. Large enough that a character filling the screen
/// keeps clean edges, because the distance field, not the texel, decides the edge.
pub const CELL: u32 = 128;
/// The distance covered by a texel's 0..1, in texels.
pub const RANGE: f64 = 16.0;
/// Levels stored: 128 down to 8.
pub const MIPS: u32 = 5;
/// Part of the cell the tallest glyph may fill, top to bottom. The rest is margin, so
/// that the distance field has room and neighbouring cells never touch.
const FILL: f64 = 0.80;
/// Bump when the bake changes what it writes for the same inputs.
const VERSION: &str = "glyphs-5";

struct Outline {
    contours: Vec<Contour>,
    current: Vec<Segment>,
    start: Point,
    at: Point,
    scale: f64,
    /// Horizontal narrowing for the few glyphs wider than a cell allows. 1 for the rest.
    squeeze: f64,
    origin: (f64, f64),
}

impl Outline {
    fn point(&self, x: f32, y: f32) -> Point {
        // Font units have y up; the image has y down.
        Point::new(self.origin.0 + x as f64 * self.scale * self.squeeze, self.origin.1 - y as f64 * self.scale)
    }

    fn finish_contour(&mut self) {
        if !self.current.is_empty() {
            self.contours.push(Contour { segments: std::mem::take(&mut self.current) });
        }
    }
}

impl ttf_parser::OutlineBuilder for Outline {
    fn move_to(&mut self, x: f32, y: f32) {
        self.finish_contour();
        self.start = self.point(x, y);
        self.at = self.start;
    }
    fn line_to(&mut self, x: f32, y: f32) {
        let p = self.point(x, y);
        if p != self.at {
            self.current.push(Segment::line(self.at, p));
        }
        self.at = p;
    }
    fn quad_to(&mut self, x1: f32, y1: f32, x: f32, y: f32) {
        let (c, p) = (self.point(x1, y1), self.point(x, y));
        self.current.push(Segment::quad(self.at, c, p));
        self.at = p;
    }
    fn curve_to(&mut self, x1: f32, y1: f32, x2: f32, y2: f32, x: f32, y: f32) {
        let (c1, c2, p) = (self.point(x1, y1), self.point(x2, y2), self.point(x, y));
        self.current.push(Segment::cubic(self.at, c1, c2, p));
        self.at = p;
    }
    fn close(&mut self) {
        if self.at != self.start {
            self.current.push(Segment::line(self.at, self.start));
        }
        self.at = self.start;
        self.finish_contour();
    }
}

struct BakedFace {
    info: FaceInfo,
    /// Level 0 texels of each glyph, RGBA8.
    glyphs: Vec<Vec<u8>>,
}

/// How a face's outlines become distance fields.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Method {
    /// From the outline's edges, with a channel for each edge colour. Keeps the sharp
    /// corners of a printed face at any size.
    Edges,
    /// From a filled picture of the glyph. A pixel face is drawn as many small squares
    /// that touch or overlap; their shared edges are not edges of the letter, and only
    /// filling them in first gives the letter's true outline.
    Raster,
}

/// Samples along each side of a texel when a glyph is filled before measuring.
const SUPER: usize = 4;

/// Fills the contours with the nonzero rule on a fine grid, then measures every
/// texel's distance to the edge of the filled shape.
fn raster_field(contours: &[Contour]) -> Vec<u8> {
    let n = CELL as usize * SUPER;
    // Flatten to line segments in fine-grid coordinates.
    let mut lines: Vec<(f64, f64, f64, f64)> = Vec::new();
    for contour in contours {
        for segment in &contour.segments {
            let steps = if matches!(segment.order(), fdsm::bezier::Order::Linear) { 1 } else { 16 };
            let mut a = segment.get(0.0);
            for i in 1..=steps {
                let b = segment.get(i as f64 / steps as f64);
                lines.push((a.x * SUPER as f64, a.y * SUPER as f64, b.x * SUPER as f64, b.y * SUPER as f64));
                a = b;
            }
        }
    }
    let mut inside = vec![false; n * n];
    let mut crossings: Vec<(f64, i32)> = Vec::new();
    for y in 0..n {
        let yc = y as f64 + 0.5;
        crossings.clear();
        for &(x0, y0, x1, y1) in &lines {
            if (y0 <= yc) != (y1 <= yc) {
                let x = x0 + (yc - y0) / (y1 - y0) * (x1 - x0);
                crossings.push((x, if y1 > y0 { 1 } else { -1 }));
            }
        }
        crossings.sort_by(|a, b| a.0.total_cmp(&b.0));
        let mut winding = 0;
        let mut next = 0;
        for x in 0..n {
            let xc = x as f64 + 0.5;
            while next < crossings.len() && crossings[next].0 <= xc {
                winding += crossings[next].1;
                next += 1;
            }
            inside[y * n + x] = winding != 0;
        }
    }
    let to_outside = distance_to(&inside, n, false);
    let to_inside = distance_to(&inside, n, true);
    let mut out = Vec::with_capacity((CELL * CELL * 4) as usize);
    for ty in 0..CELL as usize {
        for tx in 0..CELL as usize {
            // The texel's value is the mean signed distance of its samples, in texels.
            let mut sum = 0.0f64;
            for sy in 0..SUPER {
                for sx in 0..SUPER {
                    let i = (ty * SUPER + sy) * n + tx * SUPER + sx;
                    sum += if inside[i] { to_outside[i] - 0.5 } else { 0.5 - to_inside[i] };
                }
            }
            let texels = sum / (SUPER * SUPER) as f64 / SUPER as f64;
            let v = ((texels / RANGE + 0.5).clamp(0.0, 1.0) * 255.0).round() as u8;
            out.extend_from_slice(&[v, v, v, v]);
        }
    }
    out
}

/// For every cell of a square grid, the distance to the nearest cell whose `inside`
/// flag equals `target`. Exact, in two passes of the lower-envelope method.
fn distance_to(inside: &[bool], n: usize, target: bool) -> Vec<f64> {
    const FAR: f64 = 1.0e12;
    fn pass(f: &[f64], d: &mut [f64], v: &mut [usize], z: &mut [f64]) {
        let n = f.len();
        let mut k = 0;
        v[0] = 0;
        z[0] = -FAR;
        z[1] = FAR;
        for q in 1..n {
            loop {
                let p = v[k];
                let s = ((f[q] + (q * q) as f64) - (f[p] + (p * p) as f64)) / (2.0 * q as f64 - 2.0 * p as f64);
                if s <= z[k] && k > 0 {
                    k -= 1;
                } else {
                    k += 1;
                    v[k] = q;
                    z[k] = s;
                    z[k + 1] = FAR;
                    break;
                }
            }
        }
        k = 0;
        for (q, slot) in d.iter_mut().enumerate() {
            while z[k + 1] < q as f64 {
                k += 1;
            }
            let p = v[k];
            *slot = (q as f64 - p as f64).powi(2) + f[p];
        }
    }
    let mut grid: Vec<f64> = inside.iter().map(|&i| if i == target { 0.0 } else { FAR }).collect();
    let (mut f, mut d) = (vec![0.0; n], vec![0.0; n]);
    let (mut v, mut z) = (vec![0usize; n], vec![0.0; n + 1]);
    for x in 0..n {
        for y in 0..n {
            f[y] = grid[y * n + x];
        }
        pass(&f, &mut d, &mut v, &mut z);
        for y in 0..n {
            grid[y * n + x] = d[y];
        }
    }
    for y in 0..n {
        f.copy_from_slice(&grid[y * n..(y + 1) * n]);
        pass(&f, &mut d, &mut v, &mut z);
        grid[y * n..(y + 1) * n].copy_from_slice(&d);
    }
    grid.iter().map(|d2| d2.sqrt()).collect()
}

fn bake_face(bytes: &[u8], name: &str, method: Method) -> Result<BakedFace, BakeError> {
    let face = ttf_parser::Face::parse(bytes, 0).map_err(|e| BakeError(format!("{name}: {e}")))?;
    let em = face.units_per_em() as f64;

    // One scale for the whole face, from the tallest and deepest printable glyph, so
    // every character shares a baseline and a size.
    let (mut top, mut bottom, mut widest) = (0.0f64, 0.0f64, 0.0f64);
    for code in 32u8..=126 {
        let Some(id) = face.glyph_index(code as char) else {
            return err(format!("{name}: no glyph for {:?}", code as char));
        };
        if let Some(b) = face.glyph_bounding_box(id) {
            top = top.max(b.y_max as f64);
            bottom = bottom.min(b.y_min as f64);
            widest = widest.max((b.x_max - b.x_min) as f64);
        }
        widest = widest.max(face.glyph_hor_advance(id).unwrap_or(0) as f64);
    }
    // Sized by height, so that letters fill their cells. The widest letters come close to
    // the cell's edge; any that would cross the margin are narrowed to fit.
    let _ = widest;
    let scale = CELL as f64 * FILL / (top - bottom);
    let widest_allowed = CELL as f64 - RANGE;
    let cell_ems = (CELL as f64 / scale / em) as f32;
    // The band from the deepest descender to the tallest ascender is centred in the cell.
    let baseline = (CELL as f64 + (top + bottom) * scale) * 0.5;

    use rayon::prelude::*;
    let baked: Vec<(GlyphInfo, Vec<u8>)> = (32u8..=126).into_par_iter().map(|code| {
        // Each worker parses the face for itself; parsing only reads the table directory.
        let face = ttf_parser::Face::parse(bytes, 0).expect("parsed above");
        let id = face.glyph_index(code as char).unwrap();
        let advance = face.glyph_hor_advance(id).unwrap_or(0) as f64;
        // Centred by its inked box, which is what the eye centres on.
        let (x_min, x_max) = match face.glyph_bounding_box(id) {
            Some(b) => (b.x_min as f64, b.x_max as f64),
            None => (0.0, advance),
        };
        let squeeze = (widest_allowed / ((x_max - x_min) * scale).max(1.0)).min(1.0);
        let origin_x = CELL as f64 * 0.5 - (x_min + x_max) * 0.5 * scale * squeeze;
        let mut outline = Outline {
            contours: Vec::new(),
            current: Vec::new(),
            start: Point::new(0.0, 0.0),
            at: Point::new(0.0, 0.0),
            scale,
            squeeze,
            origin: (origin_x, baseline),
        };
        let has_outline = face.outline_glyph(id, &mut outline).is_some();
        outline.finish_contour();

        // An empty glyph (the space) is all zero: fully outside.
        let texels = if !has_outline || outline.contours.is_empty() {
            vec![0u8; (CELL * CELL * 4) as usize]
        } else if method == Method::Raster {
            raster_field(&outline.contours)
        } else {
            let mut image = RgbaImage::new(CELL, CELL);
            let shape = Shape { contours: outline.contours };
            let coloured = Shape::edge_coloring_simple(shape, 0.03, 0x5eed_0000 + code as u64);
            let prepared = coloured.prepare();
            generate_mtsdf(&prepared, RANGE, &mut image);
            correct_sign_mtsdf(&mut image, &prepared, FillRule::Nonzero);
            image.into_raw()
        };
        (glyph_info((advance / em) as f32, ((x_min + x_max) * 0.5 * squeeze / em) as f32, &texels), texels)
    }).collect();
    let (infos, glyphs): (Vec<GlyphInfo>, Vec<Vec<u8>>) = baked.into_iter().unzip();

    Ok(BakedFace {
        info: FaceInfo {
            ascent: (face.ascender() as f64 / em) as f32,
            descent: (face.descender() as f64 / em) as f32,
            cell_ems,
            baseline: (baseline / CELL as f64) as f32,
            glyphs: infos,
        },
        glyphs,
    })
}

/// Coverage of a cell when the edge is drawn at `threshold`, softened over one texel.
fn coverage_at(alpha: &[u8], threshold: f32) -> f32 {
    let per_texel = 1.0 / RANGE as f32;
    let sum: f32 = alpha
        .iter()
        .map(|&a| ((a as f32 / 255.0 - threshold) / per_texel + 0.5).clamp(0.0, 1.0))
        .sum();
    sum / alpha.len() as f32
}

/// Builds the lookup from target coverage to threshold for one glyph, from its true
/// distance field.
fn glyph_info(advance: f32, centre: f32, texels: &[u8]) -> GlyphInfo {
    let alpha: Vec<u8> = texels.chunks_exact(4).map(|t| t[3]).collect();
    // The threshold may move within the middle of the stored range, where the field is
    // still exact. Lower thresholds make heavier letters.
    let (heaviest, lightest) = (0.12f32, 0.88f32);
    let max_cover = coverage_at(&alpha, heaviest);
    let min_cover = coverage_at(&alpha, lightest);
    let mut tone = [0.5f32; TONE_STEPS];
    for (i, slot) in tone.iter_mut().enumerate() {
        let target = (i as f32 / (TONE_STEPS - 1) as f32).clamp(min_cover, max_cover);
        // Coverage falls as the threshold rises; bisect.
        let (mut lo, mut hi) = (heaviest, lightest);
        for _ in 0..20 {
            let mid = 0.5 * (lo + hi);
            if coverage_at(&alpha, mid) > target { lo = mid } else { hi = mid }
        }
        *slot = 0.5 * (lo + hi);
    }
    GlyphInfo { advance, centre, min_cover, max_cover, tone }
}

/// Halves a square RGBA image by averaging blocks of four. Distances average well, so
/// the smaller levels stay valid distance fields for small glyphs.
fn halve(texels: &[u8], side: u32) -> Vec<u8> {
    let half = side / 2;
    let mut out = Vec::with_capacity((half * half * 4) as usize);
    for y in 0..half {
        for x in 0..half {
            for c in 0..4 {
                let at = |dx: u32, dy: u32| texels[(((y * 2 + dy) * side + x * 2 + dx) * 4 + c) as usize] as u32;
                out.push(((at(0, 0) + at(1, 0) + at(0, 1) + at(1, 1) + 2) / 4) as u8);
            }
        }
    }
    out
}

/// The files the atlas is made from, below `third_party`.
pub const SMALL_FACE: &str = "fonts/tiny5/Tiny5.ttf";
pub const LARGE_FACE: &str = "fonts/imfellenglish/IMFeENrm28P.ttf";

/// Bakes the atlas, or returns the cached bytes if the fonts and the bake are unchanged.
pub fn bake_atlas(third_party: &Path, cache: Option<&Path>) -> Result<Vec<u8>, BakeError> {
    let read = |rel: &str| {
        std::fs::read(third_party.join(rel)).map_err(|e| BakeError(format!("third_party/{rel}: {e}")))
    };
    let (small, large) = (read(SMALL_FACE)?, read(LARGE_FACE)?);

    let mut key_input = Vec::new();
    key_input.extend_from_slice(VERSION.as_bytes());
    key_input.extend_from_slice(&sha256(&small));
    key_input.extend_from_slice(&sha256(&large));
    let cache_file = cache.map(|dir| dir.join(format!("atlas-{}.bin", &hex(&sha256(&key_input))[..16])));
    if let Some(file) = &cache_file {
        if let Ok(bytes) = std::fs::read(file) {
            if GlyphAtlas::from_bytes(&bytes).is_ok() {
                return Ok(bytes);
            }
        }
    }

    let faces = [bake_face(&small, SMALL_FACE, Method::Raster)?, bake_face(&large, LARGE_FACE, Method::Edges)?];
    let mut texels = Vec::new();
    let mut levels: Vec<Vec<Vec<u8>>> = faces.iter().map(|f| f.glyphs.clone()).collect();
    for level in 0..MIPS {
        for face in &levels {
            for glyph in face {
                texels.extend_from_slice(glyph);
            }
        }
        let side = CELL >> level;
        for face in &mut levels {
            for glyph in face.iter_mut() {
                *glyph = halve(glyph, side);
            }
        }
    }
    let atlas = GlyphAtlas {
        cell: CELL,
        mips: MIPS,
        range: RANGE as f32,
        faces: faces.into_iter().map(|f| f.info).collect(),
        texels,
    };
    let bytes = atlas.to_bytes();
    if let Some(file) = &cache_file {
        if let Some(dir) = file.parent() {
            let _ = std::fs::create_dir_all(dir);
        }
        let _ = std::fs::write(file, &bytes);
    }
    Ok(bytes)
}

/// Draws every glyph of the atlas from its distance field, as a sheet to look at.
/// `px` is the side of each drawn cell. Returns RGBA8 and the sheet's size.
pub fn preview_sheet(atlas: &GlyphAtlas, px: u32, threshold: f32) -> (Vec<u8>, u32, u32) {
    let columns = 19u32;
    let rows = (atlas.layers()).div_ceil(columns);
    let (w, h) = (columns * px, rows * px);
    let mut out = vec![255u8; (w * h * 4) as usize];
    let level0 = &atlas.texels[atlas.level_range(0)];
    let cell = atlas.cell as usize;
    let sample = |layer: usize, u: f32, v: f32| -> [f32; 4] {
        let x = (u * cell as f32 - 0.5).clamp(0.0, cell as f32 - 1.0);
        let y = (v * cell as f32 - 0.5).clamp(0.0, cell as f32 - 1.0);
        let (x0, y0) = (x.floor() as usize, y.floor() as usize);
        let (x1, y1) = ((x0 + 1).min(cell - 1), (y0 + 1).min(cell - 1));
        let (fx, fy) = (x - x0 as f32, y - y0 as f32);
        let at = |xx: usize, yy: usize, c: usize| level0[layer * cell * cell * 4 + (yy * cell + xx) * 4 + c] as f32 / 255.0;
        let mut r = [0.0; 4];
        for (c, slot) in r.iter_mut().enumerate() {
            *slot = (at(x0, y0, c) * (1.0 - fx) + at(x1, y0, c) * fx) * (1.0 - fy)
                + (at(x0, y1, c) * (1.0 - fx) + at(x1, y1, c) * fx) * fy;
        }
        r
    };
    let texels_per_px = cell as f32 / px as f32;
    for layer in 0..atlas.layers() as usize {
        let (cx, cy) = (layer as u32 % columns, layer as u32 / columns);
        for y in 0..px {
            for x in 0..px {
                let s = sample(layer, (x as f32 + 0.5) / px as f32, (y as f32 + 0.5) / px as f32);
                let median = s[0].max(s[1]).min(s[0].min(s[1]).max(s[2]));
                let d = if texels_per_px > 2.0 { s[3] } else { median };
                let cover = ((d - threshold) * atlas.range / texels_per_px + 0.5).clamp(0.0, 1.0);
                let v = (255.0 * (1.0 - cover)) as u8;
                let at = (((cy * px + y) * w + cx * px + x) * 4) as usize;
                out[at..at + 4].copy_from_slice(&[v, v, v, 255]);
            }
        }
    }
    (out, w, h)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn coverage_lookup_is_monotonic_and_reaches_its_targets() {
        // A disc of radius 40 texels in a 128 cell, as a true distance field.
        let mut texels = vec![0u8; (CELL * CELL * 4) as usize];
        for y in 0..CELL {
            for x in 0..CELL {
                let d = 40.0 - ((x as f32 - 63.5).powi(2) + (y as f32 - 63.5).powi(2)).sqrt();
                let v = ((d / RANGE as f32 + 0.5).clamp(0.0, 1.0) * 255.0) as u8;
                texels[((y * CELL + x) * 4 + 3) as usize] = v;
            }
        }
        let info = glyph_info(0.5, 0.25, &texels);
        assert!(info.min_cover < 0.26 && info.max_cover > 0.36, "{info:?}");
        for pair in info.tone.windows(2) {
            assert!(pair[1] <= pair[0] + 1e-4, "heavier targets need lower thresholds");
        }
        let alpha: Vec<u8> = texels.chunks_exact(4).map(|t| t[3]).collect();
        let target = 0.30f32;
        let i = target * (TONE_STEPS - 1) as f32;
        let t = info.tone[i.floor() as usize] + (info.tone[i.ceil() as usize] - info.tone[i.floor() as usize]) * i.fract();
        assert!((coverage_at(&alpha, t) - target).abs() < 0.01);
    }

    #[test]
    fn halving_averages() {
        let texels: Vec<u8> = [10u8, 20, 30, 40].iter().flat_map(|&v| [v, v, v, v]).collect();
        assert_eq!(halve(&texels, 2), vec![25, 25, 25, 25]);
    }
}
