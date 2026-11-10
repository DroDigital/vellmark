// The Glyph Cascade. Every opaque surface calls glyph_surface().
//
// Letters belong to surfaces. A surface's page is its anchor coordinate, in metres;
// the page is ruled into square cells at each level of the cascade, and each cell
// holds one character. Which character is decided only by where the cell is. How
// heavy it is drawn follows the surface's own tone. How strongly it shows follows how
// large the cell is on screen, and below about three pixels it does not show at all.
//
// The letters must be invisible in aggregate: ink is darker than the surface and paper
// is lighter, in the proportion that makes a cell's average exactly the surface's own
// colour.

struct MaterialParams {
    // x: tile size in metres. y: letter contrast. z: relief. w: emissive.
    surface: vec4<f32>,
    // x: number of characters in the family.
    info: vec4<u32>,
    // The family's characters, as code points.
    family: array<vec4<u32>, 2>,
}

struct MaterialTable {
    entries: array<MaterialParams, 32>,
}

struct AnchorFootprint {
    // How far the anchor moves across one pixel, right and down, in metres.
    dx: vec2<f32>,
    dy: vec2<f32>,
}

struct GlyphResult {
    // Multiplies the surface's colour. Averages to 1 over any cell.
    tone: f32,
    // Added to the tangent-space normal: letters cut in or raised.
    relief: vec2<f32>,
    // How much of this pixel is ink, 0 to 1, for close reading.
    ink: f32,
    // 1 where the letters were drawn at all, 0 beyond the reveal distance.
    revealed: f32,
}

const CELL_0: f32 = 0.0064;
const GLYPHS_PER_FACE: u32 = 95u;
const ATLAS_CELL: f32 = 128.0;
const ATLAS_RANGE: f32 = 16.0;
const TONE_STEPS: f32 = 16.0;

fn hash_cell(cell: vec2<i32>, level: u32, material: u32) -> u32 {
    var h = u32(cell.x) * 0x9e3779b1u;
    h = (h ^ (h >> 15u)) * 0x85ebca77u;
    h ^= u32(cell.y) * 0xc2b2ae3du;
    h = (h ^ (h >> 13u)) * 0x27d4eb2fu;
    h ^= (level * 0x165667b1u) ^ (material * 0x9e3779b9u);
    h = (h ^ (h >> 16u)) * 0x7feb352du;
    return h ^ (h >> 15u);
}

// How strongly a level shows, from the height of its cells on screen in pixels.
// Smooth in the logarithm of the height, so that walking closer never pops.
fn influence_rise(h: f32) -> f32 {
    return smoothstep(log2(3.0), log2(8.0), log2(max(h, 1.0e-3)));
}

// Letters are faint where they first appear, so that no grain shows at the reveal
// distance, and firm at arm's length, where they are read.
fn nearness(h: f32) -> f32 {
    return mix(0.7, 1.7, smoothstep(log2(8.0), log2(36.0), log2(max(h, 1.0e-3))));
}

fn influence_fall(h: f32) -> f32 {
    return 1.0 - smoothstep(log2(40.0), log2(160.0), log2(max(h, 1.0e-3)));
}

struct CellGlyph {
    // Coverage at this pixel, 0 to 1, antialiased to the pixel's footprint.
    cover: f32,
    // The mean of `cover` over the whole cell.
    mean: f32,
    // The true distance at this pixel, 0 to 1, with 0.5 at the glyph's edge as designed.
    distance: f32,
    threshold: f32,
}

// Reads one row of the per-glyph table: the threshold that gives a target coverage,
// what coverage the glyph can reach, and its mean distance.
fn glyph_threshold(layer: u32, wanted: f32) -> vec3<f32> {
    let row = i32(layer);
    let low = textureLoad(glyph_tone, vec2<i32>(16, row), 0).r;
    let high = textureLoad(glyph_tone, vec2<i32>(17, row), 0).r;
    let reached = clamp(wanted, low, high);
    let x = reached * (TONE_STEPS - 1.0);
    let i = i32(floor(x));
    let a = textureLoad(glyph_tone, vec2<i32>(min(i, 15), row), 0).r;
    let b = textureLoad(glyph_tone, vec2<i32>(min(i + 1, 15), row), 0).r;
    let mean_distance = textureLoad(glyph_tone, vec2<i32>(18, row), 0).r;
    return vec3<f32>(mix(a, b, fract(x)), reached, mean_distance);
}

// The character a cell of a material's grain shows.
fn family_character(material: u32, pick: u32) -> u32 {
    let m = materials.entries[material];
    let i = pick % max(m.info.x, 1u);
    return m.family[i / 4u][i % 4u];
}

// Draws the glyph of one cell. `uv` is the place within the cell, with v down the
// page. `du` and `dv` are how far uv moves across one pixel. `h` is the cell's height
// on screen in pixels. `wanted` is the share of the cell the glyph should cover.
fn cell_glyph(layer_small: u32, uv: vec2<f32>, du: vec2<f32>, dv: vec2<f32>, h: f32, wanted: f32) -> CellGlyph {
    // The pixel face while a cell is small, the printed face once it has room. The
    // change is made over a narrow band: two faces drawn half-and-half read as a blur.
    let large = smoothstep(9.0, 12.5, h);
    var out = CellGlyph(0.0, 0.0, 0.0, 0.5);
    // Screen pixels to an atlas texel, and so how steep the edge's ramp is.
    let k = ATLAS_RANGE * h / ATLAS_CELL;
    // Below this the ramp never saturates and coverage is linear in distance.
    let sharp = smoothstep(1.0, 4.0, k);

    if large < 1.0 {
        let texel = textureSampleGrad(glyph_atlas, glyph_sampler, uv, layer_small, du, dv);
        let t = glyph_threshold(layer_small, wanted);
        let cover = clamp((texel.a - t.x) * k + 0.5, 0.0, 1.0);
        let mean = mix(clamp(0.5 + (t.z - t.x) * k, 0.0, 1.0), t.y, sharp);
        out.cover += cover * (1.0 - large);
        out.mean += mean * (1.0 - large);
        out.distance += texel.a * (1.0 - large);
        out.threshold = t.x;
    }
    if large > 0.0 {
        let layer = layer_small + GLYPHS_PER_FACE;
        let texel = textureSampleGrad(glyph_atlas, glyph_sampler, uv, layer, du, dv);
        let t = glyph_threshold(layer, wanted);
        let median = max(min(texel.r, texel.g), min(max(texel.r, texel.g), texel.b));
        // The multi-channel distance keeps corners sharp once the glyph is large.
        let d = mix(texel.a, median, smoothstep(24.0, 48.0, h));
        let cover = clamp((d - t.x) * k + 0.5, 0.0, 1.0);
        let mean = mix(clamp(0.5 + (t.z - t.x) * k, 0.0, 1.0), t.y, sharp);
        out.cover += cover * large;
        out.mean += mean * large;
        out.distance += texel.a * large;
        out.threshold = mix(out.threshold, t.x, large);
    }
    return out;
}

fn glyph_surface(anchor: vec2<f32>, footprint: AnchorFootprint, material: u32, albedo: vec3<f32>) -> GlyphResult {
    var result = GlyphResult(1.0, vec2<f32>(0.0), 0.0, 0.0);
    let m = materials.entries[material];

    // Metres of page to a pixel, along the direction in which the page is most
    // squeezed. Dividing a cell's side by this gives the cell's smaller size on screen,
    // so a surface seen at a grazing angle loses its letters before they can alias.
    let squeeze = max(length(footprint.dx), length(footprint.dy));
    if squeeze <= 0.0 {
        return result;
    }
    let reveal = view.switches.w;
    let h0 = CELL_0 / squeeze * reveal;
    let h1 = h0 * 4.0;

    // The early exit. When even the larger fine level is under three pixels, nothing
    // is sampled and nothing changes.
    if h1 < 3.0 {
        return result;
    }

    // The surface's own tone decides how heavy its letters are: darker patches are
    // written in heavier strokes. Only the low frequencies of the colour are used, so
    // this is a property of the place, never of the light.
    let luminance = dot(albedo, vec3<f32>(0.2126, 0.7152, 0.0722));
    let base_cover = clamp(0.30 - (luminance - 0.18) * 0.5, 0.16, 0.42);
    let contrast = m.surface.y * view.switches.z;

    // Level 1: 2.56 cm cells.
    let w1 = influence_rise(h1) * influence_fall(h1);
    let p1 = anchor / (CELL_0 * 4.0);
    let cell1 = vec2<i32>(floor(p1));
    let f1 = fract(p1);
    let uv1 = vec2<f32>(f1.x, 1.0 - f1.y);
    let scale1 = 1.0 / (CELL_0 * 4.0);
    let char1 = family_character(material, hash_cell(cell1, 1u, material));
    let layer1 = char1 - 32u;
    let g1 = cell_glyph(layer1, uv1, footprint.dx * vec2<f32>(scale1, -scale1), footprint.dy * vec2<f32>(scale1, -scale1), h1, base_cover);

    var modulation = w1 * nearness(h1) * (g1.mean - g1.cover);
    var relief = vec2<f32>(0.0);
    var ink = w1 * g1.cover;

    // Level 0: 0.64 cm cells, the finest print. It never fades as it grows.
    let w0 = influence_rise(h0);
    if w0 > 0.0 {
        let p0 = anchor / CELL_0;
        let cell0 = vec2<i32>(floor(p0));
        let f0 = fract(p0);
        let uv0 = vec2<f32>(f0.x, 1.0 - f0.y);
        let scale0 = 1.0 / CELL_0;

        // Letters made of letters. The parent's coverage at the middle of this cell
        // decides how heavy this cell's letter is, so the larger character is built
        // from the weight of the smaller ones inside its strokes.
        let centre_in_parent = (vec2<f32>(cell0) + 0.5) / 4.0 - vec2<f32>(cell1);
        let parent_uv = vec2<f32>(centre_in_parent.x, 1.0 - centre_in_parent.y);
        let parent = textureSampleLevel(glyph_atlas, glyph_sampler, parent_uv, layer1, 2.0).a;
        let parent_cover = clamp((parent - g1.threshold) * 4.0 + 0.5, 0.0, 1.0);
        let built = influence_rise(h1);
        let target0 = base_cover + built * 0.40 * (parent_cover - g1.mean) * base_cover / max(g1.mean, 0.05);

        let char0 = family_character(material, hash_cell(cell0, 0u, material));
        let layer0 = char0 - 32u;
        let du = footprint.dx * vec2<f32>(scale0, -scale0);
        let dv = footprint.dy * vec2<f32>(scale0, -scale0);
        let g0 = cell_glyph(layer0, uv0, du, dv, h0, target0);
        modulation += w0 * nearness(h0) * (g0.mean - g0.cover);
        ink = mix(ink, g0.cover, w0);

        // Relief, once a letter is large enough to catch the light: the slope of the
        // glyph's edge tilts the normal, cutting letters into hard materials and
        // raising them on soft ones.
        let carve = smoothstep(10.0, 24.0, h0) * w0;
        if carve > 0.0 {
            let layer = layer0 + select(0u, GLYPHS_PER_FACE, h0 > 10.75);
            let step = 1.5 / ATLAS_CELL;
            let here = textureSampleLevel(glyph_atlas, glyph_sampler, uv0, layer, 0.0).a;
            let right = textureSampleLevel(glyph_atlas, glyph_sampler, uv0 + vec2<f32>(step, 0.0), layer, 0.0).a;
            let below = textureSampleLevel(glyph_atlas, glyph_sampler, uv0 + vec2<f32>(0.0, step), layer, 0.0).a;
            // Only the bevel at the glyph's edge slopes; the face and the ground are flat.
            let bevel = 1.0 - smoothstep(0.0, 0.14, abs(here - g0.threshold));
            // v runs down the page and the tangent frame's second axis runs up it.
            let slope = vec2<f32>(right - here, here - below) * (ATLAS_RANGE / 1.5);
            relief = -slope * bevel * m.surface.z * 0.55 * carve;
        }
    }

    result.tone = max(1.0 + contrast * modulation, 0.05);
    result.relief = relief;
    result.ink = ink;
    result.revealed = 1.0;
    return result;
}
