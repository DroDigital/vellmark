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
