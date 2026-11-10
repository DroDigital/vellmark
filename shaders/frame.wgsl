// Values shared by every pass of one view.

struct ViewUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    // xyz: the eye in world space. w: seconds on the fixed clock.
    eye_time: vec4<f32>,
    // xyz: direction towards the sun. w: hours since midnight.
    sun_dir_hour: vec4<f32>,
    // x: 1 when the glyph layer is drawn, 0 when it is off. yzw: unused.
    switches: vec4<f32>,
}
