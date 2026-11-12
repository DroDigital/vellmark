// Contact occlusion, at half resolution: the darkening where things meet, too fine for
// the baked share of sky. From the depth pre-pass, each half-resolution point looks at
// eight neighbours on a small spiral, about 35 cm across, and counts how much of what is
// around it rises in front of its surface. Only the sky's light and the ground's bounce
// are darkened by it, never the sun's or a lamp's: those have their own shadows.

#import frame

const PI: f32 = 3.14159265;

@group(0) @binding(0) var<uniform> view: ViewUniform;
@group(0) @binding(1) var scene_depth: texture_depth_multisampled_2d;
@group(0) @binding(2) var occlusion: texture_storage_2d<rgba8unorm, write>;

// The world position seen at a pixel of the target, or w = 0 where there is only sky.
fn seen_at(pixel: vec2<i32>) -> vec4<f32> {
    let size = vec2<i32>(textureDimensions(scene_depth));
    let p = clamp(pixel, vec2<i32>(0), size - 1);
    let d = textureLoad(scene_depth, p, 0);
    if d <= 0.0 {
        return vec4<f32>(0.0);
    }
    let local = (vec2<f32>(p) + 0.5 - view.viewport_origin.xy) / view.viewport.xy;
    let ndc = vec2<f32>(local.x * 2.0 - 1.0, 1.0 - local.y * 2.0);
    let w = view.inv_view_proj * vec4<f32>(ndc, d, 1.0);
    return vec4<f32>(w.xyz / w.w, 1.0);
}

@compute @workgroup_size(8, 8, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let half_size = vec2<u32>(view.viewport.xy * 0.5);
    if any(id.xy >= half_size) {
        return;
    }
    let pixel = vec2<i32>(view.viewport_origin.xy) + vec2<i32>(id.xy) * 2;
    let out_at = pixel / 2;
    let centre = seen_at(pixel);
    if centre.w == 0.0 {
        textureStore(occlusion, out_at, vec4<f32>(1.0));
        return;
    }
    let right = seen_at(pixel + vec2<i32>(2, 0));
    let down = seen_at(pixel + vec2<i32>(0, 2));
    let to_eye = normalize(view.eye_time.xyz - centre.xyz);
    var normal = normalize(cross(down.xyz - centre.xyz, right.xyz - centre.xyz));
    if dot(normal, to_eye) < 0.0 {
        normal = -normal;
    }
    // Pixels to a metre here, and so the spiral's size on screen.
    let metre = 2.0 / max(length(right.xyz - centre.xyz), 1.0e-4);
    let radius = clamp(0.35 * metre, 2.0, 32.0);
    let turn = fract(sin(dot(vec2<f32>(pixel), vec2<f32>(12.9898, 78.233))) * 43758.547) * 6.283;
    var hidden = 0.0;
    for (var k = 0; k < 8; k++) {
        let a = f32(k) * 2.39996 + turn;
        let r = radius * (f32(k) + 1.0) / 8.0;
        let s = seen_at(pixel + vec2<i32>(round(vec2<f32>(cos(a), sin(a)) * r)));
        if s.w == 0.0 {
            continue;
        }
        let v = s.xyz - centre.xyz;
        let d = length(v);
        hidden += max(dot(normal, v / max(d, 1.0e-4)) - 0.15, 0.0) * (1.0 - smoothstep(0.15, 0.5, d));
    }
    let open = clamp(1.0 - hidden / 8.0 * 1.6, 0.0, 1.0);
    textureStore(occlusion, out_at, vec4<f32>(open));
}
