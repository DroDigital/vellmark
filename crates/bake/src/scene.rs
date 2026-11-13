//! Scenes: `assets/scenes/<name>.ron`. A scene is authored as boxes with materials,
//! and baked into one mesh for everything that never moves and one for each thing
//! that can.

use crate::materials::material_id;
use crate::pages::{CellRect, Layout};
use crate::{BakeError, sorted_files};
use serde::Deserialize;
use std::collections::BTreeMap;
use std::path::Path;
use vellmark_content::{
    AssetId, Breakable, Light, MaterialSet, Mesh, Object, PageEntry, Scene, SolidBox, TextPages,
    Vertex,
};

type V3 = (f32, f32, f32);

#[derive(Deserialize)]
struct AuthoredBox {
    material: String,
    min: V3,
    max: V3,
    /// Degrees about the box's own centre. A turned box carries its page with it.
    #[serde(default)]
    yaw: f32,
    /// A name shared by boxes that carry one piece of authored text between them.
    #[serde(default)]
    tag: Option<String>,
}

#[derive(Deserialize, Clone, Copy, PartialEq, Eq, Debug)]
enum Facing {
    South,
    North,
    East,
    West,
    Up,
}

impl Facing {
    fn index(self) -> usize {
        match self {
            Facing::South => 0,
            Facing::North => 1,
            Facing::East => 2,
            Facing::West => 3,
            Facing::Up => 4,
        }
    }
}

/// One stretch of surface a text runs across: the faces, looking one way, of every
/// box with a tag.
#[derive(Deserialize)]
struct AuthoredRun {
    tag: String,
    facing: Facing,
    /// The block the text may fill, in metres of the surface's page: left, bottom,
    /// right, top. Without it the text may fill the whole surface.
    #[serde(default)]
    block: Option<(f32, f32, f32, f32)>,
}

#[derive(Deserialize)]
struct AuthoredText {
    /// The key of the text in the string tables.
    string: String,
    /// The cascade level it is set in: 0 is 0.64 cm letters, 1 is 2.56 cm.
    level: u32,
    /// The stretches of surface it runs across, in reading order.
    runs: Vec<AuthoredRun>,
    /// Rows from one line to the next. 2 leaves a blank row between lines.
    #[serde(default = "one_row")]
    leading: u32,
    /// Lines only a sharp enough reader sees, shown when the text is read: each a string
    /// key and the Perception it takes.
    #[serde(default)]
    hidden: Vec<(String, u32)>,
}

fn one_row() -> u32 {
    1
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
    /// In kilograms, for a loose prop that falls and can be carried. Left out, the
    /// object is fixed in place.
    #[serde(default)]
    mass: f32,
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
    /// The terrain the scene stands on, by name: `valley` for `terrain/valley.ron`.
    #[serde(default)]
    terrain: Option<String>,
    #[serde(default)]
    boxes: Vec<AuthoredBox>,
    #[serde(default)]
    objects: Vec<AuthoredObject>,
    #[serde(default)]
    lights: Vec<AuthoredLight>,
    #[serde(default)]
    texts: Vec<AuthoredText>,
    /// The things that carry Inscriptions.
    #[serde(default)]
    inscribed: Vec<AuthoredInscribed>,
}

#[derive(Deserialize)]
struct AuthoredInscribed {
    /// An object's or light's key.
    thing: String,
    /// Its target class, as in `assets/data/words/targets.ron`.
    class: String,
    /// Its words, each with its class.
    #[serde(default)]
    words: Vec<(String, String)>,
    /// 0 Plain, 1 Tough, 2 Hard, 3 Sealed.
    #[serde(default)]
    resistance: u8,
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

/// The side of the cells a box's faces are divided into, so that the sky each point
/// sees can change across a face: darker at the foot of a wall, lighter at its top.
const FACE_CELL: f32 = 0.5;

/// `page_of` gives the text page, counted from 1, of a tagged box's face.
fn add_box(mesh: &mut Mesh, b: &AuthoredBox, material: u32, page_of: &dyn Fn(&str, usize) -> u32) {
    let (min, max) = ([b.min.0, b.min.1, b.min.2], [b.max.0, b.max.1, b.max.2]);
    let centre = [
        (min[0] + max[0]) * 0.5,
        (min[1] + max[1]) * 0.5,
        (min[2] + max[2]) * 0.5,
    ];
    let half = [
        (max[0] - min[0]) * 0.5,
        (max[1] - min[1]) * 0.5,
        (max[2] - min[2]) * 0.5,
    ];
    for (face, (normal, tangent, up)) in FACES.into_iter().enumerate() {
        let first = mesh.vertices.len() as u32;
        let page = b.tag.as_deref().map(|tag| page_of(tag, face)).unwrap_or(0);
        let extent = |axis: [f32; 3]| {
            2.0 * (axis[0].abs() * half[0] + axis[1].abs() * half[1] + axis[2].abs() * half[2])
        };
        let cells = |axis: [f32; 3]| ((extent(axis) / FACE_CELL).ceil() as u32).clamp(1, 64);
        let (nu, nv) = (cells(tangent), cells(up));
        for jv in 0..=nv {
            for iu in 0..=nu {
                let (su, sv) = (
                    iu as f32 / nu as f32 * 2.0 - 1.0,
                    jv as f32 / nv as f32 * 2.0 - 1.0,
                );
                // A point of this face, about the box's centre and before any turn.
                let local: [f32; 3] =
                    std::array::from_fn(|i| (normal[i] + tangent[i] * su + up[i] * sv) * half[i]);
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
                    page,
                    // The box's extent along this face's normal.
                    thickness: extent(normal),
                    sky: 1.0,
                });
            }
        }
        let row = nu + 1;
        for jv in 0..nv {
            for iu in 0..nu {
                let a = first + jv * row + iu;
                mesh.indices
                    .extend_from_slice(&[a, a + 1, a + row + 1, a, a + row + 1, a + row]);
            }
        }
    }
}

/// Bakes the share of the sky each vertex sees: rays over its hemisphere, spread by the
/// cosine, against every box of the scene. `to_world` places the mesh.
fn bake_sky(
    mesh: &mut Mesh,
    to_world: &dyn Fn([f32; 3], bool) -> [f32; 3],
    occluders: &[([f32; 3], [f32; 3])],
) {
    const RAYS: usize = 48;
    const REACH: f32 = 25.0;
    // Directions spread evenly over a hemisphere about +Z, by the cosine: a Fibonacci
    // spiral on the disc, lifted.
    let golden = std::f32::consts::PI * (3.0 - 5.0f32.sqrt());
    let local: Vec<[f32; 3]> = (0..RAYS)
        .map(|k| {
            let r = ((k as f32 + 0.5) / RAYS as f32).sqrt();
            let a = k as f32 * golden;
            [r * a.cos(), r * a.sin(), (1.0 - r * r).max(0.0).sqrt()]
        })
        .collect();
    let hits = |origin: [f32; 3], dir: [f32; 3]| {
        occluders.iter().any(|(lo, hi)| {
            let (mut near, mut far) = (0.0f32, REACH);
            for i in 0..3 {
                if dir[i].abs() < 1e-6 {
                    if origin[i] < lo[i] || origin[i] > hi[i] {
                        return false;
                    }
                    continue;
                }
                let (t0, t1) = ((lo[i] - origin[i]) / dir[i], (hi[i] - origin[i]) / dir[i]);
                near = near.max(t0.min(t1));
                far = far.min(t0.max(t1));
                if near > far {
                    return false;
                }
            }
            true
        })
    };
    for v in &mut mesh.vertices {
        let n = to_world(v.normal, false);
        let p = to_world(v.position, true);
        let origin = [p[0] + n[0] * 0.02, p[1] + n[1] * 0.02, p[2] + n[2] * 0.02];
        // A frame about the normal.
        let helper = if n[1].abs() < 0.9 {
            [0.0, 1.0, 0.0]
        } else {
            [1.0, 0.0, 0.0]
        };
        let cross = |a: [f32; 3], b: [f32; 3]| {
            [
                a[1] * b[2] - a[2] * b[1],
                a[2] * b[0] - a[0] * b[2],
                a[0] * b[1] - a[1] * b[0],
            ]
        };
        let norm = |a: [f32; 3]| {
            let l = (a[0] * a[0] + a[1] * a[1] + a[2] * a[2]).sqrt();
            [a[0] / l, a[1] / l, a[2] / l]
        };
        let tx = norm(cross(helper, n));
        let ty = cross(n, tx);
        let open = local
            .iter()
            .filter(|d| {
                let dir: [f32; 3] =
                    std::array::from_fn(|i| tx[i] * d[0] + ty[i] * d[1] + n[i] * d[2]);
                !hits(origin, dir)
            })
            .count();
        v.sky = open as f32 / RAYS as f32;
    }
}

fn build(
    boxes: &[AuthoredBox],
    set: &MaterialSet,
    file: &Path,
    page_of: &dyn Fn(&str, usize) -> u32,
) -> Result<Mesh, BakeError> {
    let mut mesh = Mesh::default();
    for b in boxes {
        let material = set.index_of(material_id(&b.material)).ok_or_else(|| {
            BakeError(format!(
                "{}: no material named `{}`",
                file.display(),
                b.material
            ))
        })?;
        if b.min.0 >= b.max.0 || b.min.1 >= b.max.1 || b.min.2 >= b.max.2 {
            return Err(BakeError(format!(
                "{}: a `{}` box has no size",
                file.display(),
                b.material
            )));
        }
        add_box(&mut mesh, b, material, page_of);
    }
    Ok(mesh)
}

/// A box as the simulation sees it. A turned box is given the box that bounds it.
fn solid(b: &AuthoredBox, set: &MaterialSet) -> SolidBox {
    let centre = [
        (b.min.0 + b.max.0) * 0.5,
        (b.min.1 + b.max.1) * 0.5,
        (b.min.2 + b.max.2) * 0.5,
    ];
    let half = [
        (b.max.0 - b.min.0) * 0.5,
        (b.max.1 - b.min.1) * 0.5,
        (b.max.2 - b.min.2) * 0.5,
    ];
    let (sin, cos) = b.yaw.to_radians().sin_cos();
    let (hx, hz) = (
        half[0] * cos.abs() + half[2] * sin.abs(),
        half[0] * sin.abs() + half[2] * cos.abs(),
    );
    SolidBox {
        min: [centre[0] - hx, centre[1] - half[1], centre[2] - hz],
        max: [centre[0] + hx, centre[1] + half[1], centre[2] + hz],
        material: set.index_of(material_id(&b.material)).unwrap_or(0),
    }
}

pub struct BakedScene {
    pub key: String,
    pub scene: Scene,
    pub meshes: Vec<(AssetId, Mesh)>,
    pub pages: Option<TextPages>,
    /// The texts that can be read with the Read action.
    pub readables: vellmark_content::Readables,
    /// The things that carry Inscriptions.
    pub inscribed: vellmark_content::InscribedThings,
}

/// The scene's things that carry Inscriptions: each object's sphere from its boxes, or a
/// light's from where it hangs.
fn inscribed_of(
    authored: &Authored,
    name: &str,
    file: &Path,
) -> Result<vellmark_content::InscribedThings, BakeError> {
    let mut items = Vec::new();
    for i in &authored.inscribed {
        let (centre, radius) = if let Some(o) = authored.objects.iter().find(|o| o.key == i.thing) {
            let (s, c) = o.yaw.to_radians().sin_cos();
            let mut lo = [f32::MAX; 3];
            let mut hi = [f32::MIN; 3];
            for b in &o.boxes {
                for corner in 0..8 {
                    let p = [
                        if corner & 1 == 0 { b.min.0 } else { b.max.0 },
                        if corner & 2 == 0 { b.min.1 } else { b.max.1 },
                        if corner & 4 == 0 { b.min.2 } else { b.max.2 },
                    ];
                    let w = [
                        c * p[0] + s * p[2] + o.position.0,
                        p[1] + o.position.1,
                        -s * p[0] + c * p[2] + o.position.2,
                    ];
                    for k in 0..3 {
                        lo[k] = lo[k].min(w[k]);
                        hi[k] = hi[k].max(w[k]);
                    }
                }
            }
            let centre = [
                (lo[0] + hi[0]) * 0.5,
                (lo[1] + hi[1]) * 0.5,
                (lo[2] + hi[2]) * 0.5,
            ];
            let half = [
                (hi[0] - lo[0]) * 0.5,
                (hi[1] - lo[1]) * 0.5,
                (hi[2] - lo[2]) * 0.5,
            ];
            (
                centre,
                (half[0] * half[0] + half[1] * half[1] + half[2] * half[2]).sqrt() * 0.75,
            )
        } else if let Some(l) = authored.lights.iter().find(|l| l.key == i.thing) {
            ([l.position.0, l.position.1, l.position.2], 0.35)
        } else {
            return Err(BakeError(format!(
                "{}: nothing called `{}` to inscribe",
                file.display(),
                i.thing
            )));
        };
        if i.words.len() > 3 {
            return Err(BakeError(format!(
                "{}: `{}` has more than three words",
                file.display(),
                i.thing
            )));
        }
        items.push(vellmark_content::InscribedThing {
            id: AssetId::from_key(&format!("scene.{name}.{}", i.thing)),
            class: i.class.clone(),
            centre,
            radius,
            resistance: i.resistance.min(3),
            words: i.words.clone(),
        });
    }
    Ok(vellmark_content::InscribedThings { items })
}

/// Every authored text of a scene as a thing to be read: the box round the boxes its
/// runs are set on, in the world.
fn readables_of(
    authored: &Authored,
    name: &str,
    strings: &BTreeMap<String, String>,
    file: &Path,
) -> Result<vellmark_content::Readables, BakeError> {
    use vellmark_content::{Readable, StringId};
    let mut items = Vec::new();
    for text in &authored.texts {
        let mut lo = [f32::MAX; 3];
        let mut hi = [f32::MIN; 3];
        let mut grow = |p: [f32; 3]| {
            for i in 0..3 {
                lo[i] = lo[i].min(p[i]);
                hi[i] = hi[i].max(p[i]);
            }
        };
        for run in &text.runs {
            for b in authored
                .boxes
                .iter()
                .filter(|b| b.tag.as_deref() == Some(run.tag.as_str()))
            {
                grow([b.min.0, b.min.1, b.min.2]);
                grow([b.max.0, b.max.1, b.max.2]);
            }
            for o in &authored.objects {
                let (s, c) = o.yaw.to_radians().sin_cos();
                for b in o
                    .boxes
                    .iter()
                    .filter(|b| b.tag.as_deref() == Some(run.tag.as_str()))
                {
                    for corner in 0..8 {
                        let p = [
                            if corner & 1 == 0 { b.min.0 } else { b.max.0 },
                            if corner & 2 == 0 { b.min.1 } else { b.max.1 },
                            if corner & 4 == 0 { b.min.2 } else { b.max.2 },
                        ];
                        grow([
                            c * p[0] + s * p[2] + o.position.0,
                            p[1] + o.position.1,
                            -s * p[0] + c * p[2] + o.position.2,
                        ]);
                    }
                }
            }
        }
        if lo[0] > hi[0] {
            continue;
        }
        for (key, _) in &text.hidden {
            if !strings.contains_key(key) {
                return Err(BakeError(format!(
                    "{}: no string `{key}` in assets/text",
                    file.display()
                )));
            }
        }
        items.push(Readable {
            id: AssetId::from_key(&format!("scene.{name}.text.{}", text.string)),
            text: StringId::from_key(&text.string),
            min: lo,
            max: hi,
            hidden: text
                .hidden
                .iter()
                .map(|(k, p)| (StringId::from_key(k), *p))
                .collect(),
        });
    }
    Ok(vellmark_content::Readables { items })
}

/// The rectangle a box's face covers on its page, in metres: left, bottom, right, top.
fn face_rect(b: &AuthoredBox, face: usize) -> (f32, f32, f32, f32) {
    let (_, tangent, up) = FACES[face];
    let (min, max) = ([b.min.0, b.min.1, b.min.2], [b.max.0, b.max.1, b.max.2]);
    let (u0, u1) = (dot(min, tangent), dot(max, tangent));
    let (v0, v1) = (dot(min, up), dot(max, up));
    (u0.min(u1), v0.min(v1), u0.max(u1), v0.max(v1))
}

/// Lays every authored text of a scene onto its surfaces. Returns the pages and, for
/// each tagged face, which page it shows.
fn lay_out_texts(
    authored: &Authored,
    strings: &BTreeMap<String, String>,
    file: &Path,
) -> Result<(TextPages, BTreeMap<(String, usize), u32>), BakeError> {
    const SHEET: u32 = 1024;
    let mut pages = TextPages {
        entries: Vec::new(),
        sheet_size: SHEET,
        sheet: vec![0; (SHEET * SHEET) as usize],
    };
    let mut page_of = BTreeMap::new();
    let (mut shelf_x, mut shelf_y, mut shelf_h) = (0u32, 0u32, 0u32);
    for text in &authored.texts {
        let source = strings.get(&text.string).ok_or_else(|| {
            BakeError(format!(
                "{}: no string `{}` in assets/text",
                file.display(),
                text.string
            ))
        })?;
        let words: Vec<&str> = source.split_whitespace().collect();
        let cell = 0.0064 * 4f32.powi(text.level as i32);
        let to_cells = |m: f32, up: bool| {
            if up {
                (m / cell - 1.0e-3).ceil() as i32
            } else {
                (m / cell + 1.0e-3).floor() as i32
            }
        };
        let mut next_word = 0;
        for run in &text.runs {
            let face = run.facing.index();
            // Every box in the scene with this tag, static or part of an object.
            let boxes = authored
                .boxes
                .iter()
                .chain(authored.objects.iter().flat_map(|o| o.boxes.iter()));
            let rects: Vec<(f32, f32, f32, f32)> = boxes
                .filter(|b| b.tag.as_deref() == Some(run.tag.as_str()))
                .map(|b| face_rect(b, face))
                .collect();
            if rects.is_empty() {
                return Err(BakeError(format!(
                    "{}: no box is tagged `{}`",
                    file.display(),
                    run.tag
                )));
            }
            let surface: Vec<(f32, f32, f32, f32)> = rects
                .iter()
                .map(|r| (r.0 / cell, r.1 / cell, r.2 / cell, r.3 / cell))
                .collect();
            let bounds = rects
                .iter()
                .fold((f32::MAX, f32::MAX, f32::MIN, f32::MIN), |a, r| {
                    (a.0.min(r.0), a.1.min(r.1), a.2.max(r.2), a.3.max(r.3))
                });
            let b = run.block.unwrap_or(bounds);
            let block = CellRect {
                x0: to_cells(b.0, true),
                y0: to_cells(b.1, true),
                x1: to_cells(b.2, false),
                y1: to_cells(b.3, false),
            };
            let mut layout = Layout::on_surface(block, &surface);
            layout.leading = text.leading.max(1);
            // Start a row down, so that the plate has a margin above the first line.
            next_word += layout.flow(&words[next_word..], &mut (1, 1));
            layout.clear_plate();

            if shelf_x + layout.width > SHEET {
                shelf_x = 0;
                shelf_y += shelf_h;
                shelf_h = 0;
            }
            if layout.width > SHEET || shelf_y + layout.height > SHEET - TextPages::GENERATED_ROWS {
                return Err(BakeError(format!(
                    "{}: the text `{}` needs a page larger than the sheet",
                    file.display(),
                    text.string
                )));
            }
            for row in 0..layout.height {
                let from = (row * layout.width) as usize;
                let to = ((shelf_y + row) * SHEET + shelf_x) as usize;
                pages.sheet[to..to + layout.width as usize]
                    .copy_from_slice(&layout.chars[from..from + layout.width as usize]);
            }
            pages.entries.push(PageEntry {
                level: text.level,
                left: layout.left,
                top: layout.top,
                width: layout.width,
                height: layout.height,
                sheet_x: shelf_x,
                sheet_y: shelf_y,
            });
            if page_of
                .insert((run.tag.clone(), face), pages.entries.len() as u32)
                .is_some()
            {
                return Err(BakeError(format!(
                    "{}: two texts are set on the {:?} faces of `{}`",
                    file.display(),
                    run.facing,
                    run.tag
                )));
            }
            shelf_x += layout.width;
            shelf_h = shelf_h.max(layout.height);
        }
        if next_word < words.len() {
            return Err(BakeError(format!(
                "{}: the text `{}` does not fit its surface: {} of {} words were set",
                file.display(),
                text.string,
                next_word,
                words.len()
            )));
        }
    }
    if pages.entries.len() > TextPages::MAX_PAGES - TextPages::GENERATED_PAGES {
        return Err(BakeError(format!(
            "{}: more than {} text pages",
            file.display(),
            TextPages::MAX_PAGES - TextPages::GENERATED_PAGES
        )));
    }
    Ok((pages, page_of))
}

pub fn bake_all(
    dir: &Path,
    set: &MaterialSet,
    strings: &BTreeMap<String, String>,
) -> Result<Vec<BakedScene>, BakeError> {
    let mut out = Vec::new();
    for file in sorted_files(dir, "ron")? {
        let name = file
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or_default()
            .to_owned();
        let text = std::fs::read_to_string(&file)
            .map_err(|e| BakeError(format!("{}: {e}", file.display())))?;
        let authored: Authored =
            ron::from_str(&text).map_err(|e| BakeError(format!("{}: {e}", file.display())))?;

        let (pages, page_index) = lay_out_texts(&authored, strings, &file)?;
        let page_of = |tag: &str, face: usize| {
            page_index
                .get(&(tag.to_owned(), face))
                .copied()
                .unwrap_or(0)
        };

        let mut meshes = Vec::new();
        let static_mesh = AssetId::from_key(&format!("scene.{name}.static"));
        meshes.push((static_mesh, build(&authored.boxes, set, &file, &page_of)?));

        let colliders_list: Vec<SolidBox> = authored.boxes.iter().map(|b| solid(b, set)).collect();
        let colliders = colliders_list.clone();
        let mut breakables = Vec::new();
        let mut objects = Vec::new();
        for o in &authored.objects {
            let mesh = AssetId::from_key(&format!("scene.{name}.{}.mesh", o.key));
            meshes.push((mesh, build(&o.boxes, set, &file, &page_of)?));
            let (s, c) = o.yaw.to_radians().sin_cos();
            let transform = [
                c,
                0.0,
                s,
                o.position.0,
                0.0,
                1.0,
                0.0,
                o.position.1,
                -s,
                0.0,
                c,
                o.position.2,
            ];
            let key = AssetId::from_key(&format!("scene.{name}.{}", o.key));
            if !(o.mass >= 0.0 && o.mass < 2000.0) {
                return Err(BakeError(format!(
                    "{}: object {} weighs {} kg; a loose prop weighs between 0 and 2000",
                    file.display(),
                    o.key,
                    o.mass
                )));
            }
            breakables.push(Breakable {
                key,
                transform,
                boxes: o.boxes.iter().map(|b| solid(b, set)).collect(),
                mass: o.mass,
            });
            objects.push(Object {
                key,
                mesh,
                transform,
            });
        }
        // The sky each vertex sees, against every box of the scene: the still boxes, and
        // the objects' boxes where they stand.
        let mut occluders: Vec<([f32; 3], [f32; 3])> = colliders_list
            .iter()
            .map(|c: &SolidBox| (c.min, c.max))
            .collect();
        for breakable in &breakables {
            let t = breakable.transform;
            for bx in &breakable.boxes {
                let mut lo = [f32::MAX; 3];
                let mut hi = [f32::MIN; 3];
                for corner in 0..8 {
                    let p = [
                        if corner & 1 == 0 {
                            bx.min[0]
                        } else {
                            bx.max[0]
                        },
                        if corner & 2 == 0 {
                            bx.min[1]
                        } else {
                            bx.max[1]
                        },
                        if corner & 4 == 0 {
                            bx.min[2]
                        } else {
                            bx.max[2]
                        },
                    ];
                    let w = [
                        t[0] * p[0] + t[1] * p[1] + t[2] * p[2] + t[3],
                        t[4] * p[0] + t[5] * p[1] + t[6] * p[2] + t[7],
                        t[8] * p[0] + t[9] * p[1] + t[10] * p[2] + t[11],
                    ];
                    for i in 0..3 {
                        lo[i] = lo[i].min(w[i]);
                        hi[i] = hi[i].max(w[i]);
                    }
                }
                occluders.push((lo, hi));
            }
        }
        let identity = |p: [f32; 3], _point: bool| p;
        bake_sky(&mut meshes[0].1, &identity, &occluders);
        for (i, object) in objects.iter().enumerate() {
            let t = object.transform;
            let place = move |p: [f32; 3], point: bool| {
                let w = if point { 1.0 } else { 0.0 };
                [
                    t[0] * p[0] + t[1] * p[1] + t[2] * p[2] + t[3] * w,
                    t[4] * p[0] + t[5] * p[1] + t[6] * p[2] + t[7] * w,
                    t[8] * p[0] + t[9] * p[1] + t[10] * p[2] + t[11] * w,
                ]
            };
            bake_sky(&mut meshes[i + 1].1, &place, &occluders);
        }
        let lights = authored
            .lights
            .iter()
            .map(|l| Light {
                key: AssetId::from_key(&format!("scene.{name}.{}", l.key)),
                position: [l.position.0, l.position.1, l.position.2],
                colour: [
                    l.colour.0 * l.strength,
                    l.colour.1 * l.strength,
                    l.colour.2 * l.strength,
                ],
                radius: l.radius,
                casts_shadow: l.casts_shadow,
            })
            .collect();
        let scene = Scene {
            static_mesh,
            objects,
            lights,
            colliders,
            breakables,
            terrain: authored
                .terrain
                .as_ref()
                .map_or(AssetId(0), |t| AssetId::from_key(&format!("terrain.{t}"))),
            pages: if pages.entries.is_empty() {
                AssetId(0)
            } else {
                AssetId::from_key(&format!("scene.{name}.pages"))
            },
            spawn: [
                authored.spawn.eye.0,
                authored.spawn.eye.1,
                authored.spawn.eye.2,
                authored.spawn.yaw.to_radians(),
                authored.spawn.pitch.to_radians(),
            ],
        };
        let pages = (!pages.entries.is_empty()).then_some(pages);
        let readables = readables_of(&authored, &name, strings, &file)?;
        let inscribed = inscribed_of(&authored, &name, &file)?;
        out.push(BakedScene {
            key: format!("scene.{name}"),
            scene,
            meshes,
            pages,
            readables,
            inscribed,
        });
    }
    Ok(out)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn cross(a: [f32; 3], b: [f32; 3]) -> [f32; 3] {
        [
            a[1] * b[2] - a[2] * b[1],
            a[2] * b[0] - a[0] * b[2],
            a[0] * b[1] - a[1] * b[0],
        ]
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
        let unit = |min: V3, max: V3| AuthoredBox {
            material: String::new(),
            min,
            max,
            yaw: 0.0,
            tag: None,
        };
        let mut mesh = Mesh::default();
        add_box(
            &mut mesh,
            &unit((0.0, 0.0, 0.0), (2.0, 1.0, 1.0)),
            0,
            &|_, _| 0,
        );
        add_box(
            &mut mesh,
            &unit((2.0, 0.0, 0.0), (5.0, 1.0, 1.0)),
            0,
            &|_, _| 0,
        );
        // Faces are divided into half-metre cells: 78 points for the first box, 102 for
        // the second.
        assert_eq!(mesh.vertices.len(), 180);
        for tri in mesh.indices.chunks_exact(3) {
            let [a, b, c] = [
                mesh.vertices[tri[0] as usize],
                mesh.vertices[tri[1] as usize],
                mesh.vertices[tri[2] as usize],
            ];
            let e1: [f32; 3] = std::array::from_fn(|i| b.position[i] - a.position[i]);
            let e2: [f32; 3] = std::array::from_fn(|i| c.position[i] - a.position[i]);
            assert!(
                dot(cross(e1, e2), a.normal) > 0.0,
                "triangles wind counter-clockwise seen from outside"
            );
        }
        // On the south face (+Z) the page's first coordinate is world x, on both boxes.
        for v in mesh.vertices.iter().filter(|v| v.normal == [0.0, 0.0, 1.0]) {
            assert_eq!(v.anchor, [v.position[0], v.position[1]]);
        }
    }

    #[test]
    fn the_foot_of_a_wall_sees_less_sky_than_its_top() {
        let unit = |min: V3, max: V3| AuthoredBox {
            material: String::new(),
            min,
            max,
            yaw: 0.0,
            tag: None,
        };
        let ground = unit((-10.0, -0.5, -10.0), (10.0, 0.0, 10.0));
        let wall = unit((-3.0, 0.0, -0.2), (3.0, 3.0, 0.2));
        let mut mesh = Mesh::default();
        add_box(&mut mesh, &wall, 0, &|_, _| 0);
        let occluders = [
            ([-10.0, -0.5, -10.0], [10.0, 0.0, 10.0]),
            ([-3.0, 0.0, -0.2], [3.0, 3.0, 0.2]),
        ];
        let _ = ground;
        bake_sky(&mut mesh, &|p, _| p, &occluders);
        let south: Vec<&Vertex> = mesh
            .vertices
            .iter()
            .filter(|v| v.normal == [0.0, 0.0, 1.0] && v.position[0] == 0.0)
            .collect();
        let foot = south.iter().find(|v| v.position[1] == 0.0).unwrap().sky;
        let top = south.iter().find(|v| v.position[1] == 3.0).unwrap().sky;
        assert!(
            foot < top,
            "the foot of the wall ({foot}) sees less sky than its top ({top})"
        );
        assert!(
            top > 0.45 && top < 0.75,
            "a wall's face sees about half the sky: {top}"
        );
        // The top of the wall sees all of it.
        let crown = mesh
            .vertices
            .iter()
            .filter(|v| v.normal == [0.0, 1.0, 0.0])
            .map(|v| v.sky)
            .fold(0.0, f32::max);
        assert!(crown > 0.95);
    }
}
