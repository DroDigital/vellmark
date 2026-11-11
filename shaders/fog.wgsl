// The fog: a coarse grid of cells in front of each view, 160 across, 90 down, and 64
// deep, spaced ever further apart from half a metre to four hundred. The first pass
// finds how much light each cell's mist scatters towards the eye, with the sun's light
// shut out where the shadow maps say it is shadowed: that is what makes god rays. The
// second walks each column of cells from the eye outwards and adds them up, so that
// any surface or the sky can read how much mist lies between it and the eye.

#import frame

@group(0) @binding(0) var<uniform> view: ViewUniform;
@group(0) @binding(1) var shadow_map: texture_depth_2d_array;
@group(0) @binding(2) var shadow_sampler: sampler_comparison;
@group(0) @binding(3) var injected: texture_storage_3d<rgba16float, write>;

@group(1) @binding(0) var injected_in: texture_3d<f32>;
@group(1) @binding(1) var integrated: texture_storage_3d<rgba16float, write>;

const FOG_SIZE: vec3<u32> = vec3<u32>(160u, 90u, 64u);
const FOG_NEAR: f32 = 0.5;
const FOG_FAR: f32 = 400.0;
const PI: f32 = 3.14159265;

// The distance in front of the eye of the near edge of a slice.
fn slice_depth(slice: f32) -> f32 {
    return FOG_NEAR * pow(FOG_FAR / FOG_NEAR, slice / f32(FOG_SIZE.z));
}

// The direction from the eye through the middle of a column of cells, and how far along
// it one metre in front of the eye is.
fn column_ray(xy: vec2<u32>) -> vec4<f32> {
    let uv = (vec2<f32>(xy) + 0.5) / vec2<f32>(FOG_SIZE.xy);
    let ndc = vec2<f32>(uv.x * 2.0 - 1.0, 1.0 - uv.y * 2.0);
    let a = view.inv_view_proj * vec4<f32>(ndc, 1.0, 1.0);
    let b = view.inv_view_proj * vec4<f32>(ndc, 0.5, 1.0);
    let dir = normalize(b.xyz / b.w - a.xyz / a.w);
    let forward = -vec3<f32>(view.view[0].z, view.view[1].z, view.view[2].z);
    return vec4<f32>(dir, 1.0 / max(dot(dir, forward), 0.05));
}

fn sun_reaches(world: vec3<f32>, depth_in_view: f32) -> f32 {
    var cascade = 2u;
    if depth_in_view < view.cascade_far.x {
        cascade = 0u;
    } else if depth_in_view < view.cascade_far.y {
        cascade = 1u;
    }
    if depth_in_view > view.cascade_far.z {
        return 1.0;
    }
    let p = view.shadow[cascade] * vec4<f32>(world, 1.0);
    let uv = p.xy * vec2<f32>(0.5, -0.5) + 0.5;
    if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) {
        return 1.0;
    }
    return textureSampleCompareLevel(shadow_map, shadow_sampler, uv, cascade, p.z);
}

// How thick the mist is at a place: thickest at its floor, thinning upwards, and
// drifting in slow patches.
fn mist(world: vec3<f32>) -> f32 {
    let density = view.sky.y;
    if density <= 0.0 {
        return 0.0;
    }
    let above = max(world.y - view.sky.z, 0.0);
    let patches = 0.55 + 0.9 * sky_fbm(world.xz / 90.0 + vec2<f32>(0.02, 0.008) * view.eye_time.w);
    return density * exp(-above / max(view.sky.w, 0.5)) * patches;
}

@compute @workgroup_size(8, 8, 1)
fn inject(@builtin(global_invocation_id) id: vec3<u32>) {
    if any(id >= FOG_SIZE) {
        return;
    }
    let ray = column_ray(id.xy);
    let depth = slice_depth(f32(id.z) + 0.5);
    let world = view.eye_time.xyz + ray.xyz * depth * ray.w;
    let density = mist(world);
    let light = view.light_dir_hour.xyz;
    // Mist scatters mostly forwards: looking towards the low sun, it glows.
    let mu = dot(ray.xyz, light);
    let g = 0.55;
    let phase = (1.0 - g * g) / (4.0 * PI * pow(1.0 + g * g - 2.0 * g * mu, 1.5));
    let sun = view.light_colour.rgb * sun_reaches(world, depth) * phase * 4.0 * PI;
    let sky = (view.sky_zenith.rgb + view.sky_horizon.rgb) * 0.5;
    textureStore(injected, id, vec4<f32>((sun + sky) * density, density * 1.05));
}

@compute @workgroup_size(8, 8, 1)
fn integrate(@builtin(global_invocation_id) id: vec3<u32>) {
    if any(id.xy >= FOG_SIZE.xy) {
        return;
    }
    let ray = column_ray(id.xy);
    var light = vec3<f32>(0.0);
    var through = 1.0;
    var near = 0.0;
    for (var s = 0u; s < FOG_SIZE.z; s++) {
        let far = slice_depth(f32(s) + 1.0);
        let span = (far - near) * ray.w;
        let cell = textureLoad(injected_in, vec3<u32>(id.xy, s), 0);
        let extinction = max(cell.a, 1.0e-7);
        let step = exp(-extinction * span);
        // The light scattered within the cell, dimmed by the mist in front of it.
        light += through * cell.rgb * (1.0 - step) / extinction;
        through *= step;
        textureStore(integrated, vec3<u32>(id.xy, s), vec4<f32>(light, through));
        near = far;
    }
}
