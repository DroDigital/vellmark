// Every opaque surface: the depth pre-pass, the lit pass, and the sky behind them.

#import frame
#import shading
#import glyph

struct ObjectUniform {
    model: mat4x4<f32>,
}

@group(0) @binding(0) var<uniform> view: ViewUniform;
@group(0) @binding(1) var shadow_map: texture_depth_2d_array;
@group(0) @binding(2) var shadow_sampler: sampler_comparison;

@group(1) @binding(0) var material_colour: texture_2d_array<f32>;
@group(1) @binding(1) var material_normal: texture_2d_array<f32>;
@group(1) @binding(2) var material_sampler: sampler;
@group(1) @binding(3) var<uniform> materials: MaterialTable;
@group(1) @binding(4) var glyph_atlas: texture_2d_array<f32>;
@group(1) @binding(5) var glyph_sampler: sampler;
@group(1) @binding(6) var glyph_tone: texture_2d<f32>;

@group(2) @binding(0) var<uniform> object: ObjectUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) anchor: vec2<f32>,
    @location(4) material: u32,
}

struct Varyings {
    @builtin(position) @invariant position: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) anchor: vec2<f32>,
    @location(4) @interpolate(flat) material: u32,
}

@vertex
fn vs_depth(in: VertexInput) -> @builtin(position) @invariant vec4<f32> {
    let world = object.model * vec4<f32>(in.position, 1.0);
    return view.view_proj * world;
}

@vertex
fn vs(in: VertexInput) -> Varyings {
    let world = object.model * vec4<f32>(in.position, 1.0);
    var out: Varyings;
    out.position = view.view_proj * world;
    out.world = world.xyz;
    out.normal = (object.model * vec4<f32>(in.normal, 0.0)).xyz;
    out.tangent = vec4<f32>((object.model * vec4<f32>(in.tangent.xyz, 0.0)).xyz, in.tangent.w);
    out.anchor = in.anchor;
    out.material = in.material;
    return out;
}

// How much of the main light reaches a point: 0 in shadow, 1 in light, soft between.
fn sun_shadow(world: vec3<f32>, normal: vec3<f32>, depth_in_view: f32) -> f32 {
    var cascade = 2u;
    if depth_in_view < view.cascade_far.x {
        cascade = 0u;
    } else if depth_in_view < view.cascade_far.y {
        cascade = 1u;
    }
    if depth_in_view > view.cascade_far.z {
        return 1.0;
    }
    // Step off the surface along its normal, further in the coarser cascades, so that
    // a surface never shadows itself.
    let texel = view.cascade_far.w * f32(1u << (cascade * 2u));
    let p = view.shadow[cascade] * vec4<f32>(world + normal * texel * 2.0, 1.0);
    let uv = p.xy * vec2<f32>(0.5, -0.5) + 0.5;
    if any(uv < vec2<f32>(0.0)) || any(uv > vec2<f32>(1.0)) {
        return 1.0;
    }
    let size = vec2<f32>(textureDimensions(shadow_map));
    var lit = 0.0;
    // Nine filtered taps: a soft edge about three texels wide.
    for (var y = -1; y <= 1; y++) {
        for (var x = -1; x <= 1; x++) {
            let offset = vec2<f32>(f32(x), f32(y)) * 1.2 / size;
            lit += textureSampleCompareLevel(shadow_map, shadow_sampler, uv + offset, cascade, p.z);
        }
    }
    return lit / 9.0;
}

@fragment
fn fs(in: Varyings) -> @location(0) vec4<f32> {
    let m = materials.entries[in.material];
    let uv = vec2<f32>(in.anchor.x, -in.anchor.y) / m.surface.x;
    let colour_texel = textureSample(material_colour, material_sampler, uv, in.material);
    let normal_texel = textureSample(material_normal, material_sampler, uv, in.material);

    // The fine print. Beyond the reveal distance this returns at once, having sampled
    // nothing, and the surface is exactly what it would be with the layer off.
    let footprint = AnchorFootprint(dpdx(in.anchor), dpdy(in.anchor));
    var glyphs = GlyphResult(1.0, vec2<f32>(0.0), 0.0, 0.0);
    if view.switches.x > 0.5 {
        glyphs = glyph_surface(in.anchor, footprint, in.material, colour_texel.rgb);
    }

    if view.viewport.w > 0.5 {
        // A tool's view: white where the fine print is drawn, black beyond the reveal
        // distance. The capture tool uses it to know where two frames must match.
        return vec4<f32>(vec3<f32>(glyphs.revealed * 100.0), 1.0);
    }

    let n = normalize(in.normal);
    let t = normalize(in.tangent.xyz - n * dot(n, in.tangent.xyz));
    let b = cross(n, t) * in.tangent.w;
    var tangent_normal = vec2<f32>(normal_texel.x, normal_texel.y) * 2.0 - 1.0;
    tangent_normal += glyphs.relief;
    let z = sqrt(max(1.0 - dot(tangent_normal, tangent_normal), 0.05));
    let normal = normalize(t * tangent_normal.x + b * tangent_normal.y + n * z);

    let surface = Surface(colour_texel.rgb * glyphs.tone, normal, normal_texel.z, colour_texel.a);
    let to_eye = normalize(view.eye_time.xyz - in.world);
    let depth_in_view = -(view.view * vec4<f32>(in.world, 1.0)).z;

    let shadow = sun_shadow(in.world, n, depth_in_view);
    var colour = shade_light(surface, to_eye, view.light_dir_hour.xyz, view.light_colour.rgb) * shadow;
    colour += shade_sky(surface, to_eye, view.sky_zenith.rgb, view.sky_horizon.rgb);
    colour += surface.albedo * m.surface.w;
    return vec4<f32>(colour, 1.0);
}

// ---- the sky, drawn behind everything from one triangle ----

struct SkyVaryings {
    @builtin(position) position: vec4<f32>,
    @location(0) ndc: vec2<f32>,
}

@vertex
fn vs_sky(@builtin(vertex_index) index: u32) -> SkyVaryings {
    let corner = vec2<f32>(f32((index << 1u) & 2u), f32(index & 2u));
    var out: SkyVaryings;
    out.ndc = corner * 2.0 - 1.0;
    // Depth 0 is infinitely far with reversed depth.
    out.position = vec4<f32>(out.ndc, 0.0, 1.0);
    return out;
}

@fragment
fn fs_sky(in: SkyVaryings) -> @location(0) vec4<f32> {
    let far = view.inv_view_proj * vec4<f32>(in.ndc, 0.5, 1.0);
    let dir = normalize(far.xyz / far.w - view.eye_time.xyz);
    if view.viewport.w > 0.5 {
        return vec4<f32>(0.0, 0.0, 0.0, 1.0);
    }
    return vec4<f32>(sky_colour(dir, view), 1.0);
}
