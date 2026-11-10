// Depth only, from the light. Used for each cascade of the sun's shadow.

struct CascadeUniform {
    view_proj: mat4x4<f32>,
}

struct ObjectUniform {
    model: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> cascade: CascadeUniform;
@group(1) @binding(0) var<uniform> object: ObjectUniform;

@vertex
fn vs(@location(0) position: vec3<f32>) -> @builtin(position) vec4<f32> {
    return cascade.view_proj * (object.model * vec4<f32>(position, 1.0));
}
