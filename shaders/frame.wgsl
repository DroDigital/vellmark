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
    // xyz: towards the moon. w: how much of its face is lit, 0 to 1.
    moon_dir: vec4<f32>,
    // x: how much of the sky is cloud, 0 to 1. y: the mist's density per metre at its
    // floor. z: the height of its floor. w: the metres in which it thins to a third.
    sky: vec4<f32>,
    // x: rain, 0 to 1. y: how wet the world is. z: the wind's strength.
    weather: vec4<f32>,
    // xy: the wind's direction over the ground.
    wind: vec4<f32>,
    // The wind's sentence on the ground: xy where its line starts, zw the way it runs.
    sentence: vec4<f32>,
    // x: how far the gust has come, 0 to 1, or less than 0 for none. y: which sentence.
    // z: the width of a letter, in metres. w: its depth, away from the eye.
    gust: vec4<f32>,
    // The finest terrain tiles around the eye: corner x and z, layer, and 1 if present.
    ground_tiles: array<vec4<f32>, 4>,
    // What the governor has left: x the share of grass beyond 30 m, y contact
    // occlusion's strength, z the fog grid's share of its full size, w the share of
    // particles drawn.
    quality: vec4<f32>,
}

// The fog grid at its full size, and the part of it in use.
const FOG_GRID_MAX: vec3<u32> = vec3<u32>(160u, 90u, 64u);

fn fog_grid(view_quality: f32) -> vec3<u32> {
    return max(vec3<u32>(1u), vec3<u32>(round(vec3<f32>(FOG_GRID_MAX) * view_quality)));
}

// ---- the atmosphere: the same model as `crates/render/src/sky.rs` ----

const EARTH: f32 = 6360000.0;
const SKY_TOP: f32 = 6460000.0;
const RAYLEIGH: vec3<f32> = vec3<f32>(5.802e-6, 13.558e-6, 33.1e-6);
const MIE_SCATTER: f32 = 3.996e-6;
const MIE_EXTINCTION: f32 = 4.40e-6;
const OZONE: vec3<f32> = vec3<f32>(0.650e-6, 1.881e-6, 0.085e-6);
const SUN: f32 = 6.4;
const MOON: f32 = 0.05;
const NIGHT_GLOW: vec3<f32> = vec3<f32>(0.0016, 0.0022, 0.0042);
const SKY_EYE: f32 = 120.0;


fn sky_extinction(height: f32) -> vec3<f32> {
    let h = max(height, 0.0);
    let ozone = max(1.0 - abs(h - 25000.0) / 15000.0, 0.0);
    return RAYLEIGH * exp(-h / 8000.0) + vec3<f32>(MIE_EXTINCTION * exp(-h / 1200.0)) + OZONE * ozone;
}

fn to_sphere(radius: f32, cos: f32, to: f32) -> f32 {
    let b = radius * cos;
    let d = b * b - (radius * radius - to * to);
    if d < 0.0 {
        return -1.0;
    }
    let s = sqrt(d);
    return select(-b + s, -b - s, -b - s > 0.0);
}

// How much light gets through the air from above to a point at a height.
fn sky_transmittance(height: f32, cos: f32) -> vec3<f32> {
    let radius = EARTH + max(height, 0.0);
    if cos < 0.0 && to_sphere(radius, cos, EARTH) > 0.0 {
        return vec3<f32>(0.0);
    }
    let span = max(to_sphere(radius, cos, SKY_TOP), 0.0);
    let ds = span / 6.0;
    var depth = vec3<f32>(0.0);
    for (var i = 0; i < 6; i++) {
        let t = (f32(i) + 0.5) * ds;
        let r = sqrt(radius * radius + t * t + 2.0 * radius * t * cos);
        depth += sky_extinction(r - EARTH) * ds;
    }
    return exp(-depth);
}

// The light the air scatters towards the eye along a direction, from one light.
fn sky_scatter(view_dir: vec3<f32>, light: vec3<f32>) -> vec3<f32> {
    let radius = EARTH + SKY_EYE;
    let cos = view_dir.y;
    let ground = to_sphere(radius, cos, EARTH);
    var span = max(to_sphere(radius, cos, SKY_TOP), 0.0);
    if ground > 0.0 && cos < 0.0 {
        span = ground;
    }
    let mu = dot(view_dir, light);
    let pr = 3.0 / (16.0 * PI) * (1.0 + mu * mu);
    let g = 0.8;
    let pm = 3.0 / (8.0 * PI) * (1.0 - g * g) / (2.0 + g * g) * (1.0 + mu * mu) / pow(1.0 + g * g - 2.0 * g * mu, 1.5);
    let ds = span / 12.0;
    var through = vec3<f32>(1.0);
    var gathered = vec3<f32>(0.0);
    for (var i = 0; i < 12; i++) {
        let t = (f32(i) + 0.5) * ds;
        let point = vec3<f32>(0.0, radius, 0.0) + view_dir * t;
        let r = length(point);
        let height = r - EARTH;
        let up = point / r;
        let lit = sky_transmittance(height, dot(up, light));
        let scatter = (RAYLEIGH * exp(-height / 8000.0) * pr + vec3<f32>(MIE_SCATTER * exp(-height / 1200.0) * pm)) * lit;
        gathered += through * scatter * ds;
        through *= exp(-sky_extinction(height) * ds);
    }
    return gathered;
}

fn sky_hash(p: vec2<f32>) -> f32 {
    let q = fract(p * vec2<f32>(0.1031, 0.1030));
    let r = q + dot(q, q.yx + 33.33);
    return fract((r.x + r.y) * r.x);
}

fn sky_noise(p: vec2<f32>) -> f32 {
    let i = floor(p);
    let f = fract(p);
    let u = f * f * (3.0 - 2.0 * f);
    let a = sky_hash(i);
    let b = sky_hash(i + vec2<f32>(1.0, 0.0));
    let c = sky_hash(i + vec2<f32>(0.0, 1.0));
    let d = sky_hash(i + vec2<f32>(1.0, 1.0));
    return mix(mix(a, b, u.x), mix(c, d, u.x), u.y);
}

fn sky_fbm(p: vec2<f32>) -> f32 {
    var sum = 0.0;
    var amplitude = 0.5;
    var q = p;
    for (var o = 0; o < 5; o++) {
        sum += sky_noise(q) * amplitude;
        q = q * 2.03 + vec2<f32>(17.1, 9.7);
        amplitude *= 0.5;
    }
    return sum;
}

// The stars, turning about the pole as the night goes. `pixel` is how far apart two
// pixels are, in radians.
fn stars(dir: vec3<f32>, view: ViewUniform, pixel: f32) -> vec3<f32> {
    // The pole stands 52 degrees above the northern horizon.
    let pole = vec3<f32>(0.0, 0.788, -0.616);
    let angle = view.light_dir_hour.w / 24.0 * 2.0 * PI;
    let c = cos(angle);
    let s = sin(angle);
    let d = dir * c + cross(pole, dir) * s + pole * dot(pole, dir) * (1.0 - c);
    let rows = 900.0;
    let lat = asin(clamp(d.y, -1.0, 1.0));
    let lon = atan2(d.z, d.x);
    let row = floor((lat / PI + 0.5) * rows);
    let row_lat = ((row + 0.5) / rows - 0.5) * PI;
    let columns = max(1.0, floor(2.0 * rows * cos(row_lat)));
    let column = floor((lon / (2.0 * PI) + 0.5) * columns);
    let seed = vec2<f32>(row, column);
    let chance = sky_hash(seed);
    if chance > 0.08 {
        return vec3<f32>(0.0);
    }
    // Where in its cell the star is, kept clear of the cell's edges.
    let jitter = vec2<f32>(sky_hash(seed + 7.0), sky_hash(seed + 13.0)) * 0.6 + 0.2;
    let star_lat = ((row + jitter.x) / rows - 0.5) * PI;
    let star_lon = ((column + jitter.y) / columns - 0.5) * 2.0 * PI;
    let star = vec3<f32>(cos(star_lat) * cos(star_lon), sin(star_lat), cos(star_lat) * sin(star_lon));
    let apart = length(cross(d, star));
    // Most stars are faint; a few are bright.
    let magnitude = pow(sky_hash(seed + 29.0), 7.0);
    let twinkle = 0.8 + 0.2 * sin(view.eye_time.w * (5.0 + chance * 30.0) + chance * 100.0);
    let warmth = sky_hash(seed + 41.0);
    let tint = mix(vec3<f32>(0.75, 0.85, 1.0), vec3<f32>(1.0, 0.85, 0.65), warmth);
    let point = exp(-pow(apart / max(pixel, 1.0e-5), 2.0) * 1.6);
    return tint * point * (0.03 + magnitude * 2.4) * twinkle;
}

// One layer of cloud at a height: its light, and in w how much it hides what is behind.
fn cloud_layer(dir: vec3<f32>, view: ViewUniform, height: f32, scale: f32, cover: f32, drift: vec2<f32>, thick: f32) -> vec4<f32> {
    if dir.y <= 0.01 {
        return vec4<f32>(0.0);
    }
    let t = min(height / dir.y, 80000.0);
    let p = (view.eye_time.xz + dir.xz * t) / scale + drift * view.eye_time.w;
    let n = sky_fbm(p);
    let density = smoothstep(1.0 - cover, 1.0 - cover + 0.28, n) * thick;
    if density <= 0.001 {
        return vec4<f32>(0.0);
    }
    // Self-shadow: how much cloud lies between this point and the light.
    let light = view.light_dir_hour.xyz;
    let towards = sky_fbm(p + light.xz * 0.35);
    let shade = exp(-smoothstep(1.0 - cover, 1.0 - cover + 0.28, towards) * 2.2);
    let mu = dot(dir, light);
    let forward = 1.0 + 1.6 * pow(max(mu, 0.0), 8.0);
    let lit = view.light_colour.rgb * (0.12 + 0.30 * shade) * forward + view.sky_zenith.rgb * 0.9 + view.sky_horizon.rgb * 0.35;
    // Far cloud fades into the haze of the horizon.
    let far = 1.0 - exp(-t / 45000.0);
    let colour = mix(lit, view.sky_horizon.rgb, far * 0.8);
    let edge = smoothstep(0.01, 0.12, dir.y);
    return vec4<f32>(colour, clamp(density, 0.0, 1.0) * edge);
}

// The colour of the sky in a direction: the air, lit by the sun by day and the moon by
// night; the stars; the sun's disc; and two layers of cloud. `pixel` is the angle
// between two pixels, for drawing the stars and the sun's edge sharp.
fn sky_colour(dir: vec3<f32>, view: ViewUniform) -> vec3<f32> {
    return sky_colour_sized(dir, view, 0.0006);
}

fn sky_colour_sized(dir: vec3<f32>, view: ViewUniform, pixel: f32) -> vec3<f32> {
    let sun = view.sun_dir.xyz;
    let moon = view.moon_dir.xyz;
    let d = normalize(vec3<f32>(dir.x, max(dir.y, 0.0), dir.z));
    var colour = NIGHT_GLOW;
    if sun.y > -0.2 {
        colour += sky_scatter(d, sun) * SUN;
    }
    if sun.y < 0.05 && moon.y > -0.05 {
        colour += sky_scatter(d, moon) * SUN * MOON * view.moon_dir.w;
    }
    // The stars show as the sky darkens, dimmed near the horizon by the air.
    let sky_light = dot(colour, vec3<f32>(0.2126, 0.7152, 0.0722));
    let night = clamp(1.0 - sky_light * 90.0, 0.0, 1.0);
    if night > 0.0 && dir.y > 0.0 {
        colour += stars(dir, view, pixel) * night * sky_transmittance(SKY_EYE, dir.y);
    }
    // The sun's disc, darker towards its rim, behind the cloud.
    let toward = dot(dir, sun);
    let radius = 0.0047;
    let off = sqrt(max(1.0 - toward * toward, 0.0));
    if toward > 0.0 && off < radius + pixel * 2.0 {
        let rim = clamp(off / radius, 0.0, 1.0);
        let limb = 1.0 - 0.6 * (1.0 - sqrt(max(1.0 - rim * rim, 0.0)));
        let disc = 1.0 - smoothstep(radius - pixel, radius + pixel, off);
        colour += sky_transmittance(SKY_EYE, max(sun.y, 0.0)) * vec3<f32>(9000.0) * disc * limb * step(0.0, sun.y + 0.01);
    }
    // Cloud: a broken layer of cumulus, and a thin high veil above it.
    let cover = view.sky.x;
    let high = cloud_layer(dir, view, 8500.0, 9000.0, cover * 0.7, vec2<f32>(0.0018, 0.0006), 0.55);
    colour = mix(colour, high.rgb, high.w);
    let low = cloud_layer(dir, view, 1800.0, 2600.0, cover, vec2<f32>(0.004, 0.0015), 1.0);
    colour = mix(colour, low.rgb, low.w);
    return colour;
}
