// Depth only, from the light. Used for each cascade of the sun's shadow.

#import leaves

struct CascadeUniform {
    view_proj: mat4x4<f32>,
    // Bit i is set if material i gives off light. Such surfaces cast no shadow: a
    // lantern's glass must not shut its own flame in.
    glowing: vec4<u32>,
}

struct Instance {
    model: mat4x4<f32>,
    // x: heat, -1 (frozen) to 1 (as hot as it gets). y: 1 if it is not drawn (broken).
    state: vec4<f32>,
    // The sphere that holds it, in the world.
    bounds: vec4<f32>,
    // x: its batch. y: the batch's first instance.
    group: vec4<u32>,
}

struct Batch {
    // x: where this batch starts in the list being drawn. z: the leaf cards' material,
    // from 1; 0 if there is none.
    base: vec4<u32>,
}


@group(0) @binding(0) var<uniform> cascade: CascadeUniform;
@group(1) @binding(0) var<storage, read> instances: array<Instance>;
@group(1) @binding(1) var<storage, read> visible: array<u32>;
@group(1) @binding(2) var<uniform> batch: Batch;

@vertex
fn vs(@location(0) position: vec3<f32>, @location(4) material: u32, @builtin(instance_index) index: u32) -> @builtin(position) vec4<f32> {
    return from_light(position, material, index);
}

fn from_light(position: vec3<f32>, material: u32, index: u32) -> vec4<f32> {
    let object = instances[visible[batch.base.x + index]];
    // Nor do the hands in first person, nor an arrow (state.y below zero).
    if (cascade.glowing.x & (1u << material)) != 0u || object.state.y > 0.5 || object.state.y < -0.5 {
        // Outside the clip volume: the triangle is thrown away. A lantern's glass casts no
        // shadow, and nor does a broken thing.
        return vec4<f32>(2.0, 2.0, 2.0, 1.0);
    }
    return cascade.view_proj * (object.model * vec4<f32>(position, 1.0));
}

// A tree from the light: its leaf cards cast the shadows of their leaves, not of their
// squares.
struct TreeShadow {
    @builtin(position) position: vec4<f32>,
    @location(0) anchor: vec2<f32>,
    @location(1) @interpolate(flat) side: f32,
    @location(2) @interpolate(flat) material: u32,
}

@vertex
fn vs_tree(@location(0) position: vec3<f32>, @location(3) anchor: vec2<f32>, @location(4) material: u32, @location(6) side: f32, @builtin(instance_index) index: u32) -> TreeShadow {
    var out: TreeShadow;
    out.position = from_light(position, material, index);
    out.anchor = anchor;
    out.side = side;
    out.material = material;
    return out;
}

@fragment
fn fs_tree(in: TreeShadow) {
    if in.material + 1u == batch.base.z && leaf_at(in.anchor, in.side) >= 1.0 {
        discard;
    }
}

// A loose letter, from the light. Its shadow is its square drawn smaller, about the
// share of the square the character's ink covers: the shadow maps are too coarse to
// carry a character's shape, and a full square would shade like a tile.
fn turn(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    return v + 2.0 * cross(q.xyz, cross(q.xyz, v) + q.w * v);
}

// Half the side of a loose letter, in metres. Matches `LETTER_HALF` in the rules.
const LETTER_HALF: f32 = 0.0064;
const INK_SHARE: f32 = 0.62;

@vertex
fn vs_letter(@builtin(vertex_index) vertex: u32, @location(0) position: vec3<f32>, @location(1) rotation: vec4<f32>) -> @builtin(position) vec4<f32> {
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0),
    );
    let world = position + turn(rotation, vec3<f32>(corners[vertex] * LETTER_HALF * INK_SHARE, 0.0));
    return cascade.view_proj * vec4<f32>(world, 1.0);
}

// ---- the terrain, from the light ----

struct TerrainTable {
    materials: array<vec4<u32>, 3>,
    tiles: array<vec4<f32>, 384>,
}

@group(2) @binding(0) var<uniform> terrain: TerrainTable;
@group(2) @binding(1) var terrain_height: texture_2d_array<f32>;

struct TerrainShadow {
    @builtin(position) position: vec4<f32>,
    @location(0) world_xz: vec2<f32>,
    @location(1) @interpolate(flat) hole: f32,
}

@vertex
fn vs_terrain(@builtin(vertex_index) vertex: u32, @builtin(instance_index) tile: u32) -> TerrainShadow {
    let t = terrain.tiles[tile * 2u];
    let i = i32(vertex % 65u);
    let j = i32(vertex / 65u);
    let h = textureLoad(terrain_height, vec2<i32>(i + 1, j + 1), i32(t.w), 0).r;
    let world = vec3<f32>(t.x + f32(i) * t.z, h, t.y + f32(j) * t.z);
    var out: TerrainShadow;
    out.position = cascade.view_proj * vec4<f32>(world, 1.0);
    out.world_xz = world.xz;
    out.hole = terrain.tiles[tile * 2u + 1u].x;
    return out;
}

@fragment
fn fs_terrain(in: TerrainShadow) {
    if in.hole > 0.0 && max(abs(in.world_xz.x), abs(in.world_xz.y)) < in.hole - 0.01 {
        discard;
    }
}
