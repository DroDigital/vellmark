// The test field: a sky and a ruled plane, drawn from one triangle that covers the view.
// It exists to prove the frame loop, the capture tool, and the shimmer check before the
// renderer is built. The lit wall of Phase S01 replaces it.

#import frame

@group(0) @binding(0) var<uniform> view: ViewUniform;

struct Varyings {
    @builtin(position) position: vec4<f32>,
    @location(0) ndc: vec2<f32>,
}

@vertex
fn vs(@builtin(vertex_index) index: u32) -> Varyings {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    var out: Varyings;
    out.ndc = corner * 2.0 - 1.0;
    out.position = vec4<f32>(out.ndc, 0.0, 1.0);
    return out;
}

fn sky(dir: vec3<f32>, sun: vec3<f32>) -> vec3<f32> {
    let day = clamp(sun.y * 3.0 + 0.3, 0.0, 1.0);
    let horizon = mix(vec3<f32>(0.02, 0.025, 0.05), vec3<f32>(0.62, 0.74, 0.90), day);
    let zenith = mix(vec3<f32>(0.002, 0.004, 0.012), vec3<f32>(0.12, 0.30, 0.68), day);
    var colour = mix(horizon, zenith, pow(clamp(dir.y, 0.0, 1.0), 0.55));
    let warmth = pow(clamp(dot(dir, sun), 0.0, 1.0), 8.0) * clamp(1.0 - sun.y * 2.0, 0.0, 1.0) * day;
    colour += vec3<f32>(0.9, 0.35, 0.08) * warmth;
    let disc = smoothstep(0.99985, 0.99995, dot(dir, sun));
    return colour + vec3<f32>(8.0, 7.0, 5.5) * disc;
}

// A ruled line, antialiased from the footprint of the pixel so that it never crawls.
fn ruled(coord: vec2<f32>, spacing: f32, width: f32) -> f32 {
    let p = coord / spacing;
    let fw = max(fwidth(p), vec2<f32>(1.0e-6));
    let d = abs(fract(p - 0.5) - 0.5);
    let line = vec2<f32>(1.0) - smoothstep(vec2<f32>(width) - fw, vec2<f32>(width) + fw, d);
    // Where lines are closer than a pixel, settle to their average coverage.
    let fade = clamp(vec2<f32>(1.0) - (fw - 0.25) * 4.0, vec2<f32>(0.0), vec2<f32>(1.0));
    let covered = mix(vec2<f32>(width * 2.0), line, fade);
    return max(covered.x, covered.y);
}

@fragment
fn fs(in: Varyings) -> @location(0) vec4<f32> {
    let far = view.inv_view_proj * vec4<f32>(in.ndc, 1.0, 1.0);
    let dir = normalize(far.xyz / far.w - view.eye_time.xyz);
    let sun = normalize(view.sun_dir_hour.xyz);
    var colour = sky(dir, sun);

    if dir.y < -1.0e-4 {
        let t = -view.eye_time.y / dir.y;
        let hit = view.eye_time.xyz + dir * t;
        let light = clamp(sun.y, 0.0, 1.0) * 0.85 + 0.08;
        var ground = vec3<f32>(0.33, 0.36, 0.25);
        ground = mix(ground, vec3<f32>(0.20, 0.22, 0.15), ruled(hit.xz, 1.0, 0.03));
        ground = mix(ground, vec3<f32>(0.75, 0.70, 0.55), ruled(hit.xz, 10.0, 0.012));
        let haze = 1.0 - exp(-t * 0.004);
        colour = mix(ground * light, sky(vec3<f32>(dir.x, 0.0, dir.z), sun), haze);
    }
    return vec4<f32>(colour, 1.0);
}
