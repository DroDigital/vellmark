//! Scenes: `assets/scenes/<name>.ron`. A scene is authored as boxes with materials,
//! and baked into one mesh for everything that never moves and one for each thing
//! that can.

use crate::materials::material_id;
use crate::{sorted_files, BakeError};
use serde::Deserialize;
use std::path::Path;
use vellmark_content::{AssetId, Light, MaterialSet, Mesh, Object, Scene, Vertex};

type V3 = (f32, f32, f32);

#[derive(Deserialize)]
struct AuthoredBox {
    material: String,
    min: V3,
    max: V3,
    /// Degrees about the box's own centre. A turned box carries its page with it.
    #[serde(default)]
    yaw: f32,
}

#[derive(Deserialize)]
struct AuthoredObject {
    key: String,
    /// Where the object's own origin sits in the scene.
    position: V3,
    #[serde(default)]
    yaw: f32,
    /// In the object's own space.
    boxes: Vec<AuthoredBox>,
}

#[derive(Deserialize)]
struct AuthoredLight {
    key: String,
    position: V3,
    colour: V3,
    strength: f32,
    radius: f32,
    #[serde(default)]
    casts_shadow: bool,
}

#[derive(Deserialize)]
struct AuthoredSpawn {
    eye: V3,
    yaw: f32,
    pitch: f32,
}

#[derive(Deserialize)]
struct Authored {
    spawn: AuthoredSpawn,
    boxes: Vec<AuthoredBox>,
    #[serde(default)]
    objects: Vec<AuthoredObject>,
    #[serde(default)]
    lights: Vec<AuthoredLight>,
}

/// The six faces of a box: normal, the direction the page's first coordinate grows
/// (reading direction, seen from outside), and the direction its second grows (up).
const FACES: [([f32; 3], [f32; 3], [f32; 3]); 6] = [
    ([0.0, 0.0, 1.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
    ([0.0, 0.0, -1.0], [-1.0, 0.0, 0.0], [0.0, 1.0, 0.0]),
    ([1.0, 0.0, 0.0], [0.0, 0.0, -1.0], [0.0, 1.0, 0.0]),
    ([-1.0, 0.0, 0.0], [0.0, 0.0, 1.0], [0.0, 1.0, 0.0]),
    ([0.0, 1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, -1.0]),
    ([0.0, -1.0, 0.0], [1.0, 0.0, 0.0], [0.0, 0.0, 1.0]),
];

fn dot(a: [f32; 3], b: [f32; 3]) -> f32 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

fn rotate_y(v: [f32; 3], degrees: f32) -> [f32; 3] {
    let (s, c) = degrees.to_radians().sin_cos();
    [v[0] * c + v[2] * s, v[1], -v[0] * s + v[2] * c]
}

fn add_box(mesh: &mut Mesh, b: &AuthoredBox, material: u32) {
    let (min, max) = ([b.min.0, b.min.1, b.min.2], [b.max.0, b.max.1, b.max.2]);
    let centre = [(min[0] + max[0]) * 0.5, (min[1] + max[1]) * 0.5, (min[2] + max[2]) * 0.5];
    let half = [(max[0] - min[0]) * 0.5, (max[1] - min[1]) * 0.5, (max[2] - min[2]) * 0.5];
    for (normal, tangent, up) in FACES {
        let first = mesh.vertices.len() as u32;
        for (su, sv) in [(-1.0f32, -1.0f32), (1.0, -1.0), (1.0, 1.0), (-1.0, 1.0)] {
            // A corner of this face, about the box's centre and before any turn.
            let local: [f32; 3] = std::array::from_fn(|i| (normal[i] + tangent[i] * su + up[i] * sv) * half[i]);
            let unturned: [f32; 3] = std::array::from_fn(|i| centre[i] + local[i]);
            let turned = rotate_y(local, b.yaw);
            // The page is laid on the grid of the space the box was authored in, so that
            // text runs unbroken across boxes that share a plane. A turned box keeps the
            // page it would have had unturned.
            let anchor = [dot(unturned, tangent), dot(unturned, up)];
            let t = rotate_y(tangent, b.yaw);
            mesh.vertices.push(Vertex {
                position: std::array::from_fn(|i| centre[i] + turned[i]),
                normal: rotate_y(normal, b.yaw),
                tangent: [t[0], t[1], t[2], 1.0],
                anchor,
                material,
            });
        }
        mesh.indices.extend_from_slice(&[first, first + 1, first + 2, first, first + 2, first + 3]);
    }
}

fn build(boxes: &[AuthoredBox], set: &MaterialSet, file: &Path) -> Result<Mesh, BakeError> {
    let mut mesh = Mesh::default();
    for b in boxes {
        let material = set
            .index_of(material_id(&b.material))
            .ok_or_else(|| BakeError(format!("{}: no material named `{}`", file.display(), b.material)))?;
        if b.min.0 >= b.max.0 || b.min.1 >= b.max.1 || b.min.2 >= b.max.2 {
            return Err(BakeError(format!("{}: a `{}` box has no size", file.display(), b.material)));
        }
        add_box(&mut mesh, b, material);
    }
    Ok(mesh)
}

pub struct BakedScene {
    pub key: String,
    pub scene: Scene,
    pub meshes: Vec<(AssetId, Mesh)>,
}

pub fn bake_all(dir: &Path, set: &MaterialSet) -> Result<Vec<BakedScene>, BakeError> {
    let mut out = Vec::new();
    for file in sorted_files(dir, "ron")? {
        let name = file.file_stem().and_then(|s| s.to_str()).unwrap_or_default().to_owned();
        let text = std::fs::read_to_string(&file).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
        let authored: Authored = ron::from_str(&text).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;

        let mut meshes = Vec::new();
        let static_mesh = AssetId::from_key(&format!("scene.{name}.static"));
        meshes.push((static_mesh, build(&authored.boxes, set, &file)?));

        let mut objects = Vec::new();
        for o in &authored.objects {
            let mesh = AssetId::from_key(&format!("scene.{name}.{}.mesh", o.key));
            meshes.push((mesh, build(&o.boxes, set, &file)?));
            let (s, c) = o.yaw.to_radians().sin_cos();
            objects.push(Object {
                key: AssetId::from_key(&format!("scene.{name}.{}", o.key)),
                mesh,
                transform: [c, 0.0, s, o.position.0, 0.0, 1.0, 0.0, o.position.1, -s, 0.0, c, o.position.2],
            });
        }
        let lights = authored
            .lights
            .iter()
            .map(|l| Light {
                key: AssetId::from_key(&format!("scene.{name}.{}", l.key)),
                position: [l.position.0, l.position.1, l.position.2],
                colour: [l.colour.0 * l.strength, l.colour.1 * l.strength, l.colour.2 * l.strength],
                radius: l.radius,
                casts_shadow: l.casts_shadow,
            })
            .collect();
        let scene = Scene {
            static_mesh,
            objects,
            lights,
            spawn: [
                authored.spawn.eye.0,
                authored.spawn.eye.1,
                authored.spawn.eye.2,
                authored.spawn.yaw.to_radians(),
                authored.spawn.pitch.to_radians(),
            ],
        };
        out.push(BakedScene { key: format!("scene.{name}"), scene, meshes });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [a[1] * b[2] - a[2] * b[1], a[2] * b[0] - a[0] * b[2], a[0] * b[1] - a[1] * b[0]]
    }

    #[test]
    fn every_face_is_right_handed_and_upright() {
        for (normal, tangent, up) in FACES {
            assert_eq!(cross(tangent, up), normal);
            // On a wall, the page's second coordinate is the world's up.
            if normal[1] == 0.0 {
                assert_eq!(up, [0.0, 1.0, 0.0]);
            }
        }
    }

    #[test]
    fn a_box_faces_outwards_and_shares_its_page_with_its_neighbour() {
        let unit = |min: V3, max: V3| AuthoredBox { material: String::new(), min, max, yaw: 0.0 };
        let mut mesh = Mesh::default();
        add_box(&mut mesh, &unit((0.0, 0.0, 0.0), (2.0, 1.0, 1.0)), 0);
        add_box(&mut mesh, &unit((2.0, 0.0, 0.0), (5.0, 1.0, 1.0)), 0);
        assert_eq!(mesh.vertices.len(), 48);
        for tri in mesh.indices.chunks_exact(3) {
            let [a, b, c] = [mesh.vertices[tri[0] as usize], mesh.vertices[tri[1] as usize], mesh.vertices[tri[2] as usize]];
            let e1: [f32; 3] = std::array::from_fn(|i| b.position[i] - a.position[i]);
            let e2: [f32; 3] = std::array::from_fn(|i| c.position[i] - a.position[i]);
            assert!(dot(cross(e1, e2), a.normal) > 0.0, "triangles wind counter-clockwise seen from outside");
        }
        // On the south face (+Z) the page's first coordinate is world x, on both boxes.
        for v in mesh.vertices.iter().filter(|v| v.normal == [0.0, 0.0, 1.0]) {
            assert_eq!(v.anchor, [v.position[0], v.position[1]]);
        }
    }
}
