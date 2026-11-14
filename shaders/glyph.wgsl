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
    // x: number of characters in the family. y, z, w: how many element, hot, and cold
    // words its lexicon has.
    info: vec4<u32>,
    // The family's characters, as code points.
    family: array<vec4<u32>, 2>,
    // The lexicon: its name, three elements, three hot words, two cold words, each up
    // to sixteen characters, four to a component.
    words: array<vec4<u32>, 9>,
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
// Where a rewrite wave is passing, 0 to 1: the surface's coarser print shows there, as
// it does in close reading, so that the rewriting can be seen from where the word was
// written. Set by the caller before `glyph_surface`; 0 everywhere else.
var<private> wave_reveal: f32 = 0.0;

fn glyph_surface(anchor: vec2<f32>, footprint: AnchorFootprint, material: u32, page: u32, albedo: vec3<f32>, light: f32, heat: f32) -> GlyphResult {
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
    let coarse = max(reading, wave_reveal);
    let top_allowed = select(TOP_LEVEL, TOP_LEVEL_READING, coarse > 0.0);
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

    // While a thing is hot or cold enough, its own fine print says so: the word for
    // what is happening to it, over and over, in place of its grain. A change of state
    // is a change of the underlying text, as for a creature's body language.
    var state = -1;
    if abs(heat) > 0.3 {
        state = state_slot(material, heat);
    }
    var state_count = 0;
    if state >= 0 {
        state_count = i32(word_length(material, u32(state)));
    }

    var modulation = 0.0;
    var ink = 0.0;
    var relief = vec2<f32>(0.0);
    // What the level above drew, for letters made of letters.
    var parent_layer = 0u;
    var parent_threshold = 0.5;
    var parent_mean = 0.0;
    var parent_cell = vec2<i32>(0);
    var parent_built = 0.0;
    var parent_written = false;

    for (var level = first; level >= last; level--) {
        let size = CELL_0 * pow(4.0, f32(level));
        let h = h0 * pow(4.0, f32(level));

        // How strongly this level shows. The fine print of normal play is levels 0 and
        // 1. Coarser levels belong to close reading, finer ones to a dive.
        var w = influence_rise(h);
        if level >= 2 {
            w *= influence_fall(h) * coarse;
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
        // A written cell is drawn firmer than grain, and the finer grain beneath it is
        // quietened, so that a sentence is never written over by the wall's own noise.
        let written = authored_character(cell, level, page) != 0u;
        if parent_written {
            w *= 0.3;
        }
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

        if written {
            wanted = max(wanted, 0.40);
        }
        var character = cell_character(cell, level, material, page);
        if state_count > 0 && !written {
            // The words do not sit still: each row slips to a new place every second or
            // so, and between slips its letters flicker out and tear into noise.
            let t_now = view.eye_time.w;
            let span = state_count + 1;
            let row_clock = t_now * 0.9 + unit(hash_cell(vec2<i32>(0, cell.y), level, 17u)) * 9.0;
            let shift = i32(hash_cell(vec2<i32>(cell.y, i32(floor(row_clock)) & 0xffff), level, 19u) % u32(span));
            let i = ((cell.x + cell.y * 3 + shift) % span + span) % span;
            character = select(32u, word_char(material, u32(state), u32(i)), i < state_count);
            let tick = hash_cell(cell, i32(floor(t_now * 12.0)) & 0xffff, 23u);
            let flicker = unit(tick);
            if flicker < 0.10 {
                character = family_character(material, hash_cell(cell, level, material));
            } else if flicker < 0.15 {
                character = 33u + tick % 94u;
            }
        } else if level == 1 && !written {
            // The easter egg. The larger print spells what the thing is: `stone`,
            // `wood`, `iron`, and more often what it is made of: `SiO2`, `CaCO3`.
            //
            // The words are laid out like bricks in a bond: the surface is ruled into
            // patches twenty cells wide and three rows deep, each row of patches set half
            // a patch over from the last, and each patch has one place for a word. The
            // patches take turns, in an ordered pattern, so whatever is showing is always
            // spread evenly and never clumps. More patches show at once the further the
            // eye is, so from a step back the print is scattered with them; up close they
            // come fewer at a time. A word is typed in, held, and taken back. It is written
            // exactly as the grain around it is: it is found by reading.
            let t_now = view.eye_time.w;
            let run = 20;
            let band = i32(floor(f32(cell.y) / 3.0));
            let shift = select(0, run / 2, (band & 1) != 0);
            let column = i32(floor(f32(cell.x + shift) / f32(run)));
            let tile = vec2<i32>(column, band);
            let start = column * run - shift;
            let key = hash_cell(tile, level, 41u);
            // The row of the band the word sits on, and where along the patch it starts.
            let row = band * 3 + 1;
            let bayer = array<f32, 16>(0.0, 8.0, 2.0, 10.0, 12.0, 4.0, 14.0, 6.0, 3.0, 11.0, 1.0, 9.0, 15.0, 7.0, 13.0, 5.0);
            let order = bayer[((tile.y & 3) * 4) + (tile.x & 3)] / 16.0;
            let showing = mix(0.6, 0.3, smoothstep(10.0, 40.0, h));
            let cycle = t_now / 6.0 + order;
            let turn = fract(cycle);
            if cell.y == row && turn < showing {
                let which = hash_cell(tile, i32(floor(cycle)) & 0xffff, 43u);
                // What it is made of, two times in three; its name, the third.
                var slot = 0;
                if m.info.y > 0u && which % 3u != 0u {
                    slot = 1 + i32((which >> 3u) % m.info.y);
                }
                if word_char(material, u32(slot), 0u) != 0u {
                    let count = i32(word_length(material, u32(slot)));
                    let i = cell.x - start - 2 - i32(key % u32(max(run - count - 4, 1)));
                    let e = turn / showing;
                    let typed = f32(count) * smoothstep(0.0, 0.25, e);
                    let erased = f32(count) * smoothstep(0.7, 0.98, e);
                    if i >= 0 && i < count && f32(i) < typed && f32(i) >= erased {
                        character = word_char(material, u32(slot), u32(i));
                    }
                }
            }
        }
        let layer = character - 32u;
        let g = cell_glyph(layer, uv, du, dv, h, wanted);
        modulation += w * nearness(h) * select(1.0, 1.7, written) * (g.mean - g.cover);
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
        parent_written = written;
    }

    result.tone = max(1.0 + contrast * modulation, 0.05);
    result.relief = relief;
    result.ink = ink;
    result.revealed = 1.0;
    return result;
}

// One written letter at a place on the surface: how much of the pixel it covers. Zero
// where nothing is written, and where what is written is a space.
fn written_letter(anchor: vec2<f32>, level: i32, page: u32, footprint: AnchorFootprint, squeeze: f32) -> f32 {
    let size = CELL_0 * pow(4.0, f32(level));
    let p = anchor / size;
    let written = authored_character(vec2<i32>(floor(p)), level, page);
    if written == 0u || written == 32u {
        return 0.0;
    }
    let f = fract(p);
    let scale = vec2<f32>(1.0 / size, -1.0 / size);
    let h = size / squeeze * view.switches.w;
    return cell_glyph(written - 32u, vec2<f32>(f.x, 1.0 - f.y), footprint.dx * scale, footprint.dy * scale, h, 0.40).cover;
}

// How much this place belongs to a written passage, 0 to 1, eased across a cell at the
// passage's edges so that it has no hard border.
fn written_presence(anchor: vec2<f32>, page: u32) -> f32 {
    if page == 0u {
        return 0.0;
    }
    let level = i32(pages.entries[page - 1u].sheet_level.z);
    let size = CELL_0 * pow(4.0, f32(level));
    let p = anchor / size - 0.5;
    let base = vec2<i32>(floor(p));
    let f = smoothstep(vec2<f32>(0.0), vec2<f32>(1.0), fract(p));
    let a = f32(authored_character(base, level, page) != 0u);
    let b = f32(authored_character(base + vec2<i32>(1, 0), level, page) != 0u);
    let c = f32(authored_character(base + vec2<i32>(0, 1), level, page) != 0u);
    let d = f32(authored_character(base + vec2<i32>(1, 1), level, page) != 0u);
    return mix(mix(a, b, f.x), mix(c, d, f.x), f.y);
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
// The living lattice's planes per family: fewer, so each layer is seen as a layer.
const LIVING_STEPS: i32 = 4;

fn unit(h: u32) -> f32 {
    return f32(h & 0xffffu) / 65535.0;
}

fn on_line(c: i32) -> bool {
    return (c % LATTICE + LATTICE) % LATTICE == 0;
}

// The lattice is a frame within a frame within a frame. Every sixth cell is an edge of
// the small frame; every fourth of those is an edge of the middle frame; every fourth
// of those, of the large one. All are written in the same small characters. The
// larger the frame an edge belongs to, the brighter it burns.
fn frame_rank(line: i32) -> f32 {
    let n = ((line % 16) + 16) % 16;
    if n == 0 {
        return 2.0;
    }
    if n % 4 == 0 {
        return 1.45;
    }
    return 1.0;
}

// How a character at a depth is lit. The dark between the letters is a fog: deeper is
// dimmer, and that is all.
fn depth_light(depth: f32, reach: f32, source: bool) -> vec3<f32> {
    return vec3<f32>(exp(-2.3 * clamp(depth / reach, 0.0, 1.0)));
}

// One character of the lattice, on one of its planes. `q` is the place on that plane
// in cells, `dqx` and `dqy` how far that moves across one pixel. Returns how much of
// the pixel the character covers. The lattice is written in the letters of whatever
// it is seen through, or, in the source, in every printable character.
fn lattice_glyph(q: vec2<f32>, dqx: vec2<f32>, dqy: vec2<f32>, salt: i32, material: u32, every: bool, tremble: f32) -> f32 {
    let cell = vec2<i32>(floor(q));
    // Which character stands in a cell does not depend on the material in front of
    // it, only which alphabet it is drawn from: the lattice is one thing.
    let pick = hash_cell(cell, salt, 0u);
    var character = family_character(material, pick);
    if every {
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

// Where a point of the world is, and how that moves across one pixel.
struct WorldFootprint {
    at: vec3<f32>,
    dx: vec3<f32>,
    dy: vec3<f32>,
}

// One lattice, at one scale. The lattice does not belong to any surface. It is fixed
// in the world: a cubic grid of cells of side `size`, with an edge every LATTICE cells
// along north, east, and up, the same grid everywhere. A wall, the frame of the door
// in it, the ground at its foot: each is only a window onto it, so where two things
// meet, the lattice seen through one carries on, unbroken, through the other.
//
// No line of it is drawn. Along each edge stand separate characters, and between the
// edges is nothing but the dark. The ray of this pixel is followed into the surface,
// through the grid's planes, and on each plane it lands on a character or on nothing.
// `ray` points from the eye into the surface. `steep` is how squarely it enters.
// Returns the light gathered, and in w how much of the pixel the characters cover.
fn lattice(world: WorldFootprint, ray: vec3<f32>, steep: f32, distance: f32, size: f32, reach: f32, clarity: f32, material: u32, look: u32, salt: i32) -> vec4<f32> {
    let source = look == 3u;
    let pitch = size * f32(LATTICE);
    let tremble = select(0.0, 0.03, look == 1u);
    // The source streams along its edges.
    let stream = select(0.0, view.eye_time.w * 2.2, source);
    var seen = vec3<f32>(0.0);
    var cover_so_far = 0.0;
    // The furthest along the ray that is still inside the thing.
    let far = reach / steep;

    for (var axis = 0; axis < 3; axis++) {
        let along = ray[axis];
        if abs(along) < 0.03 {
            // Looking straight down these planes: nothing of them faces the eye.
            continue;
        }
        // The two directions that lie in these planes. On an upright plane the second
        // is up, so that its characters stand the right way.
        var a = 0;
        var b = 1;
        if axis == 0 {
            a = 2;
        } else if axis == 1 {
            b = 2;
        }
        let direction = sign(along);
        var plane = floor(world.at[axis] / pitch);
        if direction > 0.0 {
            plane += 1.0;
        }
        for (var k = 0; k < LATTICE_STEPS; k++) {
            let t = (plane * pitch - world.at[axis]) / along;
            let plane_rank = frame_rank(i32(plane));
            let plane_salt = i32(plane) & 63;
            plane += direction;
            if t <= 0.0 {
                continue;
            }
            if t > far {
                break;
            }
            let hit = world.at + ray * t;
            let q = vec2<f32>(hit[a], hit[b]) / size;
            let cell = vec2<i32>(floor(q));
            // An edge is where the grid's lines on this plane run.
            let line_a = on_line(cell.x);
            let line_b = on_line(cell.y);
            if !(line_a || line_b) {
                continue;
            }
            // Along an edge, every other cell is left empty. Each character stands
            // alone with dark on either side of it; the edge is only their alignment.
            if !(line_a && line_b) && ((line_b && (cell.x & 1) != 0) || (line_a && (cell.y & 1) != 0)) {
                continue;
            }
            // How this plane's coordinates move across one pixel: the surface point
            // moves, and the ray, held steady, carries that to the plane. Further
            // planes are further from the eye, and draw smaller.
            let grow = (distance + t) / distance;
            let tx = world.dx[axis] / along;
            let ty = world.dy[axis] / along;
            let dqx = vec2<f32>(world.dx[a] - ray[a] * tx, world.dx[b] - ray[b] * tx) * grow / size;
            let dqy = vec2<f32>(world.dy[a] - ray[a] * ty, world.dy[b] - ray[b] * ty) * grow / size;
            var moved = q;
            if line_b {
                moved.x += stream;
            } else {
                moved.y += stream;
            }
            let c = lattice_glyph(moved, dqx, dqy, salt + axis * 64 + plane_salt, material, source, tremble);
            // The larger the frame an edge belongs to, the brighter it is.
            var gain = plane_rank;
            if line_a {
                gain *= frame_rank(cell.x / LATTICE - select(0, 1, cell.x < 0));
            }
            if line_b {
                gain *= frame_rank(cell.y / LATTICE - select(0, 1, cell.y < 0));
            }
            // The dark between the letters is a fog. It is measured in metres, not in
            // shares of the thing's thickness, so that a thin thing beside a thick one
            // dims at the same rate and no seam shows between them.
            // And it thins as the eye comes closer: from two metres only the first few
            // centimetres can be made out, and at arm's length the far side of a wall.
            let fog = exp(-mix(16.0, 2.6, clarity) * t * steep);
            seen += (1.0 - cover_so_far) * c * min(gain, 3.0) * fog;
            cover_so_far += (1.0 - cover_so_far) * c;
        }
    }
    return vec4<f32>(seen, cover_so_far);
}

// ---- the living lattice: look 4 ----

// One character of a material's lexicon: the `i`th letter of word `slot`, or 0.
fn word_char(material: u32, slot: u32, i: u32) -> u32 {
    if i >= 16u {
        return 0u;
    }
    let w = materials.entries[material].words[slot];
    return (w[i / 4u] >> ((i % 4u) * 8u)) & 0xffu;
}

fn word_length(material: u32, slot: u32) -> u32 {
    var n = 0u;
    for (var i = 0u; i < 16u; i++) {
        if word_char(material, slot, i) == 0u {
            break;
        }
        n += 1u;
    }
    return n;
}

// The word a material's letters say at a heat, as a slot of its lexicon, or -1. The
// hotter, the later the word: wood smokes, then smoulders, then burns.
fn state_slot(material: u32, heat: f32) -> i32 {
    let m = materials.entries[material];
    if heat > 0.15 && m.info.z > 0u {
        let tier = min(u32((heat - 0.15) / 0.85 * f32(m.info.z)), m.info.z - 1u);
        return i32(4u + tier);
    }
    if heat < -0.15 && m.info.w > 0u {
        let tier = min(u32((-heat - 0.15) / 0.85 * f32(m.info.w)), m.info.w - 1u);
        return i32(7u + tier);
    }
    return -1;
}

// What a living lattice gives back: the light of its characters, already coloured and
// lit, and how much of the pixel they cover.
struct Living {
    light: vec3<f32>,
    cover: f32,
}

// The surface a living lattice is seen through: where and how it is lit, and the frame
// its page is written in, for finding the material's colour inside it.
struct Window {
    anchor: vec2<f32>,
    tangent: vec3<f32>,
    bitangent: vec3<f32>,
    // The light falling on the surface, as colour per unit of albedo.
    irradiance: vec3<f32>,
    heat: f32,
}

// One living lattice at one scale. Like `lattice`, it is fixed in the world, and the
// ray of this pixel is followed through its planes. What is different:
//
// - Each character has the colour of the material at the place it stands, inside the
//   thing: a stone's veins and a plank's grain carry on into the depth, in letters.
// - Light reaches into the thing and dies away with depth; what is deep takes on the
//   material's own darker ink, and every character gives off a little light of its
//   own, so the code can be seen in the dark.
// - The corners where three edges meet are the atoms: a heavier character with a soft
//   glow. The edges between them are bonds of single characters.
// - Every character turns over now and then, by a crossfade, on its own clock; the
//   clocks are pushed on by the eye's movement, so walking past sets the lattice
//   changing in a wave. Hot things turn over faster and their atoms shake.
// - Now and then a run of an edge spells a word: the material's name, rarely what it is
//   made of, and, while it is hot or cold, what is happening to it.
fn living_lattice(world: WorldFootprint, ray: vec3<f32>, steep: f32, distance: f32, size: f32, reach: f32, clarity: f32, material: u32, salt: i32, win: Window) -> Living {
    let m = materials.entries[material];
    let pitch = size * f32(LATTICE);
    let t_now = view.eye_time.w;
    let hot = max(win.heat, 0.0);
    let cold = max(-win.heat, 0.0);
    // The clocks: about four seconds a character, faster hot and slower cold, pushed on
    // by where the eye is and where it looks.
    let period = 4.5 * (1.0 + 3.0 * cold) / (1.0 + 5.0 * hot);
    let moved_by_eye = dot(view.eye_time.xyz, vec3<f32>(1.3, 0.9, 1.7)) * 0.9 + dot(ray, vec3<f32>(1.7, 2.3, 1.1)) * 1.4;
    let tremble = 0.015 + hot * 0.11;
    let state = state_slot(material, win.heat);
    let ember = vec3<f32>(1.0, 0.36, 0.07) * hot * hot * 0.5 / view.viewport.z;
    let frost = vec3<f32>(0.55, 0.78, 1.0);
    let far = reach / steep;
    var out = Living(vec3<f32>(0.0), 0.0);

    for (var axis = 0; axis < 3; axis++) {
        let along = ray[axis];
        if abs(along) < 0.03 {
            continue;
        }
        var a = 0;
        var b = 1;
        if axis == 0 {
            a = 2;
        } else if axis == 1 {
            b = 2;
        }
        // Whether this family of planes, seen from this side, would read mirrored. If so
        // every character on it is turned back: no letter is ever seen the wrong way.
        let handed = select(-1.0, 1.0, axis == 2);
        let mirrored = handed * sign(along) > 0.0;
        let direction = sign(along);
        var plane = floor(world.at[axis] / pitch);
        if direction > 0.0 {
            plane += 1.0;
        }
        for (var k = 0; k < LIVING_STEPS; k++) {
            let t = (plane * pitch - world.at[axis]) / along;
            let plane_rank = frame_rank(i32(plane));
            let plane_salt = i32(plane) & 63;
            plane += direction;
            if t <= 0.0 {
                continue;
            }
            if t > far {
                break;
            }
            let hit = world.at + ray * t;
            let q = vec2<f32>(hit[a], hit[b]) / size;
            let cell = vec2<i32>(floor(q));
            let line_a = on_line(cell.x);
            let line_b = on_line(cell.y);
            let depth = t * steep;
            // Each plane further in is fainter by a clear step, so that the eye can
            // count the layers: the depth is read, not guessed.
            let fog = exp(-mix(18.0, 4.0, clarity) * depth) * pow(0.72, f32(k));
            let grow = (distance + t) / distance;
            let tx = world.dx[axis] / along;
            let ty = world.dy[axis] / along;
            let dqx = vec2<f32>(world.dx[a] - ray[a] * tx, world.dx[b] - ray[b] * tx) * grow / size;
            let dqy = vec2<f32>(world.dy[a] - ray[a] * ty, world.dy[b] - ray[b] * ty) * grow / size;
            let px = view.switches.w / max(max(length(dqx), length(dqy)), 1.0e-6);
            if px < 2.5 {
                continue;
            }

            // The material at this place inside the thing: its colour, and its light.
            let offset = hit - world.at;
            let place = win.anchor + vec2<f32>(dot(offset, win.tangent), dot(offset, win.bitangent));
            let uv = vec2<f32>(place.x, -place.y) / m.surface.x + depth * vec2<f32>(0.23, 0.37) / m.surface.x;
            let albedo = textureSampleLevel(material_colour, material_sampler, uv, material, 3.0).rgb;
            let hue = albedo / max(max(albedo.r, max(albedo.g, albedo.b)), 1.0e-3);
            var tint = mix(vec3<f32>(1.0), hue * 0.7, smoothstep(0.0, 0.6, depth));
            if cold > 0.0 {
                tint = mix(tint, frost, cold * 0.6);
            }
            let inner = win.irradiance * exp(-depth * 7.0) + hue * 0.05 / view.viewport.z;
            var colour = albedo * tint * inner + ember;

            // The atoms: a soft light around each corner where three edges meet, seen
            // on every plane, whether or not a character stands where the ray lands.
            let node = round(q / f32(LATTICE)) * f32(LATTICE) + 0.5;
            let near_node = length(q - node);
            let halo = exp(-near_node * near_node / 2.2) * 0.22 * (0.7 + 0.3 * plane_rank) * smoothstep(4.0, 9.0, px);
            out.light += (1.0 - out.cover) * colour * 1.6 * halo * fog;

            // A word, if one is forming on this edge.
            var word_slot = -1;
            var letter = 0u;
            var envelope = 0.0;
            if line_b {
                let run = 24;
                let start = (cell.x - ((cell.x % run) + run) % run);
                let key = hash_cell(vec2<i32>(start, cell.y), salt + axis * 64 + plane_salt, 7u);
                let epoch_x = t_now / (period * 1.2) + unit(key) * 7.0 + moved_by_eye * 0.25;
                let pick = hash_cell(vec2<i32>(start, cell.y), i32(floor(epoch_x)) & 0xffff, 11u);
                let r = unit(pick);
                if state >= 0 && r < 0.6 {
                    word_slot = state;
                } else if word_char(material, 0u, 0u) != 0u && r < 0.12 {
                    // Common enough to be seen at the first or second look, never for
                    // long: a few runs in any view at once, each for a few seconds.
                    word_slot = 0;
                } else if m.info.y > 0u && r < 0.20 && depth > 0.02 {
                    word_slot = 1 + i32(pick % m.info.y);
                }
                if word_slot >= 0 {
                    let count = i32(word_length(material, u32(word_slot)));
                    var at = start + 2;
                    var e = fract(epoch_x);
                    var glitch = 0u;
                    if word_slot == state {
                        // What is happening to the thing does not wait its turn. Its word
                        // glitches in and out: it lives about a second, jumps to a new
                        // place along the edge each time it comes back, stutters on and
                        // off, slips a cell sideways, and loses letters to noise.
                        let quick = t_now / (0.7 + 0.5 * unit(key >> 9u)) + unit(key) * 5.0;
                        let jump = hash_cell(vec2<i32>(start, cell.y), i32(floor(quick)) & 0xffff, 29u);
                        at = start + i32(jump % u32(max(run - count, 1)));
                        e = fract(quick);
                        glitch = hash_cell(vec2<i32>(start, cell.y), i32(floor(t_now * 13.0)) & 0xffff, 31u);
                        if unit(glitch) < 0.12 {
                            at += select(-1, 1, (glitch & 1u) == 0u);
                        }
                    }
                    var i = cell.x - at;
                    if mirrored {
                        i = count - 1 - i;
                    }
                    if i >= 0 && i < count {
                        letter = word_char(material, u32(word_slot), u32(i));
                        envelope = smoothstep(0.0, 0.12, e) * (1.0 - smoothstep(0.78, 1.0, e));
                        if word_slot == state {
                            envelope = smoothstep(0.0, 0.05, e) * (1.0 - smoothstep(0.85, 1.0, e));
                            // Stutter: the whole word drops out for a tick now and then.
                            if unit(glitch >> 8u) < 0.2 {
                                envelope = 0.0;
                            }
                            // Noise: a letter is now and then some other character.
                            let torn = hash_cell(cell, i32(floor(t_now * 17.0)) & 0xffff, 37u);
                            if unit(torn) < 0.14 {
                                letter = 33u + torn % 94u;
                            }
                        }
                    }
                }
            }

            let is_atom = line_a && line_b;
            // Bonds: every third cell along an edge, so each character stands clear.
            let along_edge = select(cell.y, cell.x, line_b);
            let bond = (line_a || line_b) && !is_atom && ((along_edge % 3) + 3) % 3 == 0;
            if !(is_atom || bond || letter != 0u) {
                continue;
            }

            // The character, on its clock.
            let key = hash_cell(cell, salt + axis * 64 + plane_salt, 3u);
            let clock = t_now / period + unit(key >> 4u) * 11.0 + moved_by_eye;
            let fade = smoothstep(0.0, 0.18, fract(clock));
            let now = hash_cell(cell, i32(floor(clock)) & 0xffff, material + 5u);
            let before = hash_cell(cell, (i32(floor(clock)) - 1) & 0xffff, material + 5u);
            var f = fract(q);
            let phase = unit(key) * 6.283;
            f += vec2<f32>(sin(t_now * (4.1 + hot * 9.0) + phase), cos(t_now * (3.3 + hot * 7.0) + phase * 1.7)) * tremble;
            var glyph_uv = clamp(vec2<f32>(f.x, 1.0 - f.y), vec2<f32>(0.0), vec2<f32>(1.0));
            var du = vec2<f32>(dqx.x, -dqx.y);
            var dv = vec2<f32>(dqy.x, -dqy.y);
            if mirrored {
                glyph_uv.x = 1.0 - glyph_uv.x;
                du.x = -du.x;
                dv.x = -dv.x;
            }
            let shown = smoothstep(2.5, 5.0, px);
            var character_of = family_character(material, now);
            var character_was = family_character(material, before);
            if is_atom {
                let heavy = select(111u, 64u, plane_rank > 1.2);
                character_of = heavy;
                character_was = heavy;
            }
            var c = cell_glyph(character_of - 32u, glyph_uv, du, dv, px, 0.34).cover;
            if fade < 1.0 {
                c = mix(cell_glyph(character_was - 32u, glyph_uv, du, dv, px, 0.34).cover, c, fade);
            }
            if bond == false && is_atom == false {
                c = 0.0;
            }
            var gain = min(plane_rank, 3.0) * select(1.0, 1.5, is_atom);
            if letter != 0u {
                // A word is drawn whole, a little brighter, and in the colour of what is
                // happening to the thing while it is hot or cold.
                let w = cell_glyph(letter - 32u, glyph_uv, du, dv, px, 0.42).cover;
                c = mix(c, w, envelope);
                if word_slot == state {
                    colour = mix(colour, select(frost * 0.6 / view.viewport.z, ember * 1.8 + albedo * inner, hot > 0.0), envelope);
                }
            }
            c *= shown;
            out.light += (1.0 - out.cover) * c * colour * gain * fog;
            out.cover += (1.0 - out.cover) * c;
        }
    }
    return out;
}

// `lit` is the surface as shaded. `to_eye` is the direction to the eye in the
// surface's own frame: along the page, up the page, and out of it. `distance` is how
// far the eye is. `thickness` is how deep the thing is behind this surface. `ink` is
// how much of this pixel is a stroke of the surface's own fine print.
fn glyph_depth(lit: vec3<f32>, anchor: vec2<f32>, footprint: AnchorFootprint, to_eye: vec3<f32>, world: WorldFootprint, ray: vec3<f32>, distance: f32, material: u32, page: u32, thickness: f32, ink: f32, look: u32, win: Window) -> vec3<f32> {
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

    // The realistic texture is not cut away with any shape of its own. It retreats into
    // the letters it was always made of: first it thins, and what is left of it is left
    // only inside the strokes of the surface's fine print, each stroke still carrying
    // the colour and the light of that spot. Nothing is on screen but characters.
    let into_letters = smoothstep(0.0, 0.65, near);
    var film = mix(1.0, ink, into_letters);
    if look == 4u {
        // The living look opens the surface as the eye comes: letter by letter the
        // front print lets go, each on its own threshold and each fading rather than
        // vanishing, until about three in four are gone and the eye is inside.
        let front = vec2<i32>(floor(anchor / CELL_0));
        let threshold = unit(hash_cell(front, 0, material + 91u));
        let opened = smoothstep(0.15, 1.0, near) * 0.78;
        let stays = smoothstep(threshold - 0.1, threshold + 0.1, 1.0 - opened);
        film = mix(1.0, ink * stays, into_letters);
    }
    if look == 2u {
        // The lattice look keeps a veil of the texture over what lies behind.
        film = mix(1.0, max(ink, 0.42), into_letters);
    }
    var keep = film;

    // Written words come first. Where the surface carries a sentence its letters stand
    // whole at the front, and they are given what the solid wall could not give them:
    // thickness. Each letter is drawn again a few millimetres behind itself, and again,
    // so that seen from the side it is a small block of the thing's own stuff. Around
    // the passage the lattice is hushed, easing off at its edges, so that the sentence
    // can be read through everything else.
    let steep = max(to_eye.z, 0.15);
    // How far the ray moves across the page for each metre it goes in.
    let slide = to_eye.xy / steep;
    let presence = written_presence(anchor, page);
    var sentence = 0.0;
    var body = 0.0;
    if presence > 0.0 {
        let level = i32(pages.entries[page - 1u].sheet_level.z);
        sentence = written_letter(anchor, level, page, footprint, squeeze);
        // The letter's thickness is a fifth of its cell.
        let thick = CELL_0 * pow(4.0, f32(level)) * 0.2;
        for (var step = 1; step <= 4; step++) {
            let back = thick * f32(step) / 4.0;
            let side = written_letter(anchor - slide * back, level, page, footprint, squeeze);
            body = max(body, side * (1.0 - 0.15 * f32(step)));
        }
        // Nothing else is left where words are written: no dots, no grain. In the living
        // look the words stand on the material itself, as letters cut in stone do.
        keep = mix(keep, select(0.0, 0.8, look == 4u), presence * into_letters);
    }
    let hush = mix(1.0, 0.10, presence);
    if keep >= 0.999 && sentence <= 0.0 && body <= 0.0 {
        return lit;
    }

    // The lattice, and the lattice inside it. Each is three frames nested in one
    // another, in characters all of one size. And inside the cubes of the first lattice
    // is a second, a quarter the size, which shows when the eye is close enough to
    // read it: walking closer does not reach the end of it. It finds the next one.
    let gold = vec3<f32>(1.0, 0.58, 0.14) * 0.55 / view.viewport.z;
    // A thin thing is still a window onto the lattice: the eye sees at least a
    // quarter of a metre into it, so that thin and thick things meet without a step.
    let reach = clamp(thickness, 0.25, 1.2);
    // How far in the eye can see grows as it comes closer.
    let clarity = smoothstep(log2(8.0), log2(90.0), log2(max(h, 1.0e-3)));
    var seen = vec3<f32>(0.0);
    var cover_so_far = 0.0;
    for (var scale = 1; scale >= 0; scale--) {
        // 1.28 cm cells, and 0.32 cm cells for the lattice inside the lattice.
        let cell_size = CELL_0 * 0.5 * pow(4.0, f32(scale));
        let cell_pixels = cell_size / squeeze * view.switches.w;
        let show = smoothstep(log2(5.0), log2(14.0), log2(max(cell_pixels, 1.0e-3)));
        if show <= 0.0 {
            continue;
        }
        // The small lattice does not run as deep: it fills the cube it stands in.
        let scale_reach = min(reach, cell_size * f32(LATTICE) * f32(LATTICE_STEPS));
        let strength = show * select(0.6, 1.0, scale == 1);
        if look == 4u {
            // The living lattice comes lit and coloured; the inner lattice shows only
            // within arm's reach, a little fainter.
            let l = living_lattice(world, ray, steep, distance, cell_size, scale_reach, clarity, material, scale * 200, win);
            let s = strength * select(0.7, 1.0, scale == 1);
            seen += (1.0 - cover_so_far) * l.light * s;
            cover_so_far += (1.0 - cover_so_far) * l.cover * s;
        } else {
            let l = lattice(world, ray, steep, distance, cell_size, scale_reach, clarity, material, look, scale * 200);
            seen += (1.0 - cover_so_far) * l.rgb * strength;
            cover_so_far += (1.0 - cover_so_far) * l.w * strength;
        }
    }

    var behind = lit * 1.05 * seen;
    if source {
        behind = gold * seen;
    }
    if look == 4u {
        behind = seen;
    }
    let through = 1.0 - cover_so_far;
    // Whatever no letter covers is the dark the letters hang in.
    var dark = lit * 0.012;
    if source {
        dark = vec3<f32>(0.004, 0.0025, 0.0008) / view.viewport.z;
    }
    if look == 4u {
        // In the living look the gaps are not empty. The thing is still there, as a
        // haze of its own colour and light, thick at arm's length and thinning as the
        // eye leans in, so that the model is never lost while it is looked into.
        dark = lit * mix(0.42, 0.07, clarity);
    }
    behind += through * dark;
    var colour = mix(behind * hush, lit, keep);
    // The sides of the written letters, in shade, then their faces, catching the light.
    colour = mix(colour, lit * 0.34, body * near);
    var face = lit * (1.0 + 0.3 * near);
    if source {
        // Over the source, words take a little of its gold, as leaf does.
        face = mix(face, gold * 2.2, 0.3 * near);
    }
    return mix(colour, face, sentence);
}
