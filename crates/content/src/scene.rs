//! Meshes, material sets, and scenes, as the renderer loads them.

use crate::id::AssetId;

struct Reader<'a>(&'a [u8], usize);

impl<'a> Reader<'a> {
    fn take(&mut self, n: usize) -> Result<&'a [u8], &'static str> {
        let end = self
            .1
            .checked_add(n)
            .filter(|e| *e <= self.0.len())
            .ok_or("asset is cut short")?;
        let s = &self.0[self.1..end];
        self.1 = end;
        Ok(s)
    }
    fn u32(&mut self) -> Result<u32, &'static str> {
        Ok(u32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn u64(&mut self) -> Result<u64, &'static str> {
        Ok(u64::from_le_bytes(self.take(8)?.try_into().unwrap()))
    }
    fn f32(&mut self) -> Result<f32, &'static str> {
        Ok(f32::from_le_bytes(self.take(4)?.try_into().unwrap()))
    }
    fn f32s<const N: usize>(&mut self) -> Result<[f32; N], &'static str> {
        let mut out = [0.0; N];
        for v in &mut out {
            *v = self.f32()?;
        }
        Ok(out)
    }
}

fn put_f32s(out: &mut Vec<u8>, values: &[f32]) {
    for v in values {
        out.extend_from_slice(&v.to_le_bytes());
    }
}

/// One corner of a triangle. 60 bytes, laid out as the GPU reads it.
#[derive(Clone, Copy, Debug, PartialEq, bytemuck::Pod, bytemuck::Zeroable)]
#[repr(C)]
pub struct Vertex {
    pub position: [f32; 3],
    pub normal: [f32; 3],
    /// xyz: the direction in which the anchor's first coordinate grows. w: the sign of
    /// the bitangent.
    pub tangent: [f32; 4],
    /// Where this point is on its surface's page, in metres. Letters are fixed to this
    /// coordinate, never to the screen. The first coordinate runs in reading direction
    /// and the second runs up.
    pub anchor: [f32; 2],
    /// Index into the material set.
    pub material: u32,
    /// The scene's text page that covers this surface, counted from 1, or 0 for none.
    pub page: u32,
    /// How thick the object is behind this surface, in metres. Up close the eye can
    /// see this far into it.
    pub thickness: f32,
    /// The share of the sky this point can see, 0 to 1, baked: it scales the sky's light.
    pub sky: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Mesh {
    pub vertices: Vec<Vertex>,
    pub indices: Vec<u32>,
}

impl Mesh {
    const MAGIC: &'static [u8; 8] = b"VLMKMSH2";

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(16 + self.vertices.len() * 64 + self.indices.len() * 4);
        out.extend_from_slice(Self::MAGIC);
        out.extend_from_slice(&(self.vertices.len() as u32).to_le_bytes());
        out.extend_from_slice(&(self.indices.len() as u32).to_le_bytes());
        out.extend_from_slice(bytemuck::cast_slice(&self.vertices));
        out.extend_from_slice(bytemuck::cast_slice(&self.indices));
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut r = Reader(bytes, 0);
        if r.take(8)? != Self::MAGIC {
            return Err("not a mesh");
        }
        let (vertex_count, index_count) = (r.u32()? as usize, r.u32()? as usize);
        let vertices = bytemuck::pod_collect_to_vec(r.take(vertex_count * 64)?);
        let indices: Vec<u32> = bytemuck::pod_collect_to_vec(r.take(index_count * 4)?);
        if indices.iter().any(|&i| i as usize >= vertex_count) || index_count % 3 != 0 {
            return Err("mesh indices are out of range");
        }
        Ok(Self { vertices, indices })
    }
}

/// How a material's letters sit in its surface, and what they are.
#[derive(Clone, Debug, PartialEq)]
pub struct Material {
    pub key: AssetId,
    /// Side of the square of surface one copy of the maps covers, in metres.
    pub tile_metres: f32,
    /// How far ink and paper part from the surface's own colour, 0 to 1.
    pub contrast: f32,
    /// Depth of the letters' relief. Negative cuts them in, positive raises them.
    pub relief: f32,
    /// Light the surface gives off, as a multiple of its colour.
    pub emissive: f32,
    /// The characters this material is written in, when no authored text covers it.
    pub family: Vec<u8>,
    /// What the material's lattice may spell, now and then: its own name, what it is
    /// made of, and the words for it hot and cold, from least to most.
    pub lexicon: Lexicon,
}

/// The words a material's letters may form. Every word is 1 to 16 printable ASCII
/// characters.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct Lexicon {
    pub name: Vec<u8>,
    pub elements: Vec<Vec<u8>>,
    pub hot: Vec<Vec<u8>>,
    pub cold: Vec<Vec<u8>>,
}

impl Lexicon {
    pub const WORD: usize = 16;
    pub const ELEMENTS: usize = 3;
    pub const HOT: usize = 3;
    pub const COLD: usize = 2;
    /// Slots in order: the name, the elements, the hot words, the cold words.
    pub const SLOTS: usize = 1 + Self::ELEMENTS + Self::HOT + Self::COLD;

    /// Every slot, empty where there is no word.
    pub fn slots(&self) -> [[u8; Self::WORD]; Self::SLOTS] {
        let mut out = [[0u8; Self::WORD]; Self::SLOTS];
        let words = std::iter::once(Some(&self.name))
            .chain((0..Self::ELEMENTS).map(|i| self.elements.get(i)))
            .chain((0..Self::HOT).map(|i| self.hot.get(i)))
            .chain((0..Self::COLD).map(|i| self.cold.get(i)));
        for (slot, word) in out.iter_mut().zip(words) {
            if let Some(word) = word {
                let n = word.len().min(Self::WORD);
                slot[..n].copy_from_slice(&word[..n]);
            }
        }
        out
    }

    fn from_slots(slots: &[[u8; Self::WORD]]) -> Self {
        let word = |s: &[u8; Self::WORD]| {
            s.iter()
                .take_while(|&&b| b != 0)
                .copied()
                .collect::<Vec<u8>>()
        };
        let list = |range: std::ops::Range<usize>| {
            range
                .map(|i| word(&slots[i]))
                .filter(|w| !w.is_empty())
                .collect()
        };
        Self {
            name: word(&slots[0]),
            elements: list(1..1 + Self::ELEMENTS),
            hot: list(1 + Self::ELEMENTS..1 + Self::ELEMENTS + Self::HOT),
            cold: list(1 + Self::ELEMENTS + Self::HOT..Self::SLOTS),
        }
    }
}

/// Every material's maps as two array textures: colour, and normal with roughness.
#[derive(Clone, Debug, PartialEq)]
pub struct MaterialSet {
    pub size: u32,
    pub mips: u32,
    pub materials: Vec<Material>,
    /// RGBA8. For each level: every material's colour map (sRGB, alpha is metalness),
    /// then every material's normal map (xy), roughness (z), and height (w).
    pub texels: Vec<u8>,
}

impl MaterialSet {
    const MAGIC: &'static [u8; 8] = b"VLMKMAT2";
    pub const MAX_FAMILY: usize = 8;

    pub fn index_of(&self, key: AssetId) -> Option<u32> {
        self.materials
            .iter()
            .position(|m| m.key == key)
            .map(|i| i as u32)
    }

    /// Bytes of one map of one level: every material back to back. `map` 0 is colour.
    pub fn level_range(&self, level: u32, map: u32) -> std::ops::Range<usize> {
        let count = self.materials.len();
        let mut start = 0;
        for l in 0..level {
            let side = (self.size >> l) as usize;
            start += side * side * 4 * count * 2;
        }
        let side = (self.size >> level) as usize;
        let one = side * side * 4 * count;
        start + one * map as usize..start + one * (map as usize + 1)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(self.texels.len() + 1024);
        out.extend_from_slice(Self::MAGIC);
        for v in [self.materials.len() as u32, self.size, self.mips] {
            out.extend_from_slice(&v.to_le_bytes());
        }
        for m in &self.materials {
            out.extend_from_slice(&m.key.0.to_le_bytes());
            put_f32s(&mut out, &[m.tile_metres, m.contrast, m.relief, m.emissive]);
            out.extend_from_slice(&(m.family.len() as u32).to_le_bytes());
            let mut family = [0u8; Self::MAX_FAMILY];
            family[..m.family.len()].copy_from_slice(&m.family);
            out.extend_from_slice(&family);
            for slot in m.lexicon.slots() {
                out.extend_from_slice(&slot);
            }
        }
        out.extend_from_slice(&self.texels);
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut r = Reader(bytes, 0);
        if r.take(8)? != Self::MAGIC {
            return Err("not a material set");
        }
        let (count, size, mips) = (r.u32()? as usize, r.u32()?, r.u32()?);
        if size == 0 || !size.is_power_of_two() || mips == 0 || (size >> (mips - 1)) == 0 {
            return Err("material set header is not valid");
        }
        let mut materials = Vec::with_capacity(count);
        for _ in 0..count {
            let key = AssetId(r.u64()?);
            let [tile_metres, contrast, relief, emissive] = r.f32s()?;
            let len = r.u32()? as usize;
            let family = r.take(Self::MAX_FAMILY)?;
            if len == 0 || len > Self::MAX_FAMILY {
                return Err("material has no glyph family");
            }
            let mut slots = [[0u8; Lexicon::WORD]; Lexicon::SLOTS];
            for slot in &mut slots {
                slot.copy_from_slice(r.take(Lexicon::WORD)?);
            }
            materials.push(Material {
                key,
                tile_metres,
                contrast,
                relief,
                emissive,
                family: family[..len].to_vec(),
                lexicon: Lexicon::from_slots(&slots),
            });
        }
        let set = Self {
            size,
            mips,
            materials,
            texels: bytes[r.1..].to_vec(),
        };
        if set.level_range(mips - 1, 1).end != set.texels.len() {
            return Err("material set texels have the wrong length");
        }
        Ok(set)
    }
}

/// Where one page of authored text lies on its surface.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PageEntry {
    /// The cascade level whose cells the page fills: 0 for the finest print.
    pub level: u32,
    /// The page's left column and top row, counted in that level's cells of the
    /// surface's anchor. Rows are counted upwards, as the anchor is.
    pub left: i32,
    pub top: i32,
    pub width: u32,
    pub height: u32,
    /// Where the page's characters start in the sheet.
    pub sheet_x: u32,
    pub sheet_y: u32,
}

/// A scene's authored text: every page's characters on one sheet, in reading order.
/// A character of 0 means no authored text here: the surface shows its own grain.
#[derive(Clone, Debug, PartialEq, Default)]
pub struct TextPages {
    pub entries: Vec<PageEntry>,
    /// Side of the square sheet, in characters.
    pub sheet_size: u32,
    pub sheet: Vec<u8>,
}

impl TextPages {
    const MAGIC: &'static [u8; 8] = b"VLMKTXT1";
    pub const MAX_PAGES: usize = 64;
    /// Of the table's pages, how many at its end are kept for generated truth, written
    /// by the renderer for the things nearest the eye, and how many rows at the bottom
    /// of the sheet they take. Authored text uses the rest.
    pub const GENERATED_PAGES: usize = 8;
    pub const GENERATED_ROWS: u32 = 128;

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(Self::MAGIC);
        out.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());
        out.extend_from_slice(&self.sheet_size.to_le_bytes());
        for e in &self.entries {
            for v in [
                e.level as i32,
                e.left,
                e.top,
                e.width as i32,
                e.height as i32,
                e.sheet_x as i32,
                e.sheet_y as i32,
            ] {
                out.extend_from_slice(&v.to_le_bytes());
            }
        }
        out.extend_from_slice(&self.sheet);
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut r = Reader(bytes, 0);
        if r.take(8)? != Self::MAGIC {
            return Err("not a set of text pages");
        }
        let (count, sheet_size) = (r.u32()? as usize, r.u32()?);
        if count > Self::MAX_PAGES {
            return Err("too many text pages");
        }
        let mut entries = Vec::with_capacity(count);
        for _ in 0..count {
            let mut v = [0i32; 7];
            for slot in &mut v {
                *slot = r.u32()? as i32;
            }
            entries.push(PageEntry {
                level: v[0] as u32,
                left: v[1],
                top: v[2],
                width: v[3] as u32,
                height: v[4] as u32,
                sheet_x: v[5] as u32,
                sheet_y: v[6] as u32,
            });
        }
        let sheet = r.take((sheet_size * sheet_size) as usize)?.to_vec();
        if entries
            .iter()
            .any(|e| e.sheet_x + e.width > sheet_size || e.sheet_y + e.height > sheet_size)
        {
            return Err("a text page lies outside its sheet");
        }
        Ok(Self {
            entries,
            sheet_size,
            sheet,
        })
    }
}

/// A thing in a scene that can move or change: a door, a sign, a lantern's glass.
#[derive(Clone, Debug, PartialEq)]
pub struct Object {
    pub key: AssetId,
    pub mesh: AssetId,
    /// Rows of a 3 by 4 matrix: rotation and scale, then translation in the last column.
    pub transform: [f32; 12],
}

#[derive(Clone, Debug, PartialEq)]
pub struct Light {
    pub key: AssetId,
    pub position: [f32; 3],
    /// Linear colour times strength.
    pub colour: [f32; 3],
    /// The distance at which the light has faded to nothing.
    pub radius: f32,
    pub casts_shadow: bool,
}

/// A box of one material, for collision and for breaking into letters. Axis-aligned
/// in its owner's space: `min` then `max`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SolidBox {
    pub min: [f32; 3],
    pub max: [f32; 3],
    pub material: u32,
}

/// A thing that can be broken into its letters: an object, by its key, and the boxes
/// it is made of, in the object's own space.
#[derive(Clone, Debug, PartialEq)]
pub struct Breakable {
    pub key: AssetId,
    pub transform: [f32; 12],
    pub boxes: Vec<SolidBox>,
    /// In kilograms if it is a loose prop; 0 if it is fixed in place.
    pub mass: f32,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Scene {
    /// Everything that never moves, as one mesh.
    pub static_mesh: AssetId,
    pub objects: Vec<Object>,
    pub lights: Vec<Light>,
    /// The scene's authored text, or `AssetId(0)` if it has none.
    pub pages: AssetId,
    /// Everything that never moves, as boxes in the world, for things to land on.
    pub colliders: Vec<SolidBox>,
    pub breakables: Vec<Breakable>,
    /// Where a visitor starts: eye position, yaw, pitch.
    pub spawn: [f32; 5],
    /// The terrain the scene stands on, by its key, or `AssetId(0)` if it has none. Its
    /// tiles are found by where they are, as `TileCoord::asset_id` gives them.
    pub terrain: AssetId,
}

impl Scene {
    const MAGIC: &'static [u8; 8] = b"VLMKSCN4";

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(Self::MAGIC);
        out.extend_from_slice(&self.static_mesh.0.to_le_bytes());
        out.extend_from_slice(&self.pages.0.to_le_bytes());
        out.extend_from_slice(&self.terrain.0.to_le_bytes());
        out.extend_from_slice(&(self.objects.len() as u32).to_le_bytes());
        out.extend_from_slice(&(self.lights.len() as u32).to_le_bytes());
        put_f32s(&mut out, &self.spawn);
        for o in &self.objects {
            out.extend_from_slice(&o.key.0.to_le_bytes());
            out.extend_from_slice(&o.mesh.0.to_le_bytes());
            put_f32s(&mut out, &o.transform);
        }
        for l in &self.lights {
            out.extend_from_slice(&l.key.0.to_le_bytes());
            put_f32s(&mut out, &l.position);
            put_f32s(&mut out, &l.colour);
            put_f32s(&mut out, &[l.radius]);
            out.extend_from_slice(&(l.casts_shadow as u32).to_le_bytes());
        }
        let put_box = |out: &mut Vec<u8>, b: &SolidBox| {
            put_f32s(out, &b.min);
            put_f32s(out, &b.max);
            out.extend_from_slice(&b.material.to_le_bytes());
        };
        out.extend_from_slice(&(self.colliders.len() as u32).to_le_bytes());
        for b in &self.colliders {
            put_box(&mut out, b);
        }
        out.extend_from_slice(&(self.breakables.len() as u32).to_le_bytes());
        for breakable in &self.breakables {
            out.extend_from_slice(&breakable.key.0.to_le_bytes());
            put_f32s(&mut out, &breakable.transform);
            put_f32s(&mut out, &[breakable.mass]);
            out.extend_from_slice(&(breakable.boxes.len() as u32).to_le_bytes());
            for b in &breakable.boxes {
                put_box(&mut out, b);
            }
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        let mut r = Reader(bytes, 0);
        if r.take(8)? != Self::MAGIC {
            return Err("not a scene");
        }
        let static_mesh = AssetId(r.u64()?);
        let pages = AssetId(r.u64()?);
        let terrain = AssetId(r.u64()?);
        let (object_count, light_count) = (r.u32()? as usize, r.u32()? as usize);
        let spawn = r.f32s()?;
        let mut objects = Vec::with_capacity(object_count);
        for _ in 0..object_count {
            objects.push(Object {
                key: AssetId(r.u64()?),
                mesh: AssetId(r.u64()?),
                transform: r.f32s()?,
            });
        }
        let mut lights = Vec::with_capacity(light_count);
        for _ in 0..light_count {
            lights.push(Light {
                key: AssetId(r.u64()?),
                position: r.f32s()?,
                colour: r.f32s()?,
                radius: r.f32()?,
                casts_shadow: r.u32()? != 0,
            });
        }
        fn solid(r: &mut Reader<'_>) -> Result<SolidBox, &'static str> {
            Ok(SolidBox {
                min: r.f32s()?,
                max: r.f32s()?,
                material: r.u32()?,
            })
        }
        let mut colliders = Vec::new();
        for _ in 0..r.u32()? {
            colliders.push(solid(&mut r)?);
        }
        let mut breakables = Vec::new();
        for _ in 0..r.u32()? {
            let key = AssetId(r.u64()?);
            let transform = r.f32s()?;
            let mass = r.f32()?;
            let mut boxes = Vec::new();
            for _ in 0..r.u32()? {
                boxes.push(solid(&mut r)?);
            }
            breakables.push(Breakable {
                key,
                transform,
                boxes,
                mass,
            });
        }
        if r.1 != bytes.len() {
            return Err("scene has trailing bytes");
        }
        Ok(Self {
            static_mesh,
            objects,
            lights,
            pages,
            colliders,
            breakables,
            spawn,
            terrain,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mesh_round_trip() {
        let v = |x: f32| Vertex {
            position: [x, 0.0, 0.0],
            normal: [0.0, 1.0, 0.0],
            tangent: [1.0, 0.0, 0.0, 1.0],
            anchor: [x, 0.0],
            material: 2,
            page: 0,
            thickness: 0.3,
            sky: 1.0,
        };
        let mesh = Mesh {
            vertices: vec![v(0.0), v(1.0), v(2.0)],
            indices: vec![0, 1, 2],
        };
        assert_eq!(Mesh::from_bytes(&mesh.to_bytes()).unwrap(), mesh);
        let bad = Mesh {
            indices: vec![0, 1, 7],
            ..mesh
        };
        assert!(Mesh::from_bytes(&bad.to_bytes()).is_err());
    }

    #[test]
    fn material_set_round_trip() {
        let material = |name: &str| Material {
            key: AssetId::from_key(name),
            tile_metres: 2.0,
            contrast: 0.3,
            relief: -0.5,
            emissive: 0.0,
            family: b"HEB[]".to_vec(),
            lexicon: Lexicon {
                name: b"stone".to_vec(),
                elements: vec![b"SiO2".to_vec(), b"CaCO3".to_vec()],
                hot: vec![b"HOT".to_vec(), b"CRACKING".to_vec()],
                cold: vec![b"COLD".to_vec()],
            },
        };
        // Two materials, 2x2 with two levels: (4 + 1) texels * 4 bytes * 2 maps * 2 materials.
        let set = MaterialSet {
            size: 2,
            mips: 2,
            materials: vec![material("a"), material("b")],
            texels: (0..80).collect(),
        };
        let back = MaterialSet::from_bytes(&set.to_bytes()).unwrap();
        assert_eq!(back, set);
        assert_eq!(back.index_of(AssetId::from_key("b")), Some(1));
        assert_eq!(back.level_range(0, 1), 32..64);
        assert_eq!(back.level_range(1, 0), 64..72);
    }

    #[test]
    fn text_pages_round_trip() {
        let pages = TextPages {
            entries: vec![PageEntry {
                level: 1,
                left: -12,
                top: 80,
                width: 3,
                height: 2,
                sheet_x: 1,
                sheet_y: 0,
            }],
            sheet_size: 4,
            sheet: (0..16).collect(),
        };
        assert_eq!(TextPages::from_bytes(&pages.to_bytes()).unwrap(), pages);
        let mut outside = pages.clone();
        outside.entries[0].width = 9;
        assert!(TextPages::from_bytes(&outside.to_bytes()).is_err());
    }

    #[test]
    fn scene_round_trip() {
        let scene = Scene {
            static_mesh: AssetId(9),
            objects: vec![Object {
                key: AssetId(1),
                mesh: AssetId(2),
                transform: [1.0; 12],
            }],
            lights: vec![Light {
                key: AssetId(3),
                position: [1.0, 2.0, 3.0],
                colour: [4.0, 3.0, 2.0],
                radius: 9.0,
                casts_shadow: true,
            }],
            pages: AssetId(77),
            colliders: vec![SolidBox {
                min: [0.0; 3],
                max: [1.0; 3],
                material: 2,
            }],
            breakables: vec![Breakable {
                key: AssetId(5),
                transform: [0.5; 12],
                boxes: vec![SolidBox {
                    min: [0.0; 3],
                    max: [2.0; 3],
                    material: 1,
                }],
                mass: 4.0,
            }],
            spawn: [0.0, 1.7, 5.0, 0.0, 0.0],
            terrain: AssetId::from_key("terrain.valley"),
        };
        assert_eq!(Scene::from_bytes(&scene.to_bytes()).unwrap(), scene);
    }
}
