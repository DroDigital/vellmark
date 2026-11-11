// Values shared by every pass of one view.

struct ViewUniform {
    view_proj: mat4x4<f32>,
    inv_view_proj: mat4x4<f32>,
    view: mat4x4<f32>,
    // xyz: the eye in world space. w: seconds on the fixed clock.
    eye_time: vec4<f32>,
    // xyz: direction towards the light that casts shadows, the sun by day and the moon
    // by night. w: hours since midnight.
    light_dir_hour: vec4<f32>,
    // rgb: that light's colour and strength.
    light_colour: vec4<f32>,
    // xyz: direction towards the sun itself, which may be below the horizon.
    sun_dir: vec4<f32>,
    // Light from the sky, straight up and at the horizon.
    sky_zenith: vec4<f32>,
    sky_horizon: vec4<f32>,
    // x: 1 when the glyph layer is drawn. y: close reading, 0 to 1.
    // z: letter contrast scale. w: reveal distance scale.
    switches: vec4<f32>,
    // xy: the view's size in pixels. z: exposure. w: 1 draws the reveal mask, for tools.
    viewport: vec4<f32>,
    // The far edge of each shadow cascade, as distance in front of the eye.
    cascade_far: vec4<f32>,
    shadow: array<mat4x4<f32>, 3>,
    // x: this view's index among the frame's views.
    indices: vec4<u32>,
    // xy: the top left of the view on the target, in pixels.
    viewport_origin: vec4<f32>,
    // x: the dive, 0 to 1: levels finer than the finest print are drawn.
    // y: how brightly lamps and glowing things burn, 0 to 1.
    // z: the depth look: 0 flat, 1 atoms, 2 lattice, 3 source.
    effects: vec4<f32>,
}

// The colour of the sky in a direction.
fn sky_colour(dir: vec3<f32>, view: ViewUniform) -> vec3<f32> {
    let sun = view.sun_dir.xyz;
    let up = clamp(dir.y, 0.0, 1.0);
    var colour = mix(view.sky_horizon.rgb, view.sky_zenith.rgb, pow(up, 0.45));
    // Warmth around a low sun, and the disc itself.
    let toward = clamp(dot(dir, sun), 0.0, 1.0);
    let low = clamp(1.0 - sun.y * 2.5, 0.0, 1.0) * clamp(sun.y * 8.0 + 1.0, 0.0, 1.0);
    colour += vec3<f32>(1.0, 0.42, 0.12) * pow(toward, 6.0) * low * 0.9 * length(view.sky_horizon.rgb);
    let disc = smoothstep(0.99975, 0.99992, toward) * clamp(sun.y * 20.0 + 1.0, 0.0, 1.0);
    colour += vec3<f32>(60.0, 50.0, 36.0) * disc;
    return colour;
}
