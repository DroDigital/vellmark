//! Fires, lanterns, and windows: point lights, sorted into clusters of the view so
//! that each pixel looks only at the lights that can reach it, and the one nearest
//! light that is allowed a shadow.

use crate::camera::{look_at, View};
use bytemuck::{Pod, Zeroable};
use glam::{Mat4, Vec3};
use vellmark_content::Light;
use vellmark_gpu::{wgpu, Gpu};

pub const MAX_LIGHTS: usize = 64;
/// Clusters across, down, and into the view.
pub const CLUSTERS: (u32, u32, u32) = (16, 9, 16);
/// The nearest and farthest depth the clusters cover, in metres.
pub const CLUSTER_DEPTH: (f32, f32) = (0.1, 120.0);
pub const CUBE_SIZE: u32 = 512;
const CUBE_NEAR: f32 = 0.04;

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct PointLight {
    position_radius: [f32; 4],
    /// rgb: colour times strength. w: 1 if this light's shadow is in the cube.
    colour_shadow: [f32; 4],
}

#[repr(C)]
#[derive(Clone, Copy, Pod, Zeroable)]
struct LightsUniform {
    /// x: number of lights. yzw: unused.
    count: [u32; 4],
    lights: [PointLight; MAX_LIGHTS],
}

pub struct Lights {
    pub uniform: wgpu::Buffer,
    cluster_texture: wgpu::Texture,
    pub cluster_view: wgpu::TextureView,
    pub cube_view: wgpu::TextureView,
    pub cube_faces: Vec<wgpu::TextureView>,
    /// The light whose shadow the cube holds this frame, if any.
    pub shadowed: Option<(Vec3, f32)>,
}

/// The view-projection of one face of a light's shadow cube. Faces are in the order
/// +X, -X, +Y, -Y, +Z, -Z, each drawn so that the cube can be sampled by direction.
pub fn cube_face_matrix(position: Vec3, radius: f32, face: usize) -> Mat4 {
    let (forward, up) = [
        (Vec3::X, Vec3::NEG_Y),
        (Vec3::NEG_X, Vec3::NEG_Y),
        (Vec3::Y, Vec3::Z),
        (Vec3::NEG_Y, Vec3::NEG_Z),
        (Vec3::Z, Vec3::NEG_Y),
        (Vec3::NEG_Z, Vec3::NEG_Y),
    ][face];
    let (n, f) = (CUBE_NEAR, radius);
    // A quarter-turn field of view, depth 0 at near to 1 at far, and the picture
    // turned over so that its rows match a cube map's.
    let projection = Mat4::from_cols_array(&[
        1.0, 0.0, 0.0, 0.0,
        0.0, -1.0, 0.0, 0.0,
        0.0, 0.0, f / (n - f), -1.0,
        0.0, 0.0, n * f / (n - f), 0.0,
    ]);
    projection * look_at(position, position + forward, up)
}

impl Lights {
    pub fn new(gpu: &Gpu) -> Self {
        let uniform = gpu.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("lights"),
            size: std::mem::size_of::<LightsUniform>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        // One row of clusters for each depth slice, one layer for each view. A texel
        // holds 64 bits: which of the lights reach that cluster.
        let cluster_texture = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("light clusters"),
            size: wgpu::Extent3d {
                width: CLUSTERS.0 * CLUSTERS.2,
                height: CLUSTERS.1,
                depth_or_array_layers: crate::frame::MAX_VIEWS as u32,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rg32Uint,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });
        let cluster_view = cluster_texture.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::D2Array),
            ..Default::default()
        });
        let cube = gpu.device.create_texture(&wgpu::TextureDescriptor {
            label: Some("light shadow cube"),
            size: wgpu::Extent3d { width: CUBE_SIZE, height: CUBE_SIZE, depth_or_array_layers: 6 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let cube_view = cube.create_view(&wgpu::TextureViewDescriptor {
            dimension: Some(wgpu::TextureViewDimension::Cube),
            ..Default::default()
        });
        let cube_faces = (0..6)
            .map(|face| {
                cube.create_view(&wgpu::TextureViewDescriptor {
                    dimension: Some(wgpu::TextureViewDimension::D2),
                    base_array_layer: face,
                    array_layer_count: Some(1),
                    ..Default::default()
                })
            })
            .collect();
        Self { uniform, cluster_texture, cluster_view, cube_view, cube_faces, shadowed: None }
    }

    /// Uploads the lights and sorts them into each view's clusters.
    pub fn update(&mut self, gpu: &Gpu, lights: &[Light], views: &[View], aspects: &[f32]) {
        let lights = &lights[..lights.len().min(MAX_LIGHTS)];
        // The shadow goes to the shadow-casting light nearest the first view.
        let eye = views.first().map(|v| v.camera.eye).unwrap_or(Vec3::ZERO);
        let shadowed = lights
            .iter()
            .enumerate()
            .filter(|(_, l)| l.casts_shadow)
            .min_by(|a, b| {
                Vec3::from(a.1.position).distance(eye).total_cmp(&Vec3::from(b.1.position).distance(eye))
            })
            .map(|(i, _)| i);
        self.shadowed = shadowed.map(|i| (Vec3::from(lights[i].position), lights[i].radius));

        let mut uniform = LightsUniform::zeroed();
        uniform.count[0] = lights.len() as u32;
        for (i, (slot, light)) in uniform.lights.iter_mut().zip(lights).enumerate() {
            slot.position_radius = [light.position[0], light.position[1], light.position[2], light.radius];
            slot.colour_shadow = [light.colour[0], light.colour[1], light.colour[2], if shadowed == Some(i) { 1.0 } else { 0.0 }];
        }
        gpu.queue.write_buffer(&self.uniform, 0, bytemuck::bytes_of(&uniform));

        let (cx, cy, cz) = CLUSTERS;
        for (v, view) in views.iter().enumerate() {
            let mut masks = vec![[0u32; 2]; (cx * cy * cz) as usize];
            let view_matrix = view.camera.view_matrix();
            let tan_y = (view.camera.fov_y * 0.5).tan();
            let tan_x = tan_y * aspects[v];
            for (i, light) in lights.iter().enumerate() {
                let p = view_matrix.transform_point3(Vec3::from(light.position));
                let r = light.radius;
                let (near, far) = (-p.z - r, -p.z + r);
                if far < CLUSTER_DEPTH.0 || near > CLUSTER_DEPTH.1 {
                    continue;
                }
                let slice = |d: f32| {
                    let t = (d.max(CLUSTER_DEPTH.0) / CLUSTER_DEPTH.0).ln() / (CLUSTER_DEPTH.1 / CLUSTER_DEPTH.0).ln();
                    ((t * cz as f32).floor() as i32).clamp(0, cz as i32 - 1) as u32
                };
                for z in slice(near)..=slice(far) {
                    // The slice's depth range, and so how wide the view is across it.
                    let d0 = CLUSTER_DEPTH.0 * (CLUSTER_DEPTH.1 / CLUSTER_DEPTH.0).powf(z as f32 / cz as f32);
                    let d1 = CLUSTER_DEPTH.0 * (CLUSTER_DEPTH.1 / CLUSTER_DEPTH.0).powf((z + 1) as f32 / cz as f32);
                    for y in 0..cy {
                        for x in 0..cx {
                            // The cluster's box in view space, taken at both ends of the slice.
                            let edge = |i: u32, n: u32, tan: f32, d: f32| ((i as f32 / n as f32) * 2.0 - 1.0) * tan * d;
                            let xs = [edge(x, cx, tan_x, d0), edge(x + 1, cx, tan_x, d0), edge(x, cx, tan_x, d1), edge(x + 1, cx, tan_x, d1)];
                            // Row 0 is the top of the screen, which is +y in view space.
                            let ys = [edge(cy - y, cy, tan_y, d0), edge(cy - y - 1, cy, tan_y, d0), edge(cy - y, cy, tan_y, d1), edge(cy - y - 1, cy, tan_y, d1)];
                            let (x0, x1) = (xs.iter().copied().fold(f32::MAX, f32::min), xs.iter().copied().fold(f32::MIN, f32::max));
                            let (y0, y1) = (ys.iter().copied().fold(f32::MAX, f32::min), ys.iter().copied().fold(f32::MIN, f32::max));
                            let nearest = Vec3::new(p.x.clamp(x0, x1), p.y.clamp(y0, y1), p.z.clamp(-d1, -d0));
                            if nearest.distance_squared(p) <= r * r {
                                masks[(y * cx * cz + z * cx + x) as usize][i / 32] |= 1 << (i % 32);
                            }
                        }
                    }
                }
            }
            gpu.queue.write_texture(
                wgpu::TexelCopyTextureInfo {
                    texture: &self.cluster_texture,
                    mip_level: 0,
                    origin: wgpu::Origin3d { x: 0, y: 0, z: v as u32 },
                    aspect: wgpu::TextureAspect::All,
                },
                bytemuck::cast_slice(&masks),
                wgpu::TexelCopyBufferLayout { offset: 0, bytes_per_row: Some(cx * cz * 8), rows_per_image: Some(cy) },
                wgpu::Extent3d { width: cx * cz, height: cy, depth_or_array_layers: 1 },
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_cube_face_sees_its_own_direction() {
        let light = Vec3::new(1.0, 2.0, 3.0);
        for (face, dir) in [Vec3::X, Vec3::NEG_X, Vec3::Y, Vec3::NEG_Y, Vec3::Z, Vec3::NEG_Z].into_iter().enumerate() {
            let m = cube_face_matrix(light, 10.0, face);
            let clip = m.project_point3(light + dir * 5.0);
            assert!(clip.x.abs() < 1e-4 && clip.y.abs() < 1e-4, "face {face}: {clip:?}");
            assert!(clip.z > 0.0 && clip.z < 1.0);
        }
    }

    #[test]
    fn cube_faces_are_laid_out_as_a_cube_map_reads_them() {
        // On the +X face a cube map's first coordinate grows towards -Z and its
        // second, down the picture, grows towards -Y.
        let m = cube_face_matrix(Vec3::ZERO, 10.0, 0);
        let towards_minus_z = m.project_point3(Vec3::new(5.0, 0.0, -1.0));
        assert!(towards_minus_z.x > 0.0);
        let towards_minus_y = m.project_point3(Vec3::new(5.0, -1.0, 0.0));
        // Clip y is up, the picture's rows run down: lower on screen is negative y.
        assert!(towards_minus_y.y < 0.0);
    }
}
