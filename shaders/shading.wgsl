// Physically based shading: one lobe of diffuse and one of specular.

const PI: f32 = 3.14159265;

fn distribution_ggx(n_dot_h: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let a2 = a * a;
    let d = n_dot_h * n_dot_h * (a2 - 1.0) + 1.0;
    return a2 / (PI * d * d);
}

fn visibility_smith(n_dot_v: f32, n_dot_l: f32, roughness: f32) -> f32 {
    let a = roughness * roughness;
    let v = n_dot_l * (n_dot_v * (1.0 - a) + a);
    let l = n_dot_v * (n_dot_l * (1.0 - a) + a);
    return 0.5 / max(v + l, 1.0e-5);
}

fn fresnel_schlick(cos_theta: f32, f0: vec3<f32>) -> vec3<f32> {
    return f0 + (vec3<f32>(1.0) - f0) * pow(clamp(1.0 - cos_theta, 0.0, 1.0), 5.0);
}

struct Surface {
    albedo: vec3<f32>,
    normal: vec3<f32>,
    roughness: f32,
    metal: f32,
}

// Light leaving the surface towards `v` from a light in direction `l` with radiance
// `radiance`.
fn shade_light(s: Surface, v: vec3<f32>, l: vec3<f32>, radiance: vec3<f32>) -> vec3<f32> {
    let n_dot_l = dot(s.normal, l);
    if n_dot_l <= 0.0 {
        return vec3<f32>(0.0);
    }
    let h = normalize(v + l);
    let n_dot_v = max(dot(s.normal, v), 1.0e-4);
    let n_dot_h = max(dot(s.normal, h), 0.0);
    let f0 = mix(vec3<f32>(0.04), s.albedo, s.metal);
    let f = fresnel_schlick(max(dot(h, v), 0.0), f0);
    let rough = max(s.roughness, 0.06);
    let specular = f * distribution_ggx(n_dot_h, rough) * visibility_smith(n_dot_v, n_dot_l, rough);
    let diffuse = (vec3<f32>(1.0) - f) * (1.0 - s.metal) * s.albedo / PI;
    return (diffuse + specular) * radiance * n_dot_l;
}

// Light from the whole sky: brighter from above, and from below what the ground sends
// back, which `ground` is.
fn shade_sky(s: Surface, v: vec3<f32>, zenith: vec3<f32>, horizon: vec3<f32>, ground: vec3<f32>) -> vec3<f32> {
    let up = s.normal.y * 0.5 + 0.5;
    let irradiance = mix(ground, mix(horizon, zenith, 0.6), up);
    let f0 = mix(vec3<f32>(0.04), s.albedo, s.metal);
    let diffuse = (1.0 - s.metal) * s.albedo * irradiance;
    // A rough reflection of the sky in the mirror direction.
    let r = reflect(-v, s.normal);
    let reflected = mix(ground, mix(horizon, zenith, 0.6), clamp(r.y * 0.5 + 0.5, 0.0, 1.0));
    let f = fresnel_schlick(max(dot(s.normal, v), 0.0), f0);
    let specular = f * reflected * (1.0 - s.roughness * 0.7);
    return diffuse + specular;
}
