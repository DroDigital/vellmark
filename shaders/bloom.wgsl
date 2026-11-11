// Bloom: the lit scene is shrunk step by step, then grown back, each step adding the
// one below. What comes out is the glow of whatever is very bright, with no threshold
// and no halo on things that are not.

@group(0) @binding(0) var source: texture_2d<f32>;
@group(0) @binding(1) var source_sampler: sampler;

struct Varyings {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
}

@vertex
fn vs(@builtin(vertex_index) index: u32) -> Varyings {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    var out: Varyings;
    out.position = vec4<f32>(corner * 2.0 - 1.0, 0.0, 1.0);
    out.uv = vec2<f32>(corner.x, 1.0 - corner.y);
    return out;
}

fn tap(uv: vec2<f32>) -> vec3<f32> {
    return textureSampleLevel(source, source_sampler, uv, 0.0).rgb;
}

// Weighs a sample down by its brightness, so that one burning pixel cannot flicker a
// whole glow on and off as it moves.
fn calm(c: vec3<f32>) -> vec4<f32> {
    let w = 1.0 / (1.0 + max(c.r, max(c.g, c.b)));
    return vec4<f32>(c * w, w);
}

@fragment
fn fs_first(in: Varyings) -> @location(0) vec4<f32> {
    let texel = 1.0 / vec2<f32>(textureDimensions(source));
    var sum = vec4<f32>(0.0);
    sum += calm(tap(in.uv + texel * vec2<f32>(-1.0, -1.0)));
    sum += calm(tap(in.uv + texel * vec2<f32>(1.0, -1.0)));
    sum += calm(tap(in.uv + texel * vec2<f32>(-1.0, 1.0)));
    sum += calm(tap(in.uv + texel * vec2<f32>(1.0, 1.0)));
    return vec4<f32>(sum.rgb / max(sum.a, 1.0e-4), 1.0);
}

@fragment
fn fs_down(in: Varyings) -> @location(0) vec4<f32> {
    let texel = 1.0 / vec2<f32>(textureDimensions(source));
    var sum = tap(in.uv) * 4.0;
    sum += tap(in.uv + texel * vec2<f32>(-1.0, -1.0));
    sum += tap(in.uv + texel * vec2<f32>(1.0, -1.0));
    sum += tap(in.uv + texel * vec2<f32>(-1.0, 1.0));
    sum += tap(in.uv + texel * vec2<f32>(1.0, 1.0));
    return vec4<f32>(sum / 8.0, 1.0);
}

// A nine-tap tent, added onto the larger level beneath.
@fragment
fn fs_up(in: Varyings) -> @location(0) vec4<f32> {
    let t = 1.0 / vec2<f32>(textureDimensions(source));
    var sum = tap(in.uv) * 4.0;
    sum += (tap(in.uv + vec2<f32>(t.x, 0.0)) + tap(in.uv - vec2<f32>(t.x, 0.0)) + tap(in.uv + vec2<f32>(0.0, t.y)) + tap(in.uv - vec2<f32>(0.0, t.y))) * 2.0;
    sum += tap(in.uv + t) + tap(in.uv - t) + tap(in.uv + vec2<f32>(t.x, -t.y)) + tap(in.uv + vec2<f32>(-t.x, t.y));
    return vec4<f32>(sum / 16.0, 1.0);
}
