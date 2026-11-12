// Every opaque surface: the depth pre-pass, the lit pass, and the sky behind them.

#import frame
#import shading
#import glyph

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
    // x: where this batch starts in the list being drawn.
    base: vec4<u32>,
}


@group(0) @binding(0) var<uniform> view: ViewUniform;
@group(0) @binding(1) var shadow_map: texture_depth_2d_array;
@group(0) @binding(2) var shadow_sampler: sampler_comparison;
@group(0) @binding(3) var<uniform> lights: Lights;
@group(0) @binding(4) var light_clusters: texture_2d_array<u32>;
@group(0) @binding(5) var light_shadow: texture_depth_cube;
@group(0) @binding(6) var fog_volume: texture_3d<f32>;
@group(0) @binding(7) var fog_sampler: sampler;
@group(0) @binding(8) var contact_occlusion: texture_2d<f32>;

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

// Every placed thing; the lists of which to draw; and where this batch starts in its list.
@group(2) @binding(0) var<storage, read> instances: array<Instance>;
@group(2) @binding(1) var<storage, read> visible: array<u32>;
@group(2) @binding(2) var<uniform> batch: Batch;

// Which instance this draw is, in the list being drawn.
fn instance_of(index: u32) -> u32 {
    return visible[batch.base.x + index];
}

// A position that every clip test throws away: the triangle is not drawn.
const NOWHERE: vec4<f32> = vec4<f32>(2.0, 2.0, 2.0, 1.0);

struct VertexInput {
    @location(0) position: vec3<f32>,
    @location(1) normal: vec3<f32>,
    @location(2) tangent: vec4<f32>,
    @location(3) anchor: vec2<f32>,
    @location(4) material: u32,
    @location(5) page: u32,
    @location(6) thickness: f32,
    @location(7) sky: f32,
    @builtin(instance_index) instance: u32,
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
    @location(7) sky: f32,
    @location(8) @interpolate(flat) instance: u32,
}

@vertex
fn vs_depth(in: VertexInput) -> @builtin(position) @invariant vec4<f32> {
    let object = instances[instance_of(in.instance)];
    if object.state.y > 0.5 {
        return NOWHERE;
    }
    let world = object.model * vec4<f32>(in.position, 1.0);
    return view.view_proj * world;
}

@vertex
fn vs(in: VertexInput) -> Varyings {
    let id = instance_of(in.instance);
    let object = instances[id];
    let world = object.model * vec4<f32>(in.position, 1.0);
    var out: Varyings;
    out.position = view.view_proj * world;
    if object.state.y > 0.5 {
        out.position = NOWHERE;
    }
    out.instance = id;
    out.world = world.xyz;
    out.normal = (object.model * vec4<f32>(in.normal, 0.0)).xyz;
    out.tangent = vec4<f32>((object.model * vec4<f32>(in.tangent.xyz, 0.0)).xyz, in.tangent.w);
    out.anchor = in.anchor;
    out.material = in.material;
    out.page = in.page;
    out.thickness = in.thickness;
    out.sky = in.sky;
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
// `seen` is the share of the sky the place can see, baked: it scales the sky's light.
fn light_surface(surface: Surface, in: Varyings, flat_normal: vec3<f32>, glow: f32, seen: f32) -> vec3<f32> {
    let to_eye = normalize(view.eye_time.xyz - in.world);
    let depth_in_view = -(view.view * vec4<f32>(in.world, 1.0)).z;
    let shadow = sun_shadow(in.world, flat_normal, depth_in_view);
    var colour = shade_light(surface, to_eye, view.light_dir_hour.xyz, view.light_colour.rgb) * shadow;
    // Contact occlusion: the fine darkening where things meet, at half resolution.
    let contact = textureSampleLevel(contact_occlusion, fog_sampler, in.position.xy * 0.5 / vec2<f32>(textureDimensions(contact_occlusion)), 0.0).r;
    // The sky's light, as much of it as this place can see.
    colour += shade_sky(surface, to_eye, view.sky_zenith.rgb, view.sky_horizon.rgb, view.sky_horizon.rgb * 0.35) * seen * contact;
    // And what the sunlit ground sends back, from below and the sides. A place that sees
    // less sky sees more ground, so this is not scaled by `seen`.
    let bounce = view.light_colour.rgb * max(view.light_dir_hour.y, 0.0) * 0.1;
    colour += surface.albedo * (1.0 - surface.metal) * bounce * (0.5 - surface.normal.y * 0.5) * contact;
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
        plain_lit = light_surface(plain, in, n, m.surface.w, in.sky);
        let seen = plain_lit * view.viewport.z;
        light = clamp(dot(seen / (seen + vec3<f32>(0.6)), vec3<f32>(0.2126, 0.7152, 0.0722)) * 1.6, 0.0, 1.0);
    }

    // The fine print. Beyond the reveal distance this returns at once, having sampled
    // nothing, and the surface is exactly what it would be with the layer off.
    var glyphs = GlyphResult(1.0, vec2<f32>(0.0), 0.0, 0.0);
    if view.switches.x > 0.5 {
        glyphs = glyph_surface(in.anchor, footprint, in.material, in.page, colour_texel.rgb, light, instances[in.instance].state.x);
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
        colour = light_surface(surface, in, n, m.surface.w, in.sky);
        // Heat. A thing hot enough glows, as hot iron does, at any distance: that is the
        // world, not its letters. Up close, the strokes of the letters saying so glow
        // first, as embers in the grain.
        let hot = max(instances[in.instance].state.x, 0.0);
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
            let win = Window(in.anchor, t, b, colour / albedo, instances[in.instance].state.x);
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
    let as_surface = Varyings(in.position, in.world, n, vec4<f32>(1.0, 0.0, 0.0, 1.0), vec2<f32>(0.0), in.material, 0u, 0.0, 1.0, 0u);
    let colour = through_air(light_surface(surface, as_surface, n, 0.0, 1.0), in.world, in.position.xy - view.viewport_origin.xy);
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
@group(3) @binding(3) var terrain_sky: texture_2d_array<f32>;

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
    let as_surface = Varyings(in.position, in.world, n, vec4<f32>(tangent, 1.0), anchor, material, 0u, 1.2, 1.0, 0u);
    // The share of the sky this place sees, baked, between its four samples.
    let s00 = textureLoad(terrain_sky, base + 1, layer, 0).r;
    let s10 = textureLoad(terrain_sky, base + vec2<i32>(2, 1), layer, 0).r;
    let s01 = textureLoad(terrain_sky, base + vec2<i32>(1, 2), layer, 0).r;
    let s11 = textureLoad(terrain_sky, base + 2, layer, 0).r;
    let seen = mix(mix(s00, s10, f.x), mix(s01, s11, f.x), f.y);
    var colour = light_surface(surface, as_surface, n, 0.0, seen);
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

// ---- grass: blades made from their number, on the terrain they stand on ----

struct GrassUniform {
    // xy: the wind's direction over the ground. z: its strength, 0 to 1. w: how fast its
    // gusts travel, in metres a second.
    wind: vec4<f32>,
    // x: within this distance the grass is at full density. y: blades per square metre
    // there. z: the side of a patch. w: the farthest any blade is drawn.
    shape: vec4<f32>,
    // Each thing that pushes the grass aside: where it is, and how far it reaches.
    pushers: array<vec4<f32>, 8>,
    // x: how many pushers. y: the blades a patch holds at full density.
    counts: vec4<u32>,
}

@group(3) @binding(4) var<uniform> grass: GrassUniform;

struct GrassVaryings {
    @builtin(position) @invariant position: vec4<f32>,
    @location(0) world: vec3<f32>,
    @location(1) normal: vec3<f32>,
    // x: along the blade, in metres from the root. y: across it, in metres from its middle.
    @location(2) page: vec2<f32>,
    // How far up the blade: 0 at the root, 1 at the tip.
    @location(3) up: f32,
    @location(4) @interpolate(flat) seed: u32,
    @location(5) @interpolate(flat) material: u32,
    // x: the share of the sky the root sees. y: the side of the blade's character cells.
    @location(6) @interpolate(flat) blade: vec2<f32>,
}

@vertex
fn vs_grass(@builtin(vertex_index) vertex: u32, @builtin(instance_index) index: u32) -> GrassVaryings {
    var out: GrassVaryings;
    out.position = NOWHERE;
    out.world = vec3<f32>(0.0);
    out.normal = vec3<f32>(0.0, 1.0, 0.0);
    out.page = vec2<f32>(0.0);
    out.up = 0.0;
    out.seed = 0u;
    out.material = 0u;
    out.blade = vec2<f32>(0.0);

    let object = instances[instance_of(index)];
    let origin = object.model[3].xyz;
    let tile = u32(object.state.z);
    let blade = vertex / 7u;
    let k = vertex % 7u;
    let side_of_patch = grass.shape.z;
    let cell = vec2<i32>(floor(origin.xz / side_of_patch + 0.5));
    let seed = hash_cell(cell, i32(blade), 0x9e37u);
    let shape_hash = hash_cell(cell, i32(blade), 0x61u);
    let r1 = unit(shape_hash);
    let r2 = unit(shape_hash >> 16u);
    let r3 = unit(seed >> 16u);

    // Where the blade stands. Any first run of this sequence is spread evenly over the
    // patch, so each level of detail, which draws only the first blades, is even too.
    let spread = fract(vec2<f32>(0.5) + f32(blade) * vec2<f32>(0.7548776662, 0.5698402910));
    let jitter = vec2<f32>(unit(seed), unit(shape_hash >> 8u)) - 0.5;
    let at = origin.xz + fract(spread + jitter * 0.6 / sqrt(f32(grass.counts.y))) * side_of_patch;

    // The ground under it, exactly as the terrain is drawn there.
    let t = terrain.tiles[tile * 2u];
    let layer = i32(t.w);
    let local = (at - t.xy) / t.z;
    let base = clamp(vec2<i32>(floor(local)), vec2<i32>(0), vec2<i32>(63));
    let f = clamp(local - vec2<f32>(base), vec2<f32>(0.0), vec2<f32>(1.0));
    let h00 = terrain_sample(base.x, base.y, layer);
    let h10 = terrain_sample(base.x + 1, base.y, layer);
    let h01 = terrain_sample(base.x, base.y + 1, layer);
    let h11 = terrain_sample(base.x + 1, base.y + 1, layer);
    var height: f32;
    if f.x >= f.y {
        height = h00 + f.x * (h10 - h00) + f.y * (h11 - h10);
    } else {
        height = h00 + f.y * (h01 - h00) + f.x * (h11 - h01);
    }
    let slope = normalize(vec3<f32>(-(h10 - h00 + h11 - h01) * 0.5 / t.z, 1.0, -(h01 - h00 + h11 - h10) * 0.5 / t.z));
    let nearest = clamp(vec2<i32>(round(local)), vec2<i32>(0), vec2<i32>(64));
    let ground = textureLoad(terrain_ground, nearest + 1, layer, 0).r;
    let barley = ground == 1u;
    // Pasture and barley are full of it; a wood's floor has a little; nothing grows on
    // roads, earth, banks, the bed of the water, or rock.
    var grows = select(0.0, 1.0, ground == 0u || barley);
    if ground == 6u {
        grows = 0.2;
    }
    grows = select(0.0, 1.0, unit(seed >> 8u) < grows) * smoothstep(0.62, 0.8, slope.y);
    let root = vec3<f32>(at.x, height, at.y);

    // Fewer blades with distance, each fading out by its own number, so that none pops.
    let eye = view.eye_time.xyz;
    let dist = distance(eye, root);
    let near = grass.shape.x;
    let keep = min(1.0, (near * near) / max(dist * dist, 1.0e-3));
    let wanted = f32(grass.counts.y) * keep;
    var fade = clamp((wanted - f32(blade)) / max(wanted * 0.3, 1.0), 0.0, 1.0);
    fade *= 1.0 - smoothstep(grass.shape.w * 0.7, grass.shape.w, dist);
    fade *= grows;
    if fade <= 0.0 {
        return out;
    }

    // Its shape: meadows run tall and short in broad swathes, and in clumps within them.
    let clump = sky_noise(at * 0.35);
    let meadow = sky_noise(at * 0.06 + vec2<f32>(17.0, 3.0));
    var tall: f32;
    var wide: f32;
    if barley {
        tall = (0.85 + 0.25 * r1) * (0.9 + 0.2 * clump);
        wide = 0.0065 + 0.002 * r2;
    } else {
        tall = (0.16 + 0.3 * r1) * (0.55 + 0.9 * meadow) * (0.8 + 0.4 * clump);
        wide = 0.009 + 0.007 * r2;
    }
    let cell_side = wide;
    tall *= mix(0.25, 1.0, fade);
    // Beyond the near distance each blade stands for several, so it is wider: about the
    // same width on screen at any distance, and the field keeps its cover.
    wide *= max(1.0, dist / near) * fade;

    // Its lean: its own, the wind's, and anything walking through it.
    let yaw = unit(shape_hash >> 4u) * 6.2831853;
    let across = vec3<f32>(cos(yaw), 0.0, sin(yaw));
    let facing = vec3<f32>(-sin(yaw), 0.0, cos(yaw));
    var lean = facing.xz * select(0.1 + 0.45 * r3, 0.04 + 0.08 * r3, barley);
    let time = view.eye_time.w;
    let wind = grass.wind.xy;
    let gust_at = at * 0.045 - wind * time * grass.wind.w * 0.045;
    let gust = sky_noise(gust_at) * 0.7 + sky_noise(gust_at * 2.3 + vec2<f32>(5.0, 9.0)) * 0.3;
    let flutter = sin(time * (1.7 + 1.3 * r2) + r1 * 6.2831853);
    lean += wind * grass.wind.z * ((0.1 + 1.1 * gust * gust) * select(1.0, 0.7, barley) + 0.07 * flutter);
    for (var p = 0u; p < min(grass.counts.x, 8u); p++) {
        let pusher = grass.pushers[p];
        let away = at - pusher.xz;
        let d = length(away);
        if d < pusher.w && root.y > pusher.y - 2.4 && root.y < pusher.y + 0.3 {
            let push = 1.0 - d / pusher.w;
            lean += away / max(d, 1.0e-3) * push * (2.0 - push);
        }
    }
    let amount = min(length(lean), 1.45);
    let towards = vec3<f32>(lean.x, 0.0, lean.y) / max(length(lean), 1.0e-5);
    let up = vec3<f32>(0.0, 1.0, 0.0);
    let tip = root + (towards * sin(amount) + up * cos(amount)) * tall;
    let knee = root + (towards * sin(amount * 0.5) + up * cos(amount * 0.5)) * tall * 0.5;

    // Three pairs of vertices up the blade and one at the tip. Barley's head is the
    // swelling near the top.
    let level = min(k / 2u, 3u);
    var ups = array<f32, 4>(0.0, 0.4, 0.75, 1.0);
    var widths = array<f32, 4>(1.0, 0.72, 0.4, 0.0);
    if barley {
        ups = array<f32, 4>(0.0, 0.58, 0.8, 1.0);
        widths = array<f32, 4>(0.6, 0.5, 1.7, 0.0);
    }
    let s = ups[level];
    let along = mix(mix(root, knee, s), mix(knee, tip, s), s);
    let tangent = normalize(mix(knee - root, tip - knee, s) + up * 1.0e-4);
    let sign = select(select(-1.0, 1.0, (k & 1u) == 1u), 0.0, k == 6u);
    let half = wide * widths[level] * 0.5;
    out.world = along + across * sign * half;
    // Rounded across its width, so it catches the light as a real blade does, and bent
    // towards the sky, as the sward together faces it: a field is lit as the ground it
    // covers, and each blade still has its own light.
    let own = normalize(cross(across, tangent) + across * sign * 0.5);
    out.normal = normalize(mix(own, slope, 0.55));
    out.page = vec2<f32>(s * tall, sign * cell_side * widths[level] * 0.5);
    out.up = s;
    out.seed = seed;
    out.material = terrain_material(select(0u, 1u, barley));
    out.blade = vec2<f32>(textureLoad(terrain_sky, nearest + 1, layer, 0).r, cell_side);
    out.position = view.view_proj * vec4<f32>(out.world, 1.0);
    return out;
}

struct BladeLetters {
    // Multiplies the blade's colour. Averages to 1 over a character.
    tone: f32,
    // How much of the pixel is ink.
    ink: f32,
    // 1 where the letters are drawn at all.
    revealed: f32,
    // Below one half, the pixel is not part of the blade: at arm's length the blade is
    // nothing but its characters.
    keep: f32,
}

// Each blade is a line of characters, running from its root to its tip: the words of its
// lexicon, `grass` and what grass is made of. Beyond the reveal distance this returns at
// once, having sampled nothing.
fn blade_letters(in: GrassVaryings, front: bool) -> BladeLetters {
    var out = BladeLetters(1.0, 0.0, 0.0, 1.0);
    // Seen from the side the normal faces, the line would read mirrored; turned over.
    let page = vec2<f32>(in.page.x, select(in.page.y, -in.page.y, front));
    let dx = dpdx(page);
    let dy = dpdy(page);
    if view.switches.x < 0.5 {
        return out;
    }
    let c = in.blade.y;
    let squeeze = max(length(dx), length(dy));
    if squeeze <= 0.0 || c <= 0.0 {
        return out;
    }
    let h = c / squeeze * view.switches.w;
    if h < 3.0 {
        return out;
    }
    let i = i32(floor(page.x / c));
    let uv = vec2<f32>(fract(page.x / c), 0.5 - page.y / c);
    let du = dx * vec2<f32>(1.0 / c, -1.0 / c);
    let dv = dy * vec2<f32>(1.0 / c, -1.0 / c);

    // The line: the blade's words one after another, a space between: its name and what
    // it is made of, `grass C6H12O6 grass C6H10O5`. Every few seconds a blade moves on to
    // the next of its words, and now and then a letter flickers to another.
    let m = materials.entries[in.material];
    let words = 1u + m.info.y;
    let epoch = u32(floor(view.eye_time.w / 7.0 + unit(in.seed >> 3u) * 7.0));
    var slot = (in.seed >> 4u) + epoch;
    var pos = i + i32((in.seed >> 12u) % 7u);
    var character = 32u;
    for (var n = 0; n < 16; n++) {
        let s = slot % words;
        let count = i32(word_length(in.material, s));
        if pos <= count {
            if pos < count {
                character = word_char(in.material, s, u32(pos));
            }
            break;
        }
        pos -= count + 1;
        slot += 1u;
    }
    let tick = hash_cell(vec2<i32>(i, i32(floor(view.eye_time.w * 8.0)) & 0xffff), 0, in.seed);
    if character != 32u && unit(tick) < 0.015 {
        character = 33u + (tick >> 16u) % 94u;
    }
    let g = cell_glyph(character - 32u, uv, du, dv, h, 0.42);
    let w = influence_rise(h);
    let contrast = m.surface.y * view.switches.z;
    out.tone = max(1.0 + contrast * w * nearness(h) * (g.mean - g.cover), 0.05);
    out.ink = g.cover * w;
    out.revealed = 1.0;
    let solid = smoothstep(14.0, 30.0, h);
    let inside = smoothstep(g.threshold - 0.14, g.threshold - 0.05, g.distance);
    out.keep = mix(1.0, inside, solid);
    return out;
}

@fragment
fn fs_grass_depth(in: GrassVaryings, @builtin(front_facing) front: bool) {
    // In a tool's mask the whole blade is kept: what shows through its letters is seen
    // only because they are drawn, so it counts as within the reveal distance.
    if blade_letters(in, front).keep < 0.5 && view.viewport.w < 0.5 {
        discard;
    }
}

@fragment
fn fs_grass(in: GrassVaryings, @builtin(front_facing) front: bool) -> @location(0) vec4<f32> {
    let letters = blade_letters(in, front);
    if view.viewport.w > 0.5 {
        return vec4<f32>(vec3<f32>(letters.revealed * 100.0), 1.0);
    }
    if letters.keep < 0.5 {
        discard;
    }
    let to_eye = normalize(view.eye_time.xyz - in.world);
    var n = normalize(in.normal);
    if !front {
        // The far side of the blade: its normal mirrored about the sky-facing part.
        n = normalize(reflect(n, normalize(vec3<f32>(n.x, 0.0, n.z) + vec3<f32>(0.0, 1.0e-4, 0.0))));
    }
    // Low in the sward the blades shade one another.
    let ao = mix(0.5, 1.0, smoothstep(0.0, 0.8, in.up));
    let place = vec2<f32>(unit(in.seed), unit(in.seed >> 16u));
    let texel = textureSampleLevel(material_colour, material_sampler, place, in.material, 5.0);
    var albedo = texel.rgb * (0.8 + 0.4 * unit(in.seed >> 9u));
    // Tips are drier and paler than the stem.
    albedo = mix(albedo, albedo * vec3<f32>(1.25, 1.18, 0.85), in.up * in.up * 0.6) * ao;
    let surface = Surface(albedo * letters.tone, n, 0.6, 0.0);
    let as_surface = Varyings(in.position, in.world, n, vec4<f32>(1.0, 0.0, 0.0, 1.0), vec2<f32>(0.0), in.material, 0u, 0.0, 1.0, 0u);
    var colour = light_surface(surface, as_surface, n, 0.0, in.blade.x);
    // Light through the blade: a blade lit from behind glows.
    let depth_in_view = -(view.view * vec4<f32>(in.world, 1.0)).z;
    let l = view.light_dir_hour.xyz;
    let through = max(dot(-n, l), 0.0) * 0.3 + pow(max(dot(-to_eye, l), 0.0), 6.0) * 0.8;
    colour += albedo * letters.tone * view.light_colour.rgb * sun_shadow(in.world, n, depth_in_view) * through;
    colour = through_air(colour, in.world, in.position.xy - view.viewport_origin.xy);
    if view.switches.y > 0.0 {
        let text = mix(paper_of(texel.rgb), ink_of(texel.rgb), letters.ink) / view.viewport.z;
        colour = mix(colour, text, view.switches.y);
    }
    colour = select(colour, vec3<f32>(0.0), colour != colour);
    return vec4<f32>(colour, letters.revealed);
}
