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

struct PageEntry {
    // Left column, top row, width, height, in cells of the page's level.
    place: vec4<i32>,
    // xy: where the page starts on the sheet. z: its level.
    sheet_level: vec4<u32>,
}

struct PageTable {
    entries: array<PageEntry, 64>,
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

fn hash_cell(cell: vec2<i32>, level: i32, material: u32) -> u32 {
    var h = u32(cell.x) * 0x9e3779b1u;
    h = (h ^ (h >> 15u)) * 0x85ebca77u;
    h ^= u32(cell.y) * 0xc2b2ae3du;
    h = (h ^ (h >> 13u)) * 0x27d4eb2fu;
    h ^= (u32(level + 16) * 0x165667b1u) ^ (material * 0x9e3779b9u);
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

// The authored character of a cell, or 0 if the surface has no authored text there.
fn authored_character(cell: vec2<i32>, level: i32, page: u32) -> u32 {
    if page == 0u {
        return 0u;
    }
    let entry = pages.entries[page - 1u];
    if i32(entry.sheet_level.z) != level {
        return 0u;
    }
    let column = cell.x - entry.place.x;
    let row = entry.place.y - cell.y;
    if column < 0 || row < 0 || column >= entry.place.z || row >= entry.place.w {
        return 0u;
    }
    return textureLoad(page_sheet, vec2<i32>(entry.sheet_level.xy) + vec2<i32>(column, row), 0).r;
}

// The character a cell shows. Authored text comes first: if the surface has a page at
// this level and the page has a character for this cell, that is the character. A
// space on a page is a blank cell. Elsewhere the surface shows its material's grain.
// Either way the answer depends only on where the cell is.
fn cell_character(cell: vec2<i32>, level: i32, material: u32, page: u32) -> u32 {
    if page != 0u {
        let entry = pages.entries[page - 1u];
        if i32(entry.sheet_level.z) == level {
            let column = cell.x - entry.place.x;
            let row = entry.place.y - cell.y;
            if column >= 0 && row >= 0 && column < entry.place.z && row < entry.place.w {
                let c = textureLoad(page_sheet, vec2<i32>(entry.sheet_level.xy) + vec2<i32>(column, row), 0).r;
                if c != 0u {
                    return c;
                }
            }
        }
    }
    return family_character(material, hash_cell(cell, level, material));
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

// The highest level drawn in normal play, in close reading, and the lowest in a dive.
const TOP_LEVEL: i32 = 1;
const TOP_LEVEL_READING: i32 = 5;
const BOTTOM_LEVEL_DIVE: i32 = -4;

// `light` is how brightly the surface is lit, 0 to 1. It is used only in close
// reading, where light is kept as the weight of the ink and nothing else.
fn glyph_surface(anchor: vec2<f32>, footprint: AnchorFootprint, material: u32, page: u32, albedo: vec3<f32>, light: f32) -> GlyphResult {
    var result = GlyphResult(1.0, vec2<f32>(0.0), 0.0, 0.0);
    let m = materials.entries[material];

    // Metres of page to a pixel, along the direction in which the page is most
    // squeezed. Dividing a cell's side by this gives the cell's smaller size on screen,
    // so a surface seen at a grazing angle loses its letters before they can alias.
    let squeeze = max(length(footprint.dx), length(footprint.dy));
    if squeeze <= 0.0 {
        return result;
    }
    let reading = view.switches.y;
    let dive = view.effects.x;
    let h0 = CELL_0 / squeeze * view.switches.w;

    // Which levels can show. A level shows while its cells are between about 3 and 160
    // pixels tall; each level is four times the last.
    let top_allowed = select(TOP_LEVEL, TOP_LEVEL_READING, reading > 0.0);
    let bottom_allowed = select(0, BOTTOM_LEVEL_DIVE, dive > 0.0);
    let top = min(top_allowed, i32(floor(log2(160.0 / h0) * 0.5)));
    let bottom = max(bottom_allowed, i32(ceil(log2(3.0 / h0) * 0.5)));

    // The early exit. Beyond the reveal distance no level is large enough to show:
    // nothing is sampled and nothing changes.
    if max(top, bottom_allowed) < bottom {
        return result;
    }
    // Level 0 never fades as it grows, so in normal play it is drawn however large.
    let first = max(top, bottom_allowed);
    let last = max(bottom, first - 3);

    // The surface's own tone decides how heavy its letters are: darker patches are
    // written in heavier strokes. Only the low frequencies of the colour are used, so
    // this is a property of the place, never of the light.
    let luminance = dot(albedo, vec3<f32>(0.2126, 0.7152, 0.0722));
    let own_cover = clamp(0.30 - (luminance - 0.18) * 0.5, 0.16, 0.42);
    // In close reading the light itself becomes ink: shadow is written heavy.
    let lit_cover = clamp(0.48 - light * 0.30, 0.14, 0.48);
    let base_cover = mix(own_cover, lit_cover, reading);
    let contrast = m.surface.y * view.switches.z;

    var modulation = 0.0;
    var ink = 0.0;
    var relief = vec2<f32>(0.0);
    // What the level above drew, for letters made of letters.
    var parent_layer = 0u;
    var parent_threshold = 0.5;
    var parent_mean = 0.0;
    var parent_cell = vec2<i32>(0);
    var parent_built = 0.0;

    for (var level = first; level >= last; level--) {
        let size = CELL_0 * pow(4.0, f32(level));
        let h = h0 * pow(4.0, f32(level));

        // How strongly this level shows. The fine print of normal play is levels 0 and
        // 1. Coarser levels belong to close reading, finer ones to a dive.
        var w = influence_rise(h);
        if level >= 2 {
            w *= influence_fall(h) * reading;
        } else if level == 1 {
            w *= influence_fall(h);
        } else if level == 0 {
            w *= mix(1.0, influence_fall(h), dive);
        } else {
            w *= influence_fall(h) * dive;
        }

        let p = anchor / size;
        let cell = vec2<i32>(floor(p));
        let f = fract(p);
        let uv = vec2<f32>(f.x, 1.0 - f.y);
        let du = footprint.dx * vec2<f32>(1.0 / size, -1.0 / size);
        let dv = footprint.dy * vec2<f32>(1.0 / size, -1.0 / size);

        // Letters made of letters. The parent's coverage at the middle of this cell
        // decides how heavy this cell's letter is, so the larger character is built
        // from the weight of the smaller ones inside its strokes.
        var wanted = base_cover;
        if parent_built > 0.0 {
            let centre = (vec2<f32>(cell) + 0.5) / 4.0 - vec2<f32>(parent_cell);
            let parent_uv = vec2<f32>(centre.x, 1.0 - centre.y);
            let distance = textureSampleLevel(glyph_atlas, glyph_sampler, parent_uv, parent_layer, 2.0).a;
            let parent_cover = clamp((distance - parent_threshold) * 4.0 + 0.5, 0.0, 1.0);
            wanted += parent_built * 0.40 * (parent_cover - parent_mean) * base_cover / max(parent_mean, 0.05);
        }

        let character = cell_character(cell, level, material, page);
        let layer = character - 32u;
        let g = cell_glyph(layer, uv, du, dv, h, wanted);
        modulation += w * nearness(h) * (g.mean - g.cover);
        ink = mix(ink, g.cover, w);

        // Relief, once a letter is large enough to catch the light: the slope of the
        // glyph's edge tilts the normal, cutting letters into hard materials and
        // raising them on soft ones. Only the finest level drawn is carved.
        let carve = smoothstep(10.0, 24.0, h) * w;
        if level == last && carve > 0.0 {
            let carved = layer + select(0u, GLYPHS_PER_FACE, h > 10.75);
            let step = 1.5 / ATLAS_CELL;
            let here = textureSampleLevel(glyph_atlas, glyph_sampler, uv, carved, 0.0).a;
            let right = textureSampleLevel(glyph_atlas, glyph_sampler, uv + vec2<f32>(step, 0.0), carved, 0.0).a;
            let below = textureSampleLevel(glyph_atlas, glyph_sampler, uv + vec2<f32>(0.0, step), carved, 0.0).a;
            // Only the bevel at the glyph's edge slopes; the face and the ground are flat.
            let bevel = 1.0 - smoothstep(0.0, 0.14, abs(here - g.threshold));
            // v runs down the page and the tangent frame's second axis runs up it.
            let slope = vec2<f32>(right - here, here - below) * (ATLAS_RANGE / 1.5);
            relief = -slope * bevel * m.surface.z * 0.55 * carve;
        }

        parent_layer = layer + select(0u, GLYPHS_PER_FACE, h > 10.75);
        parent_threshold = g.threshold;
        parent_mean = g.mean;
        parent_cell = cell;
        parent_built = influence_rise(h);
    }

    result.tone = max(1.0 + contrast * modulation, 0.05);
    result.relief = relief;
    result.ink = ink;
    result.revealed = 1.0;
    return result;
}

// ---- depth: looking into what a thing is made of ----
//
// Up close, a surface is not a flat page. The realistic texture thins, dithers, and
// goes, and what is left is what the thing was made of all along: letters, standing
// in the dark like atoms, each one lit and coloured as the thing was at that spot.
// They do not stop at the surface. They fill the thing, layer behind layer, as deep
// as the thing is thick, so a wall is seen into as far as its other side. Each layer
// is a plane parallel to the surface, and the letter seen at a pixel is the one that
// pixel's own view ray strikes on that plane: the layers slide against each other as
// the head moves, exactly as real things at those depths would.
//
// All of it fades in with nearness and is nothing at all beyond the reveal distance.
//
// Look 1, atoms: the texture goes entirely, and the letters tremble where they stand.
// Look 2, lattice: the texture stays as a half-dithered film over a still lattice.
// Look 3, source: under the first layer is the code beneath the world: every
//   character there is, in gold, falling.

// The lattice's pitch: a line of characters every this many cells, on all three axes.
const LATTICE: i32 = 6;
// The most planes of each family a ray is followed through.
const LATTICE_STEPS: i32 = 7;

// A 4 by 4 ordered dither threshold, 0 to 1, for a cell of the dither grid.
fn bayer(cell: vec2<i32>) -> f32 {
    let x = u32(cell.x) & 3u;
    let y = u32(cell.y) & 3u;
    let table = array<u32, 16>(0u, 8u, 2u, 10u, 12u, 4u, 14u, 6u, 3u, 11u, 1u, 9u, 15u, 7u, 13u, 5u);
    return (f32(table[y * 4u + x]) + 0.5) / 16.0;
}

fn unit(h: u32) -> f32 {
    return f32(h & 0xffffu) / 65535.0;
}

fn on_line(c: i32) -> bool {
    return (c % LATTICE + LATTICE) % LATTICE == 0;
}

// One character of the lattice, on some plane inside the thing. `q` is the place on
// that plane in cells, `dqx` and `dqy` how far that moves across one pixel. Returns
// how much of the pixel the character covers.
fn lattice_glyph(q: vec2<f32>, dqx: vec2<f32>, dqy: vec2<f32>, salt: i32, material: u32, every: bool, tremble: f32) -> f32 {
    let cell = vec2<i32>(floor(q));
    let pick = hash_cell(cell, salt, material);
    var character = family_character(material, pick);
    if every {
        // The source is written in every printable character but the space.
        character = 33u + pick % 94u;
    }
    var f = fract(q);
    if tremble > 0.0 {
        let phase = unit(pick >> 8u) * 6.283;
        let t = view.eye_time.w;
        f += vec2<f32>(sin(t * 4.1 + phase), cos(t * 3.3 + phase * 1.7)) * tremble;
    }
    let uv = clamp(vec2<f32>(f.x, 1.0 - f.y), vec2<f32>(0.0), vec2<f32>(1.0));
    // The cell's smaller size on screen, as for a surface's own letters.
    let h = view.switches.w / max(max(length(dqx), length(dqy)), 1.0e-6);
    if h < 2.5 {
        return 0.0;
    }
    let g = cell_glyph(character - 32u, uv, vec2<f32>(dqx.x, -dqx.y), vec2<f32>(dqy.x, -dqy.y), h, 0.34);
    return g.cover * smoothstep(2.5, 5.0, h);
}

// `lit` is the surface as shaded. `to_eye` is the direction to the eye in the
// surface's own frame: along the page, up the page, and out of it. `distance` is how
// far the eye is. `thickness` is how deep the thing is behind this surface. `ink` is
// how much of this pixel is a stroke of the surface's own fine print.
fn glyph_depth(lit: vec3<f32>, anchor: vec2<f32>, footprint: AnchorFootprint, to_eye: vec3<f32>, distance: f32, material: u32, page: u32, thickness: f32, ink: f32, look: u32) -> vec3<f32> {
    let squeeze = max(length(footprint.dx), length(footprint.dy));
    if squeeze <= 0.0 {
        return lit;
    }
    // The lattice is ruled in cells of 1.28 cm. Its characters are no larger than the
    // fine print of the surface: what changes up close is that the eye can see past
    // the first of them.
    let size = CELL_0 * 2.0;
    let h = size / squeeze * view.switches.w;
    // Nothing beyond about two metres, everything within about sixty centimetres.
    let near = smoothstep(log2(6.0), log2(21.0), log2(max(h, 1.0e-3)));
    if near <= 0.0 {
        return lit;
    }
    let source = look == 3u;

    // The realistic texture is taken away by an ordered dither, a few dots, then most.
    var gone = near;
    if look == 2u {
        gone = near * 0.6;
    }
    let dots = anchor / (size / 4.0);
    let dot_pixels = h / 4.0;
    let hole = step(bayer(vec2<i32>(floor(dots))), gone);
    // Dots too small to draw cleanly are averaged: the surface thins before it breaks.
    let crisp = smoothstep(1.2, 2.6, dot_pixels);
    let film = mix(1.0 - gone, 1.0 - hole, crisp);
    // The surface's scattered fine print goes with the texture; the lattice's own
    // first layer, at the surface, takes its place.
    var keep = max(film, ink * (1.0 - near));

    // Written words come first. Where the surface carries a sentence, its letters
    // stand whole at the front, as large and as plain as they were on the solid wall,
    // and the lattice behind them is hushed so that the sentence can be read through
    // everything else.
    var hush = 1.0;
    var sentence = 0.0;
    var on_text = false;
    for (var level = 0; level <= 1; level++) {
        let text_size = CELL_0 * pow(4.0, f32(level));
        let tp = anchor / text_size;
        let text_cell = vec2<i32>(floor(tp));
        let written = authored_character(text_cell, level, page);
        if written != 0u {
            hush = 0.18;
            on_text = true;
            if written != 32u {
                let tf = fract(tp);
                let scale = vec2<f32>(1.0 / text_size, -1.0 / text_size);
                let letter = cell_glyph(written - 32u, vec2<f32>(tf.x, 1.0 - tf.y), footprint.dx * scale, footprint.dy * scale, text_size / squeeze * view.switches.w, 0.40);
                sentence = max(sentence, letter.cover);
            }
        }
    }
    if on_text {
        // Nothing else is left in a written cell: no dots, no grain. Only the letter.
        keep = max(film * (1.0 - near), sentence);
    }
    if keep >= 0.999 {
        return lit;
    }

    // The lattice. Inside the thing is a cubic grid, as deep as the thing is thick.
    // No line of it is drawn. Along each of its edges, in all three directions, stands
    // a row of characters, and between the edges is nothing but the dark; the eye
    // joins the rows into a frame running away to a vanishing point. The ray of this
    // pixel is followed through the planes of the grid, and on each plane it lands on
    // a character or on nothing.
    let gold = vec3<f32>(1.0, 0.60, 0.15) * 0.9 / view.viewport.z;
    let steep = max(to_eye.z, 0.15);
    // How far the ray moves across the page for each metre it goes in.
    let slide = to_eye.xy / steep;
    let reach = clamp(thickness, 0.08, 1.5);
    let pitch = size * f32(LATTICE);
    let tremble = select(0.0, 0.03, look == 1u);
    // The source streams along its edges, into the dark.
    let stream = select(0.0, view.eye_time.w * 2.2, source);
    var seen = vec3<f32>(0.0);
    var cover_so_far = 0.0;
    let d_page_x = footprint.dx / size;
    let d_page_y = footprint.dy / size;

    // Planes parallel to the surface, one pitch apart: the edges that run across the
    // page and up it.
    for (var k = 0; k <= LATTICE_STEPS; k++) {
        let depth = pitch * f32(k);
        if depth > reach {
            break;
        }
        let q = (anchor - slide * depth) / size;
        let cell = vec2<i32>(floor(q));
        let across = on_line(cell.y);
        let up = on_line(cell.x);
        if !(across || up) {
            continue;
        }
        // Further planes are further from the eye, and draw smaller.
        let shrink = (distance + depth / steep) / distance;
        var moved = q;
        if across {
            moved.x += stream;
        } else {
            moved.y += stream;
        }
        let c = lattice_glyph(moved, d_page_x * shrink, d_page_y * shrink, 20 + k, material, source, tremble);
        let dim = exp(-2.3 * depth / reach);
        seen += (1.0 - cover_so_far) * c * dim;
        cover_so_far += (1.0 - cover_so_far) * c;
    }

    // Planes that stand on the page and run into it: the edges that run away from the
    // eye. On each, the ray's place is (depth, the other page coordinate).
    for (var axis = 0; axis < 2; axis++) {
        let along = select(slide.y, slide.x, axis == 0);
        if abs(along) < 0.03 {
            // Looking straight down these planes: their edges are end on, a point each.
            continue;
        }
        let here = select(anchor.y, anchor.x, axis == 0);
        let other = select(anchor.x, anchor.y, axis == 0);
        let other_slide = select(slide.x, slide.y, axis == 0);
        let d_here = vec2<f32>(select(footprint.dx.y, footprint.dx.x, axis == 0), select(footprint.dy.y, footprint.dy.x, axis == 0));
        let d_other = vec2<f32>(select(footprint.dx.x, footprint.dx.y, axis == 0), select(footprint.dy.x, footprint.dy.y, axis == 0));
        // The ray moves towards smaller coordinates when it slides the positive way.
        let direction = -sign(along);
        var plane = floor(here / pitch);
        if direction > 0.0 {
            plane += 1.0;
        }
        for (var k = 0; k < LATTICE_STEPS; k++) {
            let depth = (here - plane * pitch) / along;
            plane += direction;
            if depth <= 0.0 {
                continue;
            }
            if depth > reach {
                break;
            }
            let place = other - other_slide * depth;
            let q = vec2<f32>(depth, place) / size;
            if !on_line(i32(floor(q.y))) {
                continue;
            }
            // How this plane's coordinates move across one pixel.
            let d_depth = d_here / along;
            let d_place = d_other - other_slide * d_depth;
            let dqx = vec2<f32>(d_depth.x, d_place.x) / size;
            let dqy = vec2<f32>(d_depth.y, d_place.y) / size;
            let c = lattice_glyph(q + vec2<f32>(stream, 0.0), dqx, dqy, 60 + axis, material, source, tremble);
            let dim = exp(-2.3 * depth / reach);
            seen += (1.0 - cover_so_far) * c * dim;
            cover_so_far += (1.0 - cover_so_far) * c;
        }
    }

    var behind = lit * 1.15 * seen;
    if source {
        behind = gold * seen;
    }
    let through = 1.0 - cover_so_far;
    // Whatever no letter covers is the dark the letters hang in.
    var dark = lit * 0.012;
    if source {
        dark = vec3<f32>(0.004, 0.0025, 0.0008) / view.viewport.z;
    }
    behind += through * dark;
    return mix(behind * hush, lit, keep);
}
