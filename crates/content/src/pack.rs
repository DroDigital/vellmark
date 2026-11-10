//! The pack file.
//!
//! ```text
//! header   32 bytes   magic "VLMKPACK", version u32, flags u32, toc_offset u64, toc_count u64
//! assets              each starts on a 16-byte boundary, in order of (kind, id)
//! toc                 toc_count entries of 40 bytes, in the same order
//! ```
//!
//! All integers are little-endian. A pack holds no filenames and no timestamps, assets
//! are never reordered, and nothing is compressed as a whole, so that a patch touches
//! only the bytes around the asset that changed.

use crate::id::{fnv1a64, AssetId, AssetKind};
use std::collections::BTreeMap;
use std::fmt;

pub const PACK_VERSION: u32 = 1;
const MAGIC: &[u8; 8] = b"VLMKPACK";
const HEADER_LEN: usize = 32;
const ENTRY_LEN: usize = 40;
const ALIGN: usize = 16;

#[derive(Debug, PartialEq, Eq)]
pub enum PackError {
    TooShort,
    BadMagic,
    /// The pack was written by a newer or older game. Holds the version found.
    Version(u32),
    BadTable,
    UnknownKind(u32),
    /// An asset's bytes do not match the hash recorded for it.
    Corrupt(AssetId),
    Duplicate(AssetKind, AssetId),
}

impl fmt::Display for PackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self:?}")
    }
}

impl std::error::Error for PackError {}

#[derive(Clone, Copy, Debug)]
struct Entry {
    offset: u64,
    len: u64,
    hash: u64,
}

/// Collects assets and writes them in a fixed order.
#[derive(Default)]
pub struct PackWriter {
    assets: BTreeMap<(AssetKind, AssetId), Vec<u8>>,
}

impl PackWriter {
    pub fn new() -> Self {
        Self::default()
    }

    /// Two different keys that hash to one ID are reported here, at bake time.
    pub fn add(&mut self, kind: AssetKind, id: AssetId, bytes: Vec<u8>) -> Result<(), PackError> {
        if self.assets.insert((kind, id), bytes).is_some() {
            return Err(PackError::Duplicate(kind, id));
        }
        Ok(())
    }

    pub fn len(&self) -> usize {
        self.assets.len()
    }

    pub fn is_empty(&self) -> bool {
        self.assets.is_empty()
    }

    pub fn finish(self) -> Vec<u8> {
        let mut out = vec![0u8; HEADER_LEN];
        let mut toc = Vec::with_capacity(self.assets.len() * ENTRY_LEN);
        let count = self.assets.len() as u64;
        for ((kind, id), bytes) in self.assets {
            while out.len() % ALIGN != 0 {
                out.push(0);
            }
            toc.extend_from_slice(&id.0.to_le_bytes());
            toc.extend_from_slice(&(kind as u32).to_le_bytes());
            toc.extend_from_slice(&0u32.to_le_bytes()); // per-asset flags, none yet
            toc.extend_from_slice(&(out.len() as u64).to_le_bytes());
            toc.extend_from_slice(&(bytes.len() as u64).to_le_bytes());
            toc.extend_from_slice(&fnv1a64(&bytes).to_le_bytes());
            out.extend_from_slice(&bytes);
        }
        while out.len() % ALIGN != 0 {
            out.push(0);
        }
        let toc_offset = out.len() as u64;
        out.extend_from_slice(&toc);
        out[0..8].copy_from_slice(MAGIC);
        out[8..12].copy_from_slice(&PACK_VERSION.to_le_bytes());
        out[12..16].copy_from_slice(&0u32.to_le_bytes());
        out[16..24].copy_from_slice(&toc_offset.to_le_bytes());
        out[24..32].copy_from_slice(&count.to_le_bytes());
        out
    }
}

/// A pack held in memory, with its table of contents read.
pub struct Pack {
    bytes: Vec<u8>,
    entries: BTreeMap<(AssetKind, AssetId), Entry>,
}

fn u32_at(b: &[u8], at: usize) -> u32 {
    u32::from_le_bytes(b[at..at + 4].try_into().unwrap())
}

fn u64_at(b: &[u8], at: usize) -> u64 {
    u64::from_le_bytes(b[at..at + 8].try_into().unwrap())
}

impl Pack {
    pub fn from_bytes(bytes: Vec<u8>) -> Result<Self, PackError> {
        if bytes.len() < HEADER_LEN {
            return Err(PackError::TooShort);
        }
        if &bytes[0..8] != MAGIC {
            return Err(PackError::BadMagic);
        }
        let version = u32_at(&bytes, 8);
        if version != PACK_VERSION {
            return Err(PackError::Version(version));
        }
        let toc_offset = u64_at(&bytes, 16) as usize;
        let count = u64_at(&bytes, 24) as usize;
        let toc_end = count
            .checked_mul(ENTRY_LEN)
            .and_then(|n| n.checked_add(toc_offset))
            .ok_or(PackError::BadTable)?;
        if toc_offset < HEADER_LEN || toc_end != bytes.len() {
            return Err(PackError::BadTable);
        }
        let mut entries = BTreeMap::new();
        for i in 0..count {
            let at = toc_offset + i * ENTRY_LEN;
            let id = AssetId(u64_at(&bytes, at));
            let kind_raw = u32_at(&bytes, at + 8);
            let kind = AssetKind::from_u32(kind_raw).ok_or(PackError::UnknownKind(kind_raw))?;
            let offset = u64_at(&bytes, at + 16);
            let len = u64_at(&bytes, at + 24);
            let hash = u64_at(&bytes, at + 32);
            let end = offset.checked_add(len).ok_or(PackError::BadTable)?;
            if (offset as usize) < HEADER_LEN || end as usize > toc_offset {
                return Err(PackError::BadTable);
            }
            if entries.insert((kind, id), Entry { offset, len, hash }).is_some() {
                return Err(PackError::Duplicate(kind, id));
            }
        }
        Ok(Self { bytes, entries })
    }

    pub fn get(&self, kind: AssetKind, id: AssetId) -> Option<&[u8]> {
        let e = self.entries.get(&(kind, id))?;
        Some(&self.bytes[e.offset as usize..(e.offset + e.len) as usize])
    }

    /// Every asset of one kind, in ID order.
    pub fn of_kind(&self, kind: AssetKind) -> impl Iterator<Item = (AssetId, &[u8])> {
        self.entries
            .range((kind, AssetId(0))..=(kind, AssetId(u64::MAX)))
            .map(|((_, id), e)| (*id, &self.bytes[e.offset as usize..(e.offset + e.len) as usize]))
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Checks every asset against its recorded hash. Run by tools and at the first
    /// launch after an update, not on every load.
    pub fn verify(&self) -> Result<(), PackError> {
        for ((_, id), e) in &self.entries {
            let bytes = &self.bytes[e.offset as usize..(e.offset + e.len) as usize];
            if fnv1a64(bytes) != e.hash {
                return Err(PackError::Corrupt(*id));
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample(order_reversed: bool) -> Vec<u8> {
        let mut items = vec![
            (AssetKind::Mesh, AssetId::from_key("mesh.wall"), vec![1u8, 2, 3]),
            (AssetKind::StringTable, AssetId::from_key("en"), b"hello".to_vec()),
            (AssetKind::Mesh, AssetId::from_key("mesh.door"), vec![9u8; 40]),
        ];
        if order_reversed {
            items.reverse();
        }
        let mut w = PackWriter::new();
        for (k, id, b) in items {
            w.add(k, id, b).unwrap();
        }
        w.finish()
    }

    #[test]
    fn round_trip() {
        let p = Pack::from_bytes(sample(false)).unwrap();
        assert_eq!(p.len(), 3);
        assert_eq!(p.get(AssetKind::Mesh, AssetId::from_key("mesh.wall")), Some(&[1u8, 2, 3][..]));
        assert_eq!(p.get(AssetKind::StringTable, AssetId::from_key("en")), Some(&b"hello"[..]));
        assert_eq!(p.get(AssetKind::Texture, AssetId::from_key("en")), None);
        assert_eq!(p.of_kind(AssetKind::Mesh).count(), 2);
        p.verify().unwrap();
    }

    #[test]
    fn bytes_do_not_depend_on_the_order_assets_were_added() {
        assert_eq!(sample(false), sample(true));
    }

    #[test]
    fn duplicate_ids_are_refused() {
        let mut w = PackWriter::new();
        w.add(AssetKind::Mesh, AssetId(5), vec![]).unwrap();
        assert_eq!(w.add(AssetKind::Mesh, AssetId(5), vec![]), Err(PackError::Duplicate(AssetKind::Mesh, AssetId(5))));
    }

    #[test]
    fn damage_is_detected() {
        let mut bytes = sample(false);
        assert_eq!(Pack::from_bytes(bytes[..10].to_vec()).err(), Some(PackError::TooShort));
        let mut wrong = bytes.clone();
        wrong[8] = 99;
        assert_eq!(Pack::from_bytes(wrong).err(), Some(PackError::Version(99)));
        bytes[HEADER_LEN + 1] ^= 0xff;
        let p = Pack::from_bytes(bytes).unwrap();
        assert!(matches!(p.verify(), Err(PackError::Corrupt(_))));
    }

    #[test]
    fn changing_one_asset_leaves_earlier_bytes_alone() {
        let build = |door: u8| {
            let mut w = PackWriter::new();
            w.add(AssetKind::StringTable, AssetId(1), vec![7; 100]).unwrap();
            w.add(AssetKind::Mesh, AssetId(2), vec![door; 100]).unwrap();
            w.finish()
        };
        let (a, b) = (build(1), build(2));
        assert_eq!(a.len(), b.len());
        let first_difference = a.iter().zip(&b).position(|(x, y)| x != y).unwrap();
        assert!(first_difference >= HEADER_LEN + 100);
    }
}
