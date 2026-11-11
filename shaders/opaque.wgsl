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
@group(0) @binding(3) var<uniform> lights: Lights;
@group(0) @binding(4) var light_clusters: texture_2d_array<u32>;
@group(0) @binding(5) var light_shadow: texture_depth_cube;

struct PointLight {
    position_radius: vec4<f32>,
    // rgb: colour times strength. w: 1 if this light's shadow is in the cube.
    colour_shadow: vec4<f32>,
}

struct Lights {
    count: vec4<u32>,
    lights: array<PointLight, 64>,
}

const CLUSTERS: vec3<u32> = vec3<u32>(16u, 9u, 16u);
const CLUSTER_NEAR: f32 = 0.1;
const CLUSTER_FAR: f32 = 120.0;
const CUBE_NEAR: f32 = 0.04;

@group(1) @binding(0) var material_colour: texture_2d_array<f32>;
@group(1) @binding(1) var material_normal: texture_2d_array<f32>;
@group(1) @binding(2) var material_sampler: sampler;
@group(1) @binding(3) var<uniform> materials: MaterialTable;
@group(1) @binding(4) var glyph_atlas: texture_2d_array<f32>;
@group(1) @binding(5) var glyph_sampler: sampler;
@group(1) @binding(6) var glyph_tone: texture_2d<f32>;
@group(1) @binding(7) var<uniform> pages: PageTable;
@group(1) @binding(8) var page_sheet: texture_2d<u32>;

@group(2) @binding(0) var<uniform> object: ObjectUniform;

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) anchor: vec2<f32>,
    @location(4) material: u32,
    @location(5) page: u32,
    @location(6) thickness: f32,
}

struct Varyings {
    @builtin(position) @invariant position: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) anchor: vec2<f32>,
    @location(4) @interpolate(flat) material: u32,
    @location(5) @interpolate(flat) page: u32,
    @location(6) @interpolate(flat) thickness: f32,
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
    out.page = in.page;
    out.thickness = in.thickness;
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

// How much of a point light reaches a point, through its shadow cube.
fn light_shadow_at(from_light: vec3<f32>, radius: f32, normal: vec3<f32>) -> f32 {
    // Step off the surface, then compare depths along the cube face's own axis.
    let d = from_light + normal * 0.02;
    let a = abs(d);
    let m = max(a.x, max(a.y, a.z)) * 0.985 - 0.012;
    let depth = radius * (m - CUBE_NEAR) / (max(m, 1.0e-4) * (radius - CUBE_NEAR));
    // Four taps a little apart: a soft edge that widens with distance.
    let side = normalize(cross(d, vec3<f32>(0.3, 1.0, 0.2))) * length(d) * 0.006;
    let up = normalize(cross(d, side)) * length(d) * 0.006;
    var lit = 0.0;
    lit += textureSampleCompareLevel(light_shadow, shadow_sampler, d + side + up, depth);
    lit += textureSampleCompareLevel(light_shadow, shadow_sampler, d - side + up, depth);
    lit += textureSampleCompareLevel(light_shadow, shadow_sampler, d + side - up, depth);
    lit += textureSampleCompareLevel(light_shadow, shadow_sampler, d - side - up, depth);
    return lit * 0.25;
}

// The light of every fire and lantern that reaches this pixel's cluster.
fn shade_point_lights(surface: Surface, world: vec3<f32>, flat_normal: vec3<f32>, to_eye: vec3<f32>, pixel: vec2<f32>, depth_in_view: f32) -> vec3<f32> {
    let cell = vec2<u32>(clamp(pixel / view.viewport.xy, vec2<f32>(0.0), vec2<f32>(0.9999)) * vec2<f32>(CLUSTERS.xy));
    let t = log(max(depth_in_view, CLUSTER_NEAR) / CLUSTER_NEAR) / log(CLUSTER_FAR / CLUSTER_NEAR);
    let slice = min(u32(t * f32(CLUSTERS.z)), CLUSTERS.z - 1u);
    let bits = textureLoad(light_clusters, vec2<u32>(slice * CLUSTERS.x + cell.x, cell.y), view.indices.x, 0).xy;
    var total = vec3<f32>(0.0);
    for (var word = 0u; word < 2u; word++) {
        var remaining = bits[word];
        while remaining != 0u {
            let bit = firstTrailingBit(remaining);
            remaining &= remaining - 1u;
            let light = lights.lights[word * 32u + bit];
            let offset = light.position_radius.xyz - world;
            let distance = length(offset);
            let radius = light.position_radius.w;
            if distance >= radius {
                continue;
            }
            // Inverse square, eased to nothing at the light's radius.
            let window = pow(clamp(1.0 - pow(distance / radius, 4.0), 0.0, 1.0), 2.0);
            var radiance = light.colour_shadow.rgb * view.effects.y * window / (distance * distance + 0.02);
            if light.colour_shadow.w > 0.5 {
                radiance *= light_shadow_at(-offset, radius, flat_normal);
            }
            total += shade_light(surface, to_eye, offset / distance, radiance);
        }
    }
    return total;
}

// All the light that reaches a surface: the sun or moon, the sky, every fire and
// lantern, and its own glow.
fn light_surface(surface: Surface, in: Varyings, flat_normal: vec3<f32>, glow: f32) -> vec3<f32> {
    let to_eye = normalize(view.eye_time.xyz - in.world);
    let depth_in_view = -(view.view * vec4<f32>(in.world, 1.0)).z;
    let shadow = sun_shadow(in.world, flat_normal, depth_in_view);
    var colour = shade_light(surface, to_eye, view.light_dir_hour.xyz, view.light_colour.rgb) * shadow;
    colour += shade_sky(surface, to_eye, view.sky_zenith.rgb, view.sky_horizon.rgb);
    colour += shade_point_lights(surface, in.world, flat_normal, to_eye, in.position.xy - view.viewport_origin.xy, depth_in_view);
    colour += surface.albedo * glow * view.effects.y;
    return colour;
}

// The colours of the text frame: warm paper and brown-black ink, each taking a little
// of the surface's own colour so that stone, timber, and cloth can still be told apart.
fn paper_of(albedo: vec3<f32>) -> vec3<f32> {
    let hue = albedo / max(max(albedo.r, max(albedo.g, albedo.b)), 1.0e-3);
    return vec3<f32>(0.50, 0.46, 0.37) * mix(vec3<f32>(1.0), hue, 0.40);
}

fn ink_of(albedo: vec3<f32>) -> vec3<f32> {
    return vec3<f32>(0.012, 0.010, 0.009) + albedo * 0.03;
}

@fragment
fn fs(in: Varyings) -> @location(0) vec4<f32> {
    let m = materials.entries[in.material];
    let uv = vec2<f32>(in.anchor.x, -in.anchor.y) / m.surface.x;
    let colour_texel = textureSample(material_colour, material_sampler, uv, in.material);
    let normal_texel = textureSample(material_normal, material_sampler, uv, in.material);
    let footprint = AnchorFootprint(dpdx(in.anchor), dpdy(in.anchor));
    let world = WorldFootprint(in.world, dpdx(in.world), dpdy(in.world));

    let n = normalize(in.normal);
    let t = normalize(in.tangent.xyz - n * dot(n, in.tangent.xyz));
    let b = cross(n, t) * in.tangent.w;
    let map_normal = vec2<f32>(normal_texel.x, normal_texel.y) * 2.0 - 1.0;
    let reading = view.switches.y;

    // In close reading the light is needed first, because it becomes the ink's weight.
    var light = 0.0;
    var plain_lit = vec3<f32>(0.0);
    if reading > 0.0 {
        let z0 = sqrt(max(1.0 - dot(map_normal, map_normal), 0.05));
        let plain = Surface(colour_texel.rgb, normalize(t * map_normal.x + b * map_normal.y + n * z0), normal_texel.z, colour_texel.a);
        plain_lit = light_surface(plain, in, n, m.surface.w);
        let seen = plain_lit * view.viewport.z;
        light = clamp(dot(seen / (seen + vec3<f32>(0.6)), vec3<f32>(0.2126, 0.7152, 0.0722)) * 1.6, 0.0, 1.0);
    }

    // The fine print. Beyond the reveal distance this returns at once, having sampled
    // nothing, and the surface is exactly what it would be with the layer off.
    var glyphs = GlyphResult(1.0, vec2<f32>(0.0), 0.0, 0.0);
    if view.switches.x > 0.5 {
        glyphs = glyph_surface(in.anchor, footprint, in.material, in.page, colour_texel.rgb, light);
    }

    if view.viewport.w > 0.5 {
        // A tool's view: white where the fine print is drawn, black beyond the reveal
        // distance. The capture tool uses it to know where two frames must match.
        return vec4<f32>(vec3<f32>(glyphs.revealed * 100.0), 1.0);
    }

    var colour: vec3<f32>;
    if reading >= 1.0 {
        colour = plain_lit;
    } else {
        let tangent_normal = map_normal + glyphs.relief;
        let z = sqrt(max(1.0 - dot(tangent_normal, tangent_normal), 0.05));
        let normal = normalize(t * tangent_normal.x + b * tangent_normal.y + n * z);
        let surface = Surface(colour_texel.rgb * glyphs.tone, normal, normal_texel.z, colour_texel.a);
        colour = light_surface(surface, in, n, m.surface.w);
        // The depth looks: up close, the surface opens onto the writing behind it.
        let look = u32(view.effects.z + 0.5);
        if look > 0u && glyphs.revealed > 0.0 && m.surface.w == 0.0 {
            let to_eye = normalize(view.eye_time.xyz - in.world);
            let in_frame = vec3<f32>(dot(to_eye, t), dot(to_eye, b), dot(to_eye, n));
            colour = glyph_depth(colour, in.anchor, footprint, in_frame, world, -to_eye, length(view.eye_time.xyz - in.world), in.material, in.page, in.thickness, glyphs.ink, look);
        }
    }
    if reading > 0.0 {
        // The text frame: colour drains to paper and ink, and light is only weight.
        // Divided by exposure so that paper is the same paper at noon and at night.
        let text = mix(paper_of(colour_texel.rgb), ink_of(colour_texel.rgb), glyphs.ink) / view.viewport.z;
        colour = mix(colour, text, reading);
    }
    // Alpha carries where the letters are, so that the edge filter leaves them crisp.
    return vec4<f32>(colour, glyphs.revealed);
}

// ---- loose letters: what a broken thing comes apart into ----

struct LetterInput {
    @builtin(vertex_index) vertex: u32,
    @location(0) position: vec3<f32>,
    @location(1) rotation: vec4<f32>,
    @location(2) material: u32,
    @location(3) seed: u32,
}

struct LetterVaryings {
    @builtin(position) position: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) uv: vec2<f32>,
    @location(3) @interpolate(flat) material: u32,
    @location(4) @interpolate(flat) seed: u32,
}

// Half the side of a loose letter, in metres. Matches `LETTER_HALF` in the rules.
const LETTER_HALF: f32 = 0.0064;

fn turn(q: vec4<f32>, v: vec3<f32>) -> vec3<f32> {
    return v + 2.0 * cross(q.xyz, cross(q.xyz, v) + q.w * v);
}

@vertex
fn vs_letter(in: LetterInput) -> LetterVaryings {
    let corners = array<vec2<f32>, 6>(
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, -1.0), vec2<f32>(1.0, 1.0),
        vec2<f32>(-1.0, -1.0), vec2<f32>(1.0, 1.0), vec2<f32>(-1.0, 1.0),
    );
    let corner = corners[in.vertex];
    let world = in.position + turn(in.rotation, vec3<f32>(corner * LETTER_HALF, 0.0));
    var out: LetterVaryings;
    out.position = view.view_proj * vec4<f32>(world, 1.0);
    out.world = world;
    out.normal = turn(in.rotation, vec3<f32>(0.0, 0.0, 1.0));
    out.uv = vec2<f32>(corner.x, -corner.y) * 0.5 + 0.5;
    out.material = in.material;
    out.seed = in.seed;
    return out;
}

@fragment
fn fs_letter(in: LetterVaryings) -> @location(0) vec4<f32> {
    // How tall the letter is on screen, in pixels.
    let du = dpdx(in.uv);
    let dv = dpdy(in.uv);
    let h = 1.0 / max(max(length(du), length(dv)), 1.0e-5);
    // Too small to read, it is a chip of its material and no letter shows: the fine
    // print is never seen from afar, loose or not.
    let legible = smoothstep(4.0, 7.0, h);
    let character = family_character(in.material, in.seed);
    let glyph = cell_glyph(character - 32u, in.uv, du, dv, h, 0.6);
    let cover = mix(0.6, glyph.cover, legible);
    if cover < 0.04 {
        discard;
    }
    if view.viewport.w > 0.5 {
        return vec4<f32>(vec3<f32>(legible * 100.0), cover);
    }

    // The colour of the thing it was part of, taken broadly and deepened: a loose
    // letter is ink without its page, and must read against whatever it lies on.
    let place = vec2<f32>(f32(in.seed & 0xffu), f32((in.seed >> 8u) & 0xffu)) / 255.0;
    let colour_texel = textureSampleLevel(material_colour, material_sampler, place, in.material, 4.0);
    let normal_texel = textureSampleLevel(material_normal, material_sampler, place, in.material, 4.0);
    let to_eye = view.eye_time.xyz - in.world;
    var n = normalize(in.normal);
    if dot(n, to_eye) < 0.0 {
        n = -n;
    }
    let surface = Surface(colour_texel.rgb * colour_texel.rgb * 0.55, n, normal_texel.z, colour_texel.a);
    let as_surface = Varyings(in.position, in.world, n, vec4<f32>(1.0, 0.0, 0.0, 1.0), vec2<f32>(0.0), in.material, 0u, 0.0);
    let colour = light_surface(surface, as_surface, n, 0.0);
    return vec4<f32>(colour, cover);
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
    var colour = sky_colour(dir, view);
    if view.switches.y > 0.0 {
        // In close reading the sky is bare paper, a little darker towards the zenith.
        let paper = vec3<f32>(0.50, 0.46, 0.37) * (1.0 - 0.25 * clamp(dir.y, 0.0, 1.0)) / view.viewport.z;
        colour = mix(colour, paper, view.switches.y);
    }
    return vec4<f32>(colour, 0.0);
}
