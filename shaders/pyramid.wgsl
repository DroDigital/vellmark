// The depth pyramid: the depth pre-pass at half resolution, then each level half the
// last, every texel the farthest of the four beneath it. Depth is reversed, so the
// farthest is the least. Culling reads it in the next frame.

@group(0) @binding(0) var scene_depth: texture_depth_multisampled_2d;
@group(0) @binding(1) var first_level: texture_storage_2d<r32float, write>;

@group(1) @binding(0) var above: texture_2d<f32>;
@group(1) @binding(1) var below: texture_storage_2d<r32float, write>;

@compute @workgroup_size(8, 8, 1)
fn first(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(first_level);
    if any(id.xy >= size) {
        return;
    }
    let limit = vec2<i32>(textureDimensions(scene_depth)) - 1;
    let p = vec2<i32>(id.xy) * 2;
    let d = min(
        min(textureLoad(scene_depth, min(p, limit), 0), textureLoad(scene_depth, min(p + vec2<i32>(1, 0), limit), 0)),
        min(textureLoad(scene_depth, min(p + vec2<i32>(0, 1), limit), 0), textureLoad(scene_depth, min(p + 1, limit), 0)),
    );
    textureStore(first_level, id.xy, vec4<f32>(d));
}

@compute @workgroup_size(8, 8, 1)
fn down(@builtin(global_invocation_id) id: vec3<u32>) {
    let size = textureDimensions(below);
    if any(id.xy >= size) {
        return;
    }
    let limit = vec2<i32>(textureDimensions(above)) - 1;
    let p = vec2<i32>(id.xy) * 2;
    // Three across and down where the level above is odd, so nothing at its edge is lost.
    let reach = vec2<i32>(select(1, 2, (limit.x + 1) % 2 == 1 && p.x + 2 == limit.x), select(1, 2, (limit.y + 1) % 2 == 1 && p.y + 2 == limit.y));
    var d = 1.0;
    for (var y = 0; y <= reach.y; y++) {
        for (var x = 0; x <= reach.x; x++) {
            d = min(d, textureLoad(above, min(p + vec2<i32>(x, y), limit), 0).r);
        }
    }
    textureStore(below, id.xy, vec4<f32>(d));
}
