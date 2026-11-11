// From the lit scene to the screen: bloom, exposure, the filmic curve, and the edge
// filter.

struct PostUniform {
    // x: exposure. y: bloom strength. z: edge filter strength. w: unused.
    settings: vec4<f32>,
}

@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var scene_sampler: sampler;
@group(0) @binding(2) var<uniform> post: PostUniform;
@group(0) @binding(3) var bloom: texture_2d<f32>;

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

// A filmic curve: a gentle toe, a long shoulder, and highlights that roll off to white
// without turning grey. (The fit by Krzysztof Narkowicz of the ACES reference curve.)
fn filmic(x: vec3<f32>) -> vec3<f32> {
    let a = 2.51;
    let b = 0.03;
    let c = 2.43;
    let d = 0.59;
    let e = 0.14;
    return clamp((x * (a * x + b)) / (x * (c * x + d) + e), vec3<f32>(0.0), vec3<f32>(1.0));
}

// The scene at a point, as it will be displayed.
fn shown(uv: vec2<f32>, glow: vec3<f32>) -> vec3<f32> {
    let hdr = textureSampleLevel(scene, scene_sampler, uv, 0.0).rgb;
    return filmic((hdr + glow) * post.settings.x);
}

fn luma(c: vec3<f32>) -> f32 {
    return dot(c, vec3<f32>(0.299, 0.587, 0.114));
}

@fragment
fn fs(in: Varyings) -> @location(0) vec4<f32> {
    let texel = 1.0 / vec2<f32>(textureDimensions(scene));
    let glow = textureSampleLevel(bloom, scene_sampler, in.uv, 0.0).rgb * post.settings.y;
    let centre = textureSampleLevel(scene, scene_sampler, in.uv, 0.0);
    let middle = filmic((centre.rgb + glow) * post.settings.x);

    // The edge filter: where the picture steps sharply, blend a little along the edge.
    // Multisampling has already smoothed the outlines of things; this catches what it
    // cannot, the sparkle of a bright highlight or a hard line inside a surface. It is
    // switched off where letters are drawn, so that type stays crisp.
    let strength = post.settings.z * (1.0 - centre.a);
    if strength <= 0.0 {
        return vec4<f32>(middle, 1.0);
    }
    let nw = luma(shown(in.uv + texel * vec2<f32>(-0.5, -0.5), glow));
    let ne = luma(shown(in.uv + texel * vec2<f32>(0.5, -0.5), glow));
    let sw = luma(shown(in.uv + texel * vec2<f32>(-0.5, 0.5), glow));
    let se = luma(shown(in.uv + texel * vec2<f32>(0.5, 0.5), glow));
    let m = luma(middle);
    let low = min(m, min(min(nw, ne), min(sw, se)));
    let high = max(m, max(max(nw, ne), max(sw, se)));
    if high - low < max(0.04, high * 0.12) {
        return vec4<f32>(middle, 1.0);
    }
    var dir = vec2<f32>(-((nw + ne) - (sw + se)), (nw + sw) - (ne + se));
    let reduce = max((nw + ne + sw + se) * 0.03125, 1.0 / 128.0);
    let scale = 1.0 / (min(abs(dir.x), abs(dir.y)) + reduce);
    dir = clamp(dir * scale, vec2<f32>(-6.0), vec2<f32>(6.0)) * texel;
    let near = 0.5 * (shown(in.uv + dir * (1.0 / 3.0 - 0.5), glow) + shown(in.uv + dir * (2.0 / 3.0 - 0.5), glow));
    let wide = near * 0.5 + 0.25 * (shown(in.uv - dir * 0.5, glow) + shown(in.uv + dir * 0.5, glow));
    let wide_luma = luma(wide);
    var filtered = wide;
    if wide_luma < low || wide_luma > high {
        filtered = near;
    }
    return vec4<f32>(mix(middle, filtered, strength), 1.0);
}
