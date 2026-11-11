// Every opaque surface: the depth pre-pass, the lit pass, and the sky behind them.

#import frame
#import shading
#import glyph

struct ObjectUniform {
    model: mat4x4<f32>,
    // x: how hot or cold the thing is, -1 (frozen) to 1 (as hot as it gets).
    state: vec4<f32>,
}

@group(0) @binding(0) var<uniform> view: ViewUniform;
@group(0) @binding(1) var shadow_map: texture_depth_2d_array;
@group(0) @binding(2) var shadow_sampler: sampler_comparison;
@group(0) @binding(3) var<uniform> lights: Lights;
@group(0) @binding(4) var light_clusters: texture_2d_array<u32>;
@group(0) @binding(5) var light_shadow: texture_depth_cube;
@group(0) @binding(6) var fog_volume: texture_3d<f32>;
@group(0) @binding(7) var fog_sampler: sampler;

const FOG_NEAR: f32 = 0.5;
const FOG_FAR: f32 = 400.0;

// The mist between the eye and a place, from the fog volume: how much of the place's
// light gets through, and the mist's own light. `depth_in_view` is the distance in front
// of the eye; `pixel` is the place on the view, in pixels.
fn mist_between(depth_in_view: f32, pixel: vec2<f32>) -> vec4<f32> {
    let uv = clamp(pixel / view.viewport.xy, vec2<f32>(0.0), vec2<f32>(1.0));
    let slice = log(max(depth_in_view, FOG_NEAR) / FOG_NEAR) / log(FOG_FAR / FOG_NEAR);
    return textureSampleLevel(fog_volume, fog_sampler, vec3<f32>(uv, clamp(slice - 0.5 / 64.0, 0.0, 1.0)), 0.0);
}

// A surface's light as it reaches the eye: through the mist, then through the air, which
// takes blue light out and puts the sky's in, so that the far hills go blue.
fn through_air(colour: vec3<f32>, world: vec3<f32>, pixel: vec2<f32>) -> vec3<f32> {
    let to = world - view.eye_time.xyz;
    let distance = max(length(to), 1.0e-3);
    let depth_in_view = -(view.view * vec4<f32>(world, 1.0)).z;
    let mist = mist_between(depth_in_view, pixel);
    let misted = colour * mist.a + mist.rgb;
    let dir = to / distance;
    let height = max((world.y + view.eye_time.y) * 0.5, 0.0);
    let extinction = RAYLEIGH * exp(-height / 8000.0) + vec3<f32>(MIE_EXTINCTION * exp(-height / 1200.0));
    let through = exp(-extinction * distance);
    let glow = view.light_colour.rgb * 0.03 * pow(max(dot(dir, view.light_dir_hour.xyz), 0.0), 6.0);
    return misted * through + (view.sky_horizon.rgb + glow) * (1.0 - through);
}

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
    // What the ground sends back: the sky's light on it, and the sun's where it is lit.
    let ground = view.sky_horizon.rgb * 0.35 + view.light_colour.rgb * max(view.light_dir_hour.y, 0.0) * 0.06;
    colour += shade_sky(surface, to_eye, view.sky_zenith.rgb, view.sky_horizon.rgb, ground);
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
        glyphs = glyph_surface(in.anchor, footprint, in.material, in.page, colour_texel.rgb, light, object.state.x);
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
        // Heat. A thing hot enough glows, as hot iron does, at any distance: that is the
        // world, not its letters. Up close, the strokes of the letters saying so glow
        // first, as embers in the grain.
        let hot = max(object.state.x, 0.0);
        if hot > 0.0 {
            let flicker = 0.75 + 0.25 * sin(view.eye_time.w * 7.3 + dot(in.world, vec3<f32>(9.1, 7.7, 8.3)));
            let ember = vec3<f32>(1.0, 0.34, 0.06) * flicker / view.viewport.z;
            colour += ember * colour_texel.rgb * smoothstep(0.75, 1.0, hot) * 0.8;
            colour += ember * glyphs.ink * glyphs.revealed * hot * hot * 0.5;
        }
        // The depth looks: up close, the surface opens onto the writing behind it.
        let look = u32(view.effects.z + 0.5);
        if look > 0u && glyphs.revealed > 0.0 && m.surface.w == 0.0 {
            let to_eye = normalize(view.eye_time.xyz - in.world);
            let in_frame = vec3<f32>(dot(to_eye, t), dot(to_eye, b), dot(to_eye, n));
            let albedo = max(colour_texel.rgb * glyphs.tone, vec3<f32>(0.03));
            let win = Window(in.anchor, t, b, colour / albedo, object.state.x);
            colour = glyph_depth(colour, in.anchor, footprint, in_frame, world, -to_eye, length(view.eye_time.xyz - in.world), in.material, in.page, in.thickness, glyphs.ink, look, win);
        }
    }
    colour = through_air(colour, in.world, in.position.xy - view.viewport_origin.xy);
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
    let colour = through_air(light_surface(surface, as_surface, n, 0.0), in.world, in.position.xy - view.viewport_origin.xy);
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
    // How far apart two pixels are, in radians, for sharp stars and discs.
    let pixel = length(dpdx(dir)) + length(dpdy(dir)) * 0.0;
    var colour = sky_colour_sized(dir, view, max(pixel, 1.0e-5));
    // The moon: a lit sphere, its phase from where the sun is, and on its face the dark
    // seas faintly in the shape of an `@`, as the brief asks.
    let moon = view.moon_dir.xyz;
    let moon_radius = 0.0095;
    let on_moon = dot(dir, moon);
    if on_moon > cos(moon_radius * 1.2) && moon.y > -0.02 {
        // The moon's own right and up as the eye sees them, so that the `@` reads the
        // right way round: no letter is ever seen mirrored.
        let right = normalize(cross(moon, vec3<f32>(0.0, 1.0, 0.0)));
        let up = cross(right, moon);
        let q = vec2<f32>(dot(dir, right), dot(dir, up)) / moon_radius;
        let r2 = dot(q, q);
        if r2 < 1.0 {
            let z = sqrt(1.0 - r2);
            let surface = right * q.x + up * q.y - moon * z;
            let lit = smoothstep(-0.04, 0.12, dot(surface, view.sun_dir.xyz));
            let face = vec2<f32>(q.x, -q.y) * 0.5 + 0.5;
            let at = textureSampleLevel(glyph_atlas, glyph_sampler, face, 64u - 32u + GLYPHS_PER_FACE, 3.0).a;
            let sea = smoothstep(0.40, 0.60, at) * 0.2;
            let craters = sky_noise(face * 24.0) * 0.12 + sky_noise(face * 7.0) * 0.1;
            let albedo = vec3<f32>(0.92, 0.90, 0.86) * (1.0 - sea - craters);
            let shine = albedo * (lit * SUN * 0.11 + 0.0025) * sky_transmittance(SKY_EYE, max(moon.y, 0.0));
            let edge = 1.0 - smoothstep(0.96, 1.0, r2);
            // The moon is behind the cloud, as the sun is.
            let cloud = cloud_layer(dir, view, 1800.0, 2600.0, view.sky.x, vec2<f32>(0.004, 0.0015), 1.0).w;
            colour = mix(colour, colour * 0.2 + shine, edge * (1.0 - cloud));
        }
    }
    // The mist in front of the sky: the god rays of a low sun are seen against it.
    let mist = mist_between(FOG_FAR, in.position.xy - view.viewport_origin.xy);
    colour = colour * mist.a + mist.rgb;
    if view.switches.y > 0.0 {
        // In close reading the sky is bare paper, a little darker towards the zenith.
        let paper = vec3<f32>(0.50, 0.46, 0.37) * (1.0 - 0.25 * clamp(dir.y, 0.0, 1.0)) / view.viewport.z;
        colour = mix(colour, paper, view.switches.y);
    }
    return vec4<f32>(colour, 0.0);
}

// ---- the terrain ----

struct TerrainTable {
    // The material of each ground class, in the order of `Ground`, then rock.
    materials: array<vec4<u32>, 3>,
    // Two to a tile: its least corner x and z, its spacing, and its layer; then the
    // half-width of the finer ring it must not draw over.
    tiles: array<vec4<f32>, 384>,
}

@group(3) @binding(0) var<uniform> terrain: TerrainTable;
@group(3) @binding(1) var terrain_height: texture_2d_array<f32>;
@group(3) @binding(2) var terrain_ground: texture_2d_array<u32>;

const TERRAIN_SAMPLES: u32 = 65u;

struct TerrainVaryings {
    @builtin(position) @invariant position: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    // The place in the tile, in samples.
    @location(2) local: vec2<f32>,
    @location(3) @interpolate(flat) tile: u32,
}

// The height of a tile's sample; -1 and 65 reach into the border from its neighbours.
fn terrain_sample(i: i32, j: i32, layer: i32) -> f32 {
    return textureLoad(terrain_height, vec2<i32>(i + 1, j + 1), layer, 0).r;
}

fn terrain_material(slot: u32) -> u32 {
    return terrain.materials[slot / 4u][slot % 4u];
}

@vertex
fn vs_terrain(@builtin(vertex_index) vertex: u32, @builtin(instance_index) tile: u32) -> TerrainVaryings {
    let t = terrain.tiles[tile * 2u];
    let layer = i32(t.w);
    let i = i32(vertex % TERRAIN_SAMPLES);
    let j = i32(vertex / TERRAIN_SAMPLES);
    let h = terrain_sample(i, j, layer);
    let hx = (terrain_sample(i + 1, j, layer) - terrain_sample(i - 1, j, layer)) / (2.0 * t.z);
    let hz = (terrain_sample(i, j + 1, layer) - terrain_sample(i, j - 1, layer)) / (2.0 * t.z);
    var out: TerrainVaryings;
    out.world = vec3<f32>(t.x + f32(i) * t.z, h, t.y + f32(j) * t.z);
    out.normal = normalize(vec3<f32>(-hx, 1.0, -hz));
    out.local = vec2<f32>(f32(i), f32(j));
    out.tile = tile;
    out.position = view.view_proj * vec4<f32>(out.world, 1.0);
    return out;
}

// True where a finer ring draws this ground already.
fn terrain_cut(world: vec3<f32>, tile: u32) -> bool {
    let hole = terrain.tiles[tile * 2u + 1u].x;
    return hole > 0.0 && max(abs(world.x), abs(world.z)) < hole - 0.01;
}

@fragment
fn fs_terrain_depth(in: TerrainVaryings) {
    if terrain_cut(in.world, in.tile) {
        discard;
    }
}

@fragment
fn fs_terrain(in: TerrainVaryings) -> @location(0) vec4<f32> {
    let t = terrain.tiles[in.tile * 2u];
    let layer = i32(t.w);
    let n = normalize(in.normal);
    let dwx = dpdx(in.world);
    let dwy = dpdy(in.world);

    // The ground at the four samples around this point, weighted by nearness.
    let base = clamp(vec2<i32>(floor(in.local)), vec2<i32>(0), vec2<i32>(63));
    let f = in.local - vec2<f32>(base);
    var weights = array<f32, 9>(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
    for (var c = 0; c < 4; c++) {
        let o = vec2<i32>(c & 1, c >> 1);
        let g = min(textureLoad(terrain_ground, base + o + 1, layer, 0).r, 7u);
        let w = select(1.0 - f.x, f.x, o.x == 1) * select(1.0 - f.y, f.y, o.y == 1);
        weights[g] += w;
    }
    // Ground too steep to hold soil is rock, and so is the highest ground. Water's bed,
    // the Erasure, and roads are what they are however they lie.
    let steep = 1.0 - smoothstep(0.62, 0.84, n.y);
    let high = smoothstep(140.0, 300.0, in.world.y) * 0.8;
    let rock = clamp(max(steep, high) * (1.0 - weights[7] - weights[2] - weights[5]), 0.0, 1.0);
    for (var g = 0; g < 8; g++) {
        weights[g] *= 1.0 - rock;
    }
    weights[8] = rock;
    // The two strongest.
    var first = 0;
    var second = 1;
    if weights[second] > weights[first] {
        first = 1;
        second = 0;
    }
    for (var g = 2; g < 9; g++) {
        if weights[g] > weights[first] {
            second = first;
            first = g;
        } else if weights[g] > weights[second] {
            second = g;
        }
    }
    let m1 = terrain_material(u32(first));
    let m2 = terrain_material(u32(second));

    // The page of each cell of the grid lies along the axis the cell most faces, chosen
    // hard and never blended, so that no letter is ever written twice.
    let h00 = terrain_sample(base.x, base.y, layer);
    let h10 = terrain_sample(base.x + 1, base.y, layer);
    let h01 = terrain_sample(base.x, base.y + 1, layer);
    let h11 = terrain_sample(base.x + 1, base.y + 1, layer);
    let cell_normal = normalize(vec3<f32>(-(h10 - h00 + h11 - h01) * 0.5 / t.z, 1.0, -(h01 - h00 + h11 - h10) * 0.5 / t.z));
    var page_x = vec3<f32>(1.0, 0.0, 0.0);
    var page_y = vec3<f32>(0.0, 0.0, -1.0);
    let a = abs(cell_normal);
    if a.x > a.y && a.x >= a.z {
        page_x = vec3<f32>(0.0, 0.0, -sign(cell_normal.x));
        page_y = vec3<f32>(0.0, 1.0, 0.0);
    } else if a.z > a.y && a.z > a.x {
        page_x = vec3<f32>(sign(cell_normal.z), 0.0, 0.0);
        page_y = vec3<f32>(0.0, 1.0, 0.0);
    }
    let anchor = vec2<f32>(dot(in.world, page_x), dot(in.world, page_y));
    let footprint = AnchorFootprint(vec2<f32>(dot(dwx, page_x), dot(dwx, page_y)), vec2<f32>(dot(dwy, page_x), dot(dwy, page_y)));

    // The two materials, blended by their own heights so that the edge between them
    // is the edge between stones and grass, not a smear.
    let tile1 = materials.entries[m1].surface.x;
    let tile2 = materials.entries[m2].surface.x;
    let uv = vec2<f32>(anchor.x, -anchor.y);
    let ddx = vec2<f32>(footprint.dx.x, -footprint.dx.y);
    let ddy = vec2<f32>(footprint.dy.x, -footprint.dy.y);
    let c1 = textureSampleGrad(material_colour, material_sampler, uv / tile1, m1, ddx / tile1, ddy / tile1);
    let c2 = textureSampleGrad(material_colour, material_sampler, uv / tile2, m2, ddx / tile2, ddy / tile2);
    let n1 = textureSampleGrad(material_normal, material_sampler, uv / tile1, m1, ddx / tile1, ddy / tile1);
    let n2 = textureSampleGrad(material_normal, material_sampler, uv / tile2, m2, ddx / tile2, ddy / tile2);
    let share = weights[second] / max(weights[first] + weights[second], 1.0e-4);
    let blend = clamp((share - 0.5) * 3.0 + (n2.w - n1.w) * 1.5 + 0.5, 0.0, 1.0);
    let colour_texel = mix(c1, c2, blend);
    let normal_texel = mix(n1, n2, blend);
    // The letters are those of whichever material shows here.
    let material = select(m1, m2, blend > 0.5);

    // The page's axes laid onto the surface. Where the cell's page and the smooth
    // normal disagree sharply, one axis can vanish; the other is then made from it.
    var tangent = page_x - n * dot(n, page_x);
    if dot(tangent, tangent) < 1.0e-4 {
        tangent = cross(page_y, n);
    }
    tangent = normalize(tangent);
    var bitangent = cross(n, tangent);
    bitangent = normalize(bitangent * sign(dot(bitangent, page_y) + 1.0e-6));
    let map_normal = vec2<f32>(normal_texel.x, normal_texel.y) * 2.0 - 1.0;

    var glyphs = GlyphResult(1.0, vec2<f32>(0.0), 0.0, 0.0);
    if view.switches.x > 0.5 {
        glyphs = glyph_surface(anchor, footprint, material, 0u, colour_texel.rgb, 0.0, 0.0);
    }
    if terrain_cut(in.world, in.tile) {
        discard;
    }
    if view.viewport.w > 0.5 {
        return vec4<f32>(vec3<f32>(glyphs.revealed * 100.0), 1.0);
    }
    let tangent_normal = map_normal + glyphs.relief;
    let z = sqrt(max(1.0 - dot(tangent_normal, tangent_normal), 0.05));
    let normal = normalize(tangent * tangent_normal.x + bitangent * tangent_normal.y + n * z);
    let surface = Surface(colour_texel.rgb * glyphs.tone, normal, normal_texel.z, colour_texel.a);
    let as_surface = Varyings(in.position, in.world, n, vec4<f32>(tangent, 1.0), anchor, material, 0u, 1.2);
    var colour = light_surface(surface, as_surface, n, 0.0);
    let look = u32(view.effects.z + 0.5);
    if look > 0u && glyphs.revealed > 0.0 {
        let to_eye = normalize(view.eye_time.xyz - in.world);
        let in_frame = vec3<f32>(dot(to_eye, tangent), dot(to_eye, bitangent), dot(to_eye, n));
        let world = WorldFootprint(in.world, dwx, dwy);
        let albedo = max(colour_texel.rgb * glyphs.tone, vec3<f32>(0.03));
        let win = Window(anchor, tangent, bitangent, colour / albedo, 0.0);
        colour = glyph_depth(colour, anchor, footprint, in_frame, world, -to_eye, length(view.eye_time.xyz - in.world), material, 0u, 1.2, glyphs.ink, look, win);
    }
    colour = through_air(colour, in.world, in.position.xy - view.viewport_origin.xy);
    if view.switches.y > 0.0 {
        let text = mix(paper_of(colour_texel.rgb), ink_of(colour_texel.rgb), glyphs.ink) / view.viewport.z;
        colour = mix(colour, text, view.switches.y);
    }
    // Never hand the bloom chain a NaN: one would spread through it into a block.
    colour = select(colour, vec3<f32>(0.0), colour != colour);
    return vec4<f32>(colour, glyphs.revealed);
}
