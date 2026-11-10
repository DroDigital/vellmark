// The interface: glyph cells and plain rectangles, in pixels, over the finished frame.

struct UiUniform {
    // xy: size of the target in pixels. z: the atlas's distance range in texels.
    // w: side of an atlas cell in texels.
    target_range_cell: vec4<f32>,
}

@group(0) @binding(0) var<uniform> ui: UiUniform;
@group(0) @binding(1) var atlas: texture_2d_array<f32>;
@group(0) @binding(2) var atlas_sampler: sampler;

struct Instance {
    @location(0) rect: vec4<f32>,
    @location(1) colour: vec4<f32>,
    @location(2) layer: u32,
}

struct Varyings {
    @builtin(position) position: vec4<f32>,
    @location(0) uv: vec2<f32>,
    @location(1) colour: vec4<f32>,
    @location(2) @interpolate(flat) layer: u32,
    // Screen pixels to an atlas texel.
    @location(3) @interpolate(flat) scale: f32,
}

@vertex
fn vs(@builtin(vertex_index) index: u32, instance: Instance) -> Varyings {
    let corner = vec2<f32>(f32(index & 1u), f32((index >> 1u) & 1u));
    let pixel = instance.rect.xy + corner * instance.rect.zw;
    var out: Varyings;
    out.position = vec4<f32>(
        pixel.x / ui.target_range_cell.x * 2.0 - 1.0,
        1.0 - pixel.y / ui.target_range_cell.y * 2.0,
        0.0,
        1.0,
    );
    out.uv = corner;
    out.colour = instance.colour;
    out.layer = instance.layer;
    out.scale = instance.rect.w / ui.target_range_cell.w;
    return out;
}

@fragment
fn fs(in: Varyings) -> @location(0) vec4<f32> {
    if in.layer == 0xffffffffu {
        return in.colour;
    }
    let texel = textureSample(atlas, atlas_sampler, in.uv, in.layer);
    let median = max(min(texel.r, texel.g), min(max(texel.r, texel.g), texel.b));
    // Small glyphs read the true distance, which survives being averaged down.
    // Large glyphs read the multi-channel distance, which keeps corners sharp.
    let cell_pixels = in.scale * ui.target_range_cell.w;
    let distance = mix(texel.a, median, smoothstep(20.0, 40.0, cell_pixels));
    let pixels = (distance - 0.5) * ui.target_range_cell.z * in.scale;
    let coverage = clamp(pixels + 0.5, 0.0, 1.0);
    return vec4<f32>(in.colour.rgb, in.colour.a * coverage);
}
