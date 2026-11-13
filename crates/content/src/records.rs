//! Generated truth: what is true of a placed thing, kept as data, from which the text on
//! its surface is written. A tree's bark records its age.

use crate::AssetId;

const MAGIC: &[u8; 8] = b"VLMKTRE1";

/// The asset key of the one table of tree records.
pub const TREE_RECORDS_KEY: &str = "trees.records";

#[derive(Clone, Debug, PartialEq)]
pub struct TreeRecord {
    /// The placed tree's object key.
    pub tree: AssetId,
    /// Its kind's key, as in `assets/trees/kinds.ron`: `oak`, `ash`.
    pub kind: String,
    /// How many years it has stood.
    pub years: u32,
    /// Round its trunk at the foot, in the metres of its bark's page.
    pub girth: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TreeRecords {
    pub trees: Vec<TreeRecord>,
}

impl TreeRecords {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.extend_from_slice(&(self.trees.len() as u32).to_le_bytes());
        for t in &self.trees {
            out.extend_from_slice(&t.tree.0.to_le_bytes());
            out.extend_from_slice(&t.years.to_le_bytes());
            out.extend_from_slice(&t.girth.to_le_bytes());
            out.extend_from_slice(&(t.kind.len() as u32).to_le_bytes());
            out.extend_from_slice(t.kind.as_bytes());
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 12 || &bytes[..8] != MAGIC {
            return Err("not a table of tree records".into());
        }
        let short = || "the table of tree records is cut short".to_string();
        let take = |at: &mut usize, n: usize| -> Result<&[u8], String> {
            let b = bytes.get(*at..*at + n).ok_or_else(short)?;
            *at += n;
            Ok(b)
        };
        let mut at = 8;
        let count = u32::from_le_bytes(take(&mut at, 4)?.try_into().unwrap()) as usize;
        let mut trees = Vec::with_capacity(count.min(65_536));
        for _ in 0..count {
            let tree = AssetId(u64::from_le_bytes(take(&mut at, 8)?.try_into().unwrap()));
            let years = u32::from_le_bytes(take(&mut at, 4)?.try_into().unwrap());
            let girth = f32::from_le_bytes(take(&mut at, 4)?.try_into().unwrap());
            let n = u32::from_le_bytes(take(&mut at, 4)?.try_into().unwrap()) as usize;
            let kind = String::from_utf8(take(&mut at, n)?.to_vec()).map_err(|_| "a tree's kind is not text".to_string())?;
            trees.push(TreeRecord { tree, kind, years, girth });
        }
        Ok(Self { trees })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn records_round_trip() {
        let t = TreeRecords {
            trees: vec![
                TreeRecord { tree: AssetId(7), kind: "oak".into(), years: 143, girth: 1.9 },
                TreeRecord { tree: AssetId(9), kind: "hawthorn".into(), years: 38, girth: 0.6 },
            ],
        };
        assert_eq!(TreeRecords::from_bytes(&t.to_bytes()).unwrap(), t);
        assert!(TreeRecords::from_bytes(&t.to_bytes()[..20]).is_err());
    }
}
