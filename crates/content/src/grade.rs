//! A place's grade: how its light is coloured and how thick its air is through the day,
//! as keys by the hour that are blended between. One for each Quire; the Showcase has
//! the Vale's.
//!
//! ```text
//! magic "VLMKGRD2", count u32, then count keys of 14 f32:
//! hour, lift rgb, gamma rgb, gain rgb, saturation,
//! fog density (per metre at its floor), fog floor (metres above the sea), fog falloff
//! (metres in which it thins to a third)
//! ```

const MAGIC: &[u8; 8] = b"VLMKGRD2";
const KEY: usize = 14 * 4;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GradeKey {
    /// Hours since midnight.
    pub hour: f32,
    /// Added to the shadows: their tint.
    pub lift: [f32; 3],
    /// Bends the middle tones; above 1 lightens them.
    pub gamma: [f32; 3],
    /// Multiplies the highlights.
    pub gain: [f32; 3],
    /// 1 leaves colour as it is; less greys it, more strengthens it.
    pub saturation: f32,
    /// The mist: how thick at its floor, per metre; the height of its floor; and how
    /// quickly it thins above it.
    pub fog_density: f32,
    pub fog_floor: f32,
    pub fog_falloff: f32,
}

impl GradeKey {
    pub const NEUTRAL: Self = Self {
        hour: 0.0,
        lift: [0.0; 3],
        gamma: [1.0; 3],
        gain: [1.0; 3],
        saturation: 1.0,
        fog_density: 0.0,
        fog_floor: 0.0,
        fog_falloff: 10.0,
    };

    fn mix(&self, other: &Self, t: f32) -> Self {
        let m = |a: [f32; 3], b: [f32; 3]| {
            [
                a[0] + (b[0] - a[0]) * t,
                a[1] + (b[1] - a[1]) * t,
                a[2] + (b[2] - a[2]) * t,
            ]
        };
        Self {
            hour: self.hour,
            lift: m(self.lift, other.lift),
            gamma: m(self.gamma, other.gamma),
            gain: m(self.gain, other.gain),
            saturation: self.saturation + (other.saturation - self.saturation) * t,
            fog_density: self.fog_density + (other.fog_density - self.fog_density) * t,
            fog_floor: self.fog_floor + (other.fog_floor - self.fog_floor) * t,
            fog_falloff: self.fog_falloff + (other.fog_falloff - self.fog_falloff) * t,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct Grade {
    /// In order of the hour.
    pub keys: Vec<GradeKey>,
}

impl Grade {
    /// The grade at an hour, blended between the keys either side, round the clock.
    pub fn at(&self, hour: f32) -> GradeKey {
        let n = self.keys.len();
        if n == 0 {
            return GradeKey::NEUTRAL;
        }
        let hour = hour.rem_euclid(24.0);
        let after = self.keys.iter().position(|k| k.hour > hour).unwrap_or(0);
        let before = (after + n - 1) % n;
        let (a, b) = (&self.keys[before], &self.keys[after]);
        let span = (b.hour - a.hour).rem_euclid(24.0);
        let into = (hour - a.hour).rem_euclid(24.0);
        let t = if span > 0.0 { into / span } else { 0.0 };
        let t = t * t * (3.0 - 2.0 * t);
        a.mix(b, t)
    }

    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = MAGIC.to_vec();
        out.extend_from_slice(&(self.keys.len() as u32).to_le_bytes());
        for k in &self.keys {
            let values = [
                k.hour,
                k.lift[0],
                k.lift[1],
                k.lift[2],
                k.gamma[0],
                k.gamma[1],
                k.gamma[2],
                k.gain[0],
                k.gain[1],
                k.gain[2],
                k.saturation,
                k.fog_density,
                k.fog_floor,
                k.fog_falloff,
            ];
            for v in values {
                out.extend_from_slice(&v.to_le_bytes());
            }
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, String> {
        if bytes.len() < 12 || &bytes[..8] != MAGIC {
            return Err("not a colour grade".into());
        }
        let count = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        if bytes.len() != 12 + count * KEY {
            return Err("colour grade has the wrong length".into());
        }
        let f = |at: usize| f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let keys = (0..count)
            .map(|i| {
                let o = 12 + i * KEY;
                GradeKey {
                    hour: f(o),
                    lift: [f(o + 4), f(o + 8), f(o + 12)],
                    gamma: [f(o + 16), f(o + 20), f(o + 24)],
                    gain: [f(o + 28), f(o + 32), f(o + 36)],
                    saturation: f(o + 40),
                    fog_density: f(o + 44),
                    fog_floor: f(o + 48),
                    fog_falloff: f(o + 52),
                }
            })
            .collect();
        Ok(Self { keys })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_grade_blends_round_the_clock_and_reads_back() {
        let key = |hour: f32, s: f32| GradeKey {
            hour,
            saturation: s,
            ..GradeKey::NEUTRAL
        };
        let grade = Grade {
            keys: vec![key(6.0, 1.0), key(18.0, 0.5)],
        };
        assert_eq!(grade.at(6.0).saturation, 1.0);
        assert!((grade.at(12.0).saturation - 0.75).abs() < 1e-6);
        // Midnight is halfway from 18 to 6, round the clock.
        assert!((grade.at(0.0).saturation - 0.75).abs() < 1e-6);
        assert_eq!(Grade::from_bytes(&grade.to_bytes()).unwrap(), grade);
    }
}
