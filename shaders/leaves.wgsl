// The trees' leaf cards and the screen-door cuts that trees are drawn with. Each card is
// a spray of leaves: a few overlapping ovals cut from its square. Matches `leafy` and
// `CARD_STRIDE` in crates/bake/src/trees.rs, so the far form's picture is the same
// leaves the near levels cut.

// Cards are set this far apart along the anchor's first axis, so that a card's place
// says which card it is.
const CARD_STRIDE: f32 = 100.0;
// Seeds of the cards' sprays: the card's number times this.
const CARD_SEED: u32 = 7919u;

fn leaf_next(state: ptr<function, u32>) -> f32 {
    *state = *state * 747796405u + 2891336453u;
    var x = *state;
    x ^= x >> 16u;
    x *= 0x7feb352du;
    x ^= x >> 15u;
    return f32(x & 0x00ffffffu) / 16777216.0;
}

fn leaf_range(state: ptr<function, u32>, a: f32, b: f32) -> f32 {
    return a + (b - a) * leaf_next(state);
}

// How far inside the nearest leaf a point of a card is, in the card's own square from 0
// to 1: below 1 is leaf.
fn leaf_distance(u: f32, v: f32, seed: u32) -> f32 {
    var state = seed ^ 0xa5a5u;
    var nearest = 1.0e3;
    for (var k = 0; k < 7; k++) {
        let cx = leaf_range(&state, 0.2, 0.8);
        let cy = leaf_range(&state, 0.2, 0.8);
        let rx = leaf_range(&state, 0.12, 0.24);
        let ry = leaf_range(&state, 0.07, 0.14);
        let a = leaf_range(&state, 0.0, 3.14159265);
        let s = sin(a);
        let c = cos(a);
        let d = vec2<f32>(u - cx, v - cy);
        let p = vec2<f32>(d.x * c + d.y * s, -d.x * s + d.y * c);
        nearest = min(nearest, dot(p / vec2<f32>(rx, ry), p / vec2<f32>(rx, ry)));
    }
    return nearest;
}

// Which card a leaf card's point is on, and where on it, from 0 to 1. `side` is the
// card's side in metres, which its vertices carry as their thickness.
fn card_of(anchor: vec2<f32>) -> f32 {
    return floor(anchor.x / CARD_STRIDE);
}

fn on_card(anchor: vec2<f32>, side: f32) -> vec2<f32> {
    let card = card_of(anchor);
    return vec2<f32>(anchor.x - card * CARD_STRIDE, anchor.y) / max(side, 1.0e-3);
}

fn leaf_at(anchor: vec2<f32>, side: f32) -> f32 {
    let uv = on_card(anchor, side);
    return leaf_distance(uv.x, uv.y, u32(card_of(anchor)) * CARD_SEED);
}

// A screen-door threshold, 0 to 1, that changes from pixel to pixel and with `salt`.
fn screen_door(pixel: vec2<f32>, salt: f32) -> f32 {
    return fract(52.9829189 * fract(dot(floor(pixel), vec2<f32>(0.06711056, 0.00583715)) + salt));
}
