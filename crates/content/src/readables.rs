//! The texts of a scene that can be read with the Read action: which string each is,
//! the box of the surface it is set on, and its hidden lines, each with the Perception
//! it takes to see it. One table beside each scene, `readables.<scene>`.

use crate::{AssetId, StringId};

const MAGIC: &[u8; 8] = b"VLMKRDB1";

/// The asset key of a scene's readables.
pub fn readables_key(scene: &str) -> String {
    format!("readables.{scene}")
}

#[derive(Clone, Debug, PartialEq)]
pub struct Readable {
    /// Permanent: the text's own key in the scene.
    pub id: AssetId,
    pub text: StringId,
    /// The box round the surface the text is set on, in the world.
    pub min: [f32; 3],
    pub max: [f32; 3],
    /// Lines only a sharp enough reader sees, each with the Perception it takes.
    pub hidden: Vec<(StringId, u32)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Readables {
    pub items: Vec<Readable>,
}

impl Readables {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.extend_from_slice(&(self.items.len() as u32).to_le_bytes());
        for r in &self.items {
            out.extend_from_slice(&r.id.0.to_le_bytes());
            out.extend_from_slice(&r.text.0.to_le_bytes());
            for v in r.min.iter().chain(&r.max) {
                out.extend_from_slice(&v.to_le_bytes());
            }
            out.extend_from_slice(&(r.hidden.len() as u32).to_le_bytes());
            for (s, p) in &r.hidden {
                out.extend_from_slice(&s.0.to_le_bytes());
                out.extend_from_slice(&p.to_le_bytes());
            }
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 12 || &bytes[..8] != MAGIC {
            return Err("not a table of readables".into());
        }
        let take = |at: &mut usize, n: usize| -> Result<&[u8], String> {
            let b = bytes
                .get(*at..*at + n)
                .ok_or_else(|| "the table of readables is cut short".to_string())?;
            *at += n;
            Ok(b)
        };
        let u64_at = |at: &mut usize| -> Result<u64, String> {
            Ok(u64::from_le_bytes(take(at, 8)?.try_into().unwrap()))
        };
        let u32_at = |at: &mut usize| -> Result<u32, String> {
            Ok(u32::from_le_bytes(take(at, 4)?.try_into().unwrap()))
        };
        let mut at = 8;
        let count = u32_at(&mut at)? as usize;
        let mut items = Vec::with_capacity(count.min(4096));
        for _ in 0..count {
            let id = AssetId(u64_at(&mut at)?);
            let text = StringId(u64_at(&mut at)?);
            let mut v = [0.0f32; 6];
            for x in v.iter_mut() {
                *x = f32::from_bits(u32_at(&mut at)?);
            }
            let n = u32_at(&mut at)? as usize;
            let mut hidden = Vec::with_capacity(n.min(64));
            for _ in 0..n {
                hidden.push((StringId(u64_at(&mut at)?), u32_at(&mut at)?));
            }
            items.push(Readable {
                id,
                text,
                min: [v[0], v[1], v[2]],
                max: [v[3], v[4], v[5]],
                hidden,
            });
        }
        Ok(Self { items })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn readables_round_trip() {
        let r = Readables {
            items: vec![Readable {
                id: AssetId(3),
                text: StringId(9),
                min: [0.0, 1.0, 2.0],
                max: [3.0, 4.0, 5.0],
                hidden: vec![(StringId(11), 2)],
            }],
        };
        assert_eq!(Readables::from_bytes(&r.to_bytes()).unwrap(), r);
    }
}

/// The things of a scene that carry Inscriptions. One table beside each scene,
/// `targets.<scene>`.
pub fn targets_key(scene: &str) -> String {
    format!("targets.{scene}")
}

#[derive(Clone, Debug, PartialEq)]
pub struct InscribedThing {
    /// The object's or light's permanent ID.
    pub id: AssetId,
    /// Its target class's key: `door`, `lamp`.
    pub class: String,
    pub centre: [f32; 3],
    pub radius: f32,
    /// 0 Plain, 1 Tough, 2 Hard, 3 Sealed.
    pub resistance: u8,
    /// Its words, each with its class: `("LOCKED", "Access")`.
    pub words: Vec<(String, String)>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct InscribedThings {
    pub items: Vec<InscribedThing>,
}

impl InscribedThings {
    const MAGIC: &'static [u8; 8] = b"VLMKTGT1";

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Self::MAGIC.to_vec();
        let s = |out: &mut Vec<u8>, t: &str| {
            out.extend_from_slice(&(t.len() as u32).to_le_bytes());
            out.extend_from_slice(t.as_bytes());
        };
        out.extend_from_slice(&(self.items.len() as u32).to_le_bytes());
        for t in &self.items {
            out.extend_from_slice(&t.id.0.to_le_bytes());
            s(&mut out, &t.class);
            for v in t.centre.iter().chain([&t.radius]) {
                out.extend_from_slice(&v.to_le_bytes());
            }
            out.push(t.resistance);
            out.extend_from_slice(&(t.words.len() as u32).to_le_bytes());
            for (w, c) in &t.words {
                s(&mut out, w);
                s(&mut out, c);
            }
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 12 || &bytes[..8] != Self::MAGIC {
            return Err("not a table of inscribed things".into());
        }
        let mut at = 8usize;
        let take = |at: &mut usize, n: usize| -> Result<&[u8], String> {
            let b = bytes
                .get(*at..*at + n)
                .ok_or_else(|| "the table of inscribed things is cut short".to_string())?;
            *at += n;
            Ok(b)
        };
        let u32_at = |at: &mut usize| -> Result<u32, String> {
            Ok(u32::from_le_bytes(take(at, 4)?.try_into().unwrap()))
        };
        let str_at = |at: &mut usize| -> Result<String, String> {
            let n = u32_at(at)? as usize;
            String::from_utf8(take(at, n)?.to_vec()).map_err(|_| "a name is not text".to_string())
        };
        let n = u32_at(&mut at)? as usize;
        let mut items = Vec::with_capacity(n.min(65_536));
        for _ in 0..n {
            let id = AssetId(u64::from_le_bytes(take(&mut at, 8)?.try_into().unwrap()));
            let class = str_at(&mut at)?;
            let mut v = [0.0f32; 4];
            for x in v.iter_mut() {
                *x = f32::from_bits(u32_at(&mut at)?);
            }
            let resistance = take(&mut at, 1)?[0];
            let k = u32_at(&mut at)? as usize;
            let mut words = Vec::with_capacity(k.min(3));
            for _ in 0..k {
                words.push((str_at(&mut at)?, str_at(&mut at)?));
            }
            items.push(InscribedThing {
                id,
                class,
                centre: [v[0], v[1], v[2]],
                radius: v[3],
                resistance,
                words,
            });
        }
        Ok(Self { items })
    }
}

#[cfg(test)]
mod inscribed_tests {
    use super::*;

    #[test]
    fn inscribed_things_round_trip() {
        let t = InscribedThings {
            items: vec![InscribedThing {
                id: AssetId(5),
                class: "door".into(),
                centre: [1.0, 2.0, 3.0],
                radius: 1.2,
                resistance: 1,
                words: vec![("LOCKED".into(), "Access".into())],
            }],
        };
        assert_eq!(InscribedThings::from_bytes(&t.to_bytes()).unwrap(), t);
    }
}
