//! The murmur's voices, as baked from `assets/data/voices.ron`: for each speaker, the
//! numbers its murmur is shaped by. One table, `voices.table`.

/// The asset key of the voices table.
pub const VOICES_KEY: &str = "voices.table";

/// A voice as the murmur speaks it.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Voice {
    /// The voice's pitch, in hertz.
    pub pitch: f32,
    /// The size of the voice: formants scaled by its inverse; 1 an adult's middle.
    pub size: f32,
    /// Syllables a second.
    pub rate: f32,
    pub breath: f32,
    pub gravel: f32,
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct VoiceTable {
    pub speakers: Vec<(String, Voice)>,
}

impl VoiceTable {
    const MAGIC: &'static [u8; 8] = b"VLMKVOX1";

    pub fn voice(&self, speaker: &str) -> Option<Voice> {
        self.speakers.iter().find(|(k, _)| k == speaker).map(|(_, v)| *v)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Self::MAGIC.to_vec();
        out.extend_from_slice(&(self.speakers.len() as u32).to_le_bytes());
        for (k, v) in &self.speakers {
            out.extend_from_slice(&(k.len() as u32).to_le_bytes());
            out.extend_from_slice(k.as_bytes());
            for f in [v.pitch, v.size, v.rate, v.breath, v.gravel] {
                out.extend_from_slice(&f.to_le_bytes());
            }
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 12 || &bytes[..8] != Self::MAGIC {
            return Err("not a table of voices".into());
        }
        let mut at = 8;
        let mut take = |n: usize| -> Result<&[u8], String> {
            let b = bytes.get(at..at + n).ok_or("the table of voices is cut short")?;
            at += n;
            Ok(b)
        };
        let count = u32::from_le_bytes(take(4)?.try_into().unwrap()) as usize;
        let mut speakers = Vec::with_capacity(count.min(1024));
        for _ in 0..count {
            let n = u32::from_le_bytes(take(4)?.try_into().unwrap()) as usize;
            let key = String::from_utf8(take(n)?.to_vec()).map_err(|_| "a speaker's key is not text".to_string())?;
            let mut f = [0.0f32; 5];
            for x in &mut f {
                *x = f32::from_le_bytes(take(4)?.try_into().unwrap());
            }
            speakers.push((key, Voice { pitch: f[0], size: f[1], rate: f[2], breath: f[3], gravel: f[4] }));
        }
        Ok(Self { speakers })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_voices_round_trip() {
        let t = VoiceTable { speakers: vec![("wick".into(), Voice { pitch: 108.0, size: 0.94, rate: 3.6, breath: 0.12, gravel: 0.35 })] };
        assert_eq!(VoiceTable::from_bytes(&t.to_bytes()).unwrap(), t);
    }
}
