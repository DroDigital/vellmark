//! Levels of detail: which meshes are coarser forms of which. A chain starts at the
//! finest mesh, the one placed objects name, and runs to the far form. The renderer
//! chooses among them by how large the thing is on screen.

use crate::AssetId;

const MAGIC: &[u8; 8] = b"VLMKLOD1";

/// The asset key of the one table of chains.
pub const MESH_LEVELS_KEY: &str = "mesh.levels";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct MeshLevels {
    /// Each chain, finest first. Every chain has the same number of levels.
    pub chains: Vec<Vec<AssetId>>,
}

impl MeshLevels {
    /// The chain that starts at `finest`, if any.
    pub fn chain(&self, finest: AssetId) -> Option<&[AssetId]> {
        self.chains.iter().find(|c| c.first() == Some(&finest)).map(|c| c.as_slice())
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.extend_from_slice(&(self.chains.len() as u32).to_le_bytes());
        for chain in &self.chains {
            out.extend_from_slice(&(chain.len() as u32).to_le_bytes());
            for id in chain {
                out.extend_from_slice(&id.0.to_le_bytes());
            }
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 12 || &bytes[..8] != MAGIC {
            return Err("not a table of mesh levels".into());
        }
        let mut at = 8;
        let u32_at = |at: &mut usize| -> Result<u32, String> {
            let b = bytes.get(*at..*at + 4).ok_or("the table of mesh levels is cut short")?;
            *at += 4;
            Ok(u32::from_le_bytes(b.try_into().unwrap()))
        };
        let count = u32_at(&mut at)? as usize;
        let mut chains = Vec::with_capacity(count.min(4096));
        for _ in 0..count {
            let n = u32_at(&mut at)? as usize;
            let mut chain = Vec::with_capacity(n.min(16));
            for _ in 0..n {
                let b = bytes.get(at..at + 8).ok_or("the table of mesh levels is cut short")?;
                at += 8;
                chain.push(AssetId(u64::from_le_bytes(b.try_into().unwrap())));
            }
            chains.push(chain);
        }
        Ok(Self { chains })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn levels_round_trip() {
        let t = MeshLevels { chains: vec![vec![AssetId(1), AssetId(2), AssetId(3), AssetId(4)], vec![AssetId(9), AssetId(8), AssetId(7), AssetId(6)]] };
        let back = MeshLevels::from_bytes(&t.to_bytes()).unwrap();
        assert_eq!(back, t);
        assert_eq!(back.chain(AssetId(9)).unwrap()[3], AssetId(6));
        assert!(MeshLevels::from_bytes(&t.to_bytes()[..14]).is_err());
    }
}
