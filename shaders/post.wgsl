// From the lit scene to the screen: exposure, the filmic curve, and the grade.

struct PostUniform {
    // x: exposure. yzw: unused.
    settings: vec4<f32>,
}

@group(0) @binding(0) var scene: texture_2d<f32>;
@group(0) @binding(1) var scene_sampler: sampler;
@group(0) @binding(2) var<uniform> post: PostUniform;

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

@fragment
fn fs(in: Varyings) -> @location(0) vec4<f32> {
    let hdr = textureSampleLevel(scene, scene_sampler, in.uv, 0.0).rgb;
    return vec4<f32>(filmic(hdr * post.settings.x), 1.0);
}
