//! String tables. Every piece of text a player can see lives in one, under a permanent
//! ID, so that it can be translated and so that no text is written in code.
//!
//! ```text
//! magic "VLMKSTR1", count u32, blob_len u32
//! count entries of 16 bytes: id u64, offset u32, len u32, sorted by id
//! blob: UTF-8, entries back to back
//! ```

use crate::id::StringId;
use std::collections::BTreeMap;

const MAGIC: &[u8; 8] = b"VLMKSTR1";

#[derive(Default)]
pub struct StringTableBuilder {
    entries: BTreeMap<StringId, (String, String)>,
}

impl StringTableBuilder {
    pub fn new() -> Self {
        Self::default()
    }

    /// Returns an error naming both keys if two keys share an ID.
    pub fn add(&mut self, key: &str, text: &str) -> Result<(), String> {
        let id = StringId::from_key(key);
        if let Some((other, _)) = self.entries.get(&id) {
            return Err(if other == key {
                format!("string key `{key}` is defined twice")
            } else {
                format!("string keys `{key}` and `{other}` have the same ID; rename the new one")
            });
        }
        self.entries.insert(id, (key.to_owned(), text.to_owned()));
        Ok(())
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.entries.values().map(|(k, _)| k.as_str())
    }

    pub fn finish(self) -> Vec<u8> {
        let mut table = Vec::new();
        let mut blob = Vec::new();
        for (id, (_, text)) in &self.entries {
            table.extend_from_slice(&id.0.to_le_bytes());
            table.extend_from_slice(&(blob.len() as u32).to_le_bytes());
            table.extend_from_slice(&(text.len() as u32).to_le_bytes());
            blob.extend_from_slice(text.as_bytes());
        }
        let mut out = Vec::with_capacity(16 + table.len() + blob.len());
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&(self.entries.len() as u32).to_le_bytes());
        out.extend_from_slice(&(blob.len() as u32).to_le_bytes());
        out.extend_from_slice(&table);
        out.extend_from_slice(&blob);
        out
    }
}

/// A baked string table, ready to look up.
#[derive(Clone, Default)]
pub struct StringTable {
    index: Vec<(StringId, u32, u32)>,
    blob: String,
}

impl StringTable {
    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 16 || &bytes[0..8] != MAGIC {
            return Err("not a string table");
        }
        let count = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        let blob_len = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
        let blob_at = 16 + count * 16;
        if bytes.len() != blob_at + blob_len {
            return Err("string table has the wrong length");
        }
        let blob =
            std::str::from_utf8(&bytes[blob_at..]).map_err(|_| "string table is not UTF-8")?;
        let mut index = Vec::with_capacity(count);
        for i in 0..count {
            let at = 16 + i * 16;
            let id = StringId(u64::from_le_bytes(bytes[at..at + 8].try_into().unwrap()));
            let offset = u32::from_le_bytes(bytes[at + 8..at + 12].try_into().unwrap());
            let len = u32::from_le_bytes(bytes[at + 12..at + 16].try_into().unwrap());
            let end = offset as usize + len as usize;
            if end > blob.len()
                || !blob.is_char_boundary(offset as usize)
                || !blob.is_char_boundary(end)
            {
                return Err("string table entry is out of range");
            }
            if index.last().is_some_and(|&(prev, _, _)| prev >= id) {
                return Err("string table is not sorted");
            }
            index.push((id, offset, len));
        }
        Ok(Self {
            index,
            blob: blob.to_owned(),
        })
    }

    pub fn get(&self, id: StringId) -> Option<&str> {
        let i = self.index.binary_search_by_key(&id, |e| e.0).ok()?;
        let (_, offset, len) = self.index[i];
        Some(&self.blob[offset as usize..(offset + len) as usize])
    }

    /// The text, or a visible marker carrying the ID. A missing string is a bug that the
    /// harness catches, and the marker makes it impossible to miss in a capture.
    pub fn text(&self, id: StringId) -> std::borrow::Cow<'_, str> {
        match self.get(id) {
            Some(s) => s.into(),
            None => format!("<{:016x}>", id.0).into(),
        }
    }

    pub fn contains(&self, id: StringId) -> bool {
        self.get(id).is_some()
    }

    pub fn len(&self) -> usize {
        self.index.len()
    }

    pub fn is_empty(&self) -> bool {
        self.index.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::sid;

    #[test]
    fn round_trip_and_lookup() {
        let mut b = StringTableBuilder::new();
        b.add("ui.title", "Vellmark").unwrap();
        b.add("ui.quit", "Leave").unwrap();
        b.add("ui.accent", "caf\u{e9}").unwrap();
        let t = StringTable::from_bytes(&b.finish()).unwrap();
        assert_eq!(t.len(), 3);
        assert_eq!(t.get(sid!("ui.title")), Some("Vellmark"));
        assert_eq!(t.get(sid!("ui.accent")), Some("caf\u{e9}"));
        assert_eq!(t.get(sid!("ui.missing")), None);
        assert!(t.text(sid!("ui.missing")).starts_with('<'));
    }

    #[test]
    fn a_key_defined_twice_is_refused() {
        let mut b = StringTableBuilder::new();
        b.add("a", "1").unwrap();
        assert!(b.add("a", "2").is_err());
    }

    #[test]
    fn damaged_tables_are_refused() {
        let mut b = StringTableBuilder::new();
        b.add("a", "1").unwrap();
        let mut bytes = b.finish();
        assert!(StringTable::from_bytes(&bytes[..bytes.len() - 1]).is_err());
        bytes[0] = b'X';
        assert!(StringTable::from_bytes(&bytes).is_err());
    }
}
