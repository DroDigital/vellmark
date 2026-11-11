// Depth only, from the light. Used for each cascade of the sun's shadow.

struct CascadeUniform {
    view_proj: mat4x4<f32>,
    // Bit i is set if material i gives off light. Such surfaces cast no shadow: a
    // lantern's glass must not shut its own flame in.
    glowing: vec4<u32>,
}

struct ObjectUniform {
    model: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> cascade: CascadeUniform;
@group(1) @binding(0) var<uniform> object: ObjectUniform;

@vertex
fn vs(@location(0) position: vec3<f32>, @location(4) material: u32) -> @builtin(position) vec4<f32> {
    if (cascade.glowing.x & (1u << material)) != 0u {
        // Outside the clip volume: the triangle is thrown away.
        return vec4<f32>(2.0, 2.0, 2.0, 1.0);
    }
    return cascade.view_proj * (object.model * vec4<f32>(position, 1.0));
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
