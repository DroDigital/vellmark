//! The valley's roads, as baked from `assets/terrain/valley.ron`: each by its key, with
//! its width and the points it runs through across the ground, for what follows a road
//! at run time (the Line, and later the map and people's ways). One table, `roads.table`.

/// The asset key of the roads table.
pub const ROADS_KEY: &str = "roads.table";

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Road {
    pub key: String,
    pub width: f32,
    /// Across the ground: x and z, in metres.
    pub points: Vec<[f32; 2]>,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct RoadTable {
    pub roads: Vec<Road>,
}

impl RoadTable {
    const MAGIC: &'static [u8; 8] = b"VLMKRDS1";

    pub fn road(&self, key: &str) -> Option<&Road> {
        self.roads.iter().find(|r| r.key == key)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Self::MAGIC.to_vec();
        out.extend_from_slice(&(self.roads.len() as u32).to_le_bytes());
        for r in &self.roads {
            out.extend_from_slice(&(r.key.len() as u32).to_le_bytes());
            out.extend_from_slice(r.key.as_bytes());
            out.extend_from_slice(&r.width.to_le_bytes());
            out.extend_from_slice(&(r.points.len() as u32).to_le_bytes());
            for p in &r.points {
                out.extend_from_slice(&p[0].to_le_bytes());
                out.extend_from_slice(&p[1].to_le_bytes());
            }
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 12 || &bytes[..8] != Self::MAGIC {
            return Err("not a table of roads".into());
        }
        let mut at = 8;
        let mut take = |n: usize| -> Result<&[u8], String> {
            let b = bytes.get(at..at + n).ok_or("the table of roads is cut short")?;
            at += n;
            Ok(b)
        };
        let u32_at = |b: &[u8]| u32::from_le_bytes(b.try_into().unwrap());
        let f32_at = |b: &[u8]| f32::from_le_bytes(b.try_into().unwrap());
        let count = u32_at(take(4)?) as usize;
        let mut roads = Vec::with_capacity(count.min(1024));
        for _ in 0..count {
            let n = u32_at(take(4)?) as usize;
            let key = String::from_utf8(take(n)?.to_vec()).map_err(|_| "a road's key is not text".to_string())?;
            let width = f32_at(take(4)?);
            let points_n = u32_at(take(4)?) as usize;
            let mut points = Vec::with_capacity(points_n.min(4096));
            for _ in 0..points_n {
                let x = f32_at(take(4)?);
                let z = f32_at(take(4)?);
                points.push([x, z]);
            }
            roads.push(Road { key, width, points });
        }
        Ok(Self { roads })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_table_round_trips_and_a_cut_one_is_refused() {
        let t = RoadTable { roads: vec![Road { key: "penance_road".into(), width: 4.2, points: vec![[-92.0, 72.0], [24.0, -22.0]] }] };
        let bytes = t.to_bytes();
        assert_eq!(RoadTable::from_bytes(&bytes).unwrap(), t);
        assert!(RoadTable::from_bytes(&bytes[..bytes.len() - 3]).is_err());
        assert!(RoadTable::from_bytes(b"VLMKVOX1\0\0\0\0").is_err());
    }
}
