// Culling: each view's own list of what it can see. Every instance is tested against
// the view's frustum and then against the depth pyramid of the last frame: if the box
// around it lies wholly behind what was drawn there, it is hidden. Those that pass are
// written into the view's list, batch by batch, and counted into that batch's indirect
// draw. After a camera cut the pyramid is not trusted and only the frustum is tested.

struct Instance {
    model: mat4x4<f32>,
    state: vec4<f32>,
    bounds: vec4<f32>,
    // x: its batch. y: where the batch starts in a list. z: how many levels of detail
    // (1 for none) in the low byte, and above it how many instances the batch has. w:
    // where its first coarser level's batch starts in a list.
    group: vec4<u32>,
}

struct Cull {
    // This frame's view and projection, for the frustum.
    view_proj: mat4x4<f32>,
    // Last frame's, for the pyramid.
    previous: mat4x4<f32>,
    // The view on the target: its top left and size, in pixels.
    viewport: vec4<f32>,
    // xy: the pyramid's first level's size. z: its levels. w: 1 to test against it.
    pyramid: vec4<f32>,
    // x: instances. y: where this view's list starts. z: where its draws start.
    counts: vec4<u32>,
    // xyz: the eye, for choosing levels of detail.
    eye: vec4<f32>,
}

#import lod

@group(0) @binding(0) var<storage, read> instances: array<Instance>;
@group(0) @binding(1) var<storage, read_write> visible: array<u32>;
@group(0) @binding(2) var<storage, read_write> draws: array<atomic<u32>>;
@group(0) @binding(3) var<uniform> cull: Cull;
@group(0) @binding(4) var pyramid: texture_2d<f32>;

fn row(m: mat4x4<f32>, i: u32) -> vec4<f32> {
    return vec4<f32>(m[0][i], m[1][i], m[2][i], m[3][i]);
}

fn outside_frustum(centre: vec3<f32>, radius: f32) -> bool {
    let m = cull.view_proj;
    let w = row(m, 3u);
    // Left, right, bottom, top, and near. The far plane is at infinity.
    let planes = array<vec4<f32>, 5>(w + row(m, 0u), w - row(m, 0u), w + row(m, 1u), w - row(m, 1u), w - row(m, 2u));
    for (var k = 0; k < 5; k++) {
        let p = planes[k];
        if (dot(p.xyz, centre) + p.w) / length(p.xyz) < -radius {
            return true;
        }
    }
    return false;
}

fn hidden_behind(centre: vec3<f32>, radius: f32) -> bool {
    if cull.pyramid.w < 0.5 {
        return false;
    }
    var lo = vec2<f32>(1.0e9);
    var hi = vec2<f32>(-1.0e9);
    var nearest = 0.0;
    for (var c = 0u; c < 8u; c++) {
        let corner = centre + vec3<f32>(select(-radius, radius, (c & 1u) != 0u), select(-radius, radius, (c & 2u) != 0u), select(-radius, radius, (c & 4u) != 0u));
        let clip = cull.previous * vec4<f32>(corner, 1.0);
        if clip.w <= 1.0e-4 {
            // Part of it was behind the eye: it may be anywhere on screen.
            return false;
        }
        let ndc = clip.xyz / clip.w;
        lo = min(lo, ndc.xy);
        hi = max(hi, ndc.xy);
        // Reversed depth: nearer is greater.
        nearest = max(nearest, ndc.z);
    }
    // To the pyramid's first level, which is half the target's resolution.
    let to_pixels = vec2<f32>(0.5, -0.5) * cull.viewport.zw;
    let a = (cull.viewport.xy + (vec2<f32>(lo.x, hi.y) * to_pixels + cull.viewport.zw * 0.5)) * 0.5;
    let b = (cull.viewport.xy + (vec2<f32>(hi.x, lo.y) * to_pixels + cull.viewport.zw * 0.5)) * 0.5;
    let size = cull.pyramid.xy;
    let p0 = clamp(min(a, b), vec2<f32>(0.0), size - 1.0);
    let p1 = clamp(max(a, b), vec2<f32>(0.0), size - 1.0);
    let span = max(p1.x - p0.x, p1.y - p0.y);
    let level = clamp(ceil(log2(max(span, 1.0))), 0.0, cull.pyramid.z - 1.0);
    let scale = exp2(level);
    let t0 = vec2<i32>(floor(p0 / scale));
    let t1 = vec2<i32>(floor(p1 / scale));
    let l = i32(level);
    // The farthest of what was drawn over the box: the least depth, reversed.
    let farthest = min(
        min(textureLoad(pyramid, t0, l).r, textureLoad(pyramid, vec2<i32>(t1.x, t0.y), l).r),
        min(textureLoad(pyramid, vec2<i32>(t0.x, t1.y), l).r, textureLoad(pyramid, t1, l).r),
    );
    return nearest < farthest;
}

@compute @workgroup_size(64, 1, 1)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= cull.counts.x {
        return;
    }
    let object = instances[i];
    if object.state.y > 0.5 {
        return;
    }
    let centre = object.bounds.xyz;
    let radius = object.bounds.w;
    if outside_frustum(centre, radius) || hidden_behind(centre, radius) {
        return;
    }
    // A thing with levels of detail goes to the level it is seen at, and, while it is
    // crossfading, to the next as well. Each level's batch has its own part of the list.
    let levels = max(object.group.z & 0xffu, 1u);
    var level = 0u;
    var both = false;
    if levels > 1u {
        let chosen = lod_of(radius, distance(cull.eye.xyz, centre), levels);
        level = u32(chosen.x);
        both = chosen.y > 0.0;
    }
    emit(i, object, level);
    if both {
        emit(i, object, level + 1u);
    }
}

fn emit(i: u32, object: Instance, level: u32) {
    let group = object.group.x + level;
    let slot = atomicAdd(&draws[cull.counts.z + group * 5u + 1u], 1u);
    // The finest level's batch is where the thing's own place is; the coarser follow
    // one another after every placed thing's.
    var base = object.group.y;
    if level > 0u {
        base = object.group.w + (level - 1u) * (object.group.z >> 8u);
    }
    visible[cull.counts.y + base + slot] = i;
}
