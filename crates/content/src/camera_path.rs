//! A scripted camera path: keyframes the camera moves between at a steady rate. The
//! performance run flies these, and later the title and the set pieces do.
//!
//! ```text
//! magic "VLMKCAM1", count u32, then count keyframes of 7 f32:
//! eye x y z, yaw, pitch (radians), hour, seconds to reach the next keyframe
//! ```

const MAGIC: &[u8; 8] = b"VLMKCAM1";

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Keyframe {
    pub eye: [f32; 3],
    /// Radians. 0 looks north (-Z), a quarter turn looks east (+X).
    pub yaw: f32,
    /// Radians. Positive looks up.
    pub pitch: f32,
    /// Hours since midnight.
    pub hour: f32,
    /// Seconds to travel from this keyframe to the next. Ignored on the last.
    pub seconds: f32,
}

#[derive(Clone, Debug, PartialEq, Default)]
pub struct CameraPath {
    pub keys: Vec<Keyframe>,
}

impl CameraPath {
    pub fn to_bytes(&self) -> Vec<u8> {
        let mut out = Vec::with_capacity(12 + self.keys.len() * 28);
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&(self.keys.len() as u32).to_le_bytes());
        for k in &self.keys {
            for v in [
                k.eye[0], k.eye[1], k.eye[2], k.yaw, k.pitch, k.hour, k.seconds,
            ] {
                out.extend_from_slice(&v.to_le_bytes());
            }
        }
        out
    }

    pub fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        if bytes.len() < 12 || &bytes[0..8] != MAGIC {
            return Err("not a camera path");
        }
        let count = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
        if bytes.len() != 12 + count * 28 {
            return Err("camera path has the wrong length");
        }
        let f = |at: usize| f32::from_le_bytes(bytes[at..at + 4].try_into().unwrap());
        let keys = (0..count)
            .map(|i| {
                let at = 12 + i * 28;
                Keyframe {
                    eye: [f(at), f(at + 4), f(at + 8)],
                    yaw: f(at + 12),
                    pitch: f(at + 16),
                    hour: f(at + 20),
                    seconds: f(at + 24),
                }
            })
            .collect();
        Ok(Self { keys })
    }

    /// Seconds from the first keyframe to the last.
    pub fn duration(&self) -> f32 {
        self.keys.iter().rev().skip(1).map(|k| k.seconds).sum()
    }

    /// The camera at `seconds` from the start. Holds the last keyframe after the end.
    pub fn at(&self, seconds: f32) -> Option<Keyframe> {
        let last = *self.keys.last()?;
        let mut t = seconds.max(0.0);
        for pair in self.keys.windows(2) {
            let (a, b) = (pair[0], pair[1]);
            if t < a.seconds {
                let s = if a.seconds > 0.0 { t / a.seconds } else { 1.0 };
                let mix = |x: f32, y: f32| x + (y - x) * s;
                return Some(Keyframe {
                    eye: [
                        mix(a.eye[0], b.eye[0]),
                        mix(a.eye[1], b.eye[1]),
                        mix(a.eye[2], b.eye[2]),
                    ],
                    yaw: mix(a.yaw, b.yaw),
                    pitch: mix(a.pitch, b.pitch),
                    hour: mix(a.hour, b.hour),
                    seconds: a.seconds,
                });
            }
            t -= a.seconds;
        }
        Some(last)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn key(x: f32, seconds: f32) -> Keyframe {
        Keyframe {
            eye: [x, 0.0, 0.0],
            yaw: 0.0,
            pitch: 0.0,
            hour: 12.0,
            seconds,
        }
    }

    #[test]
    fn round_trip() {
        let path = CameraPath {
            keys: vec![key(0.0, 2.0), key(4.0, 1.0), key(5.0, 0.0)],
        };
        assert_eq!(CameraPath::from_bytes(&path.to_bytes()).unwrap(), path);
        assert!(CameraPath::from_bytes(&path.to_bytes()[..20]).is_err());
    }

    #[test]
    fn moves_at_a_steady_rate_and_holds_the_end() {
        let path = CameraPath {
            keys: vec![key(0.0, 2.0), key(4.0, 1.0), key(5.0, 0.0)],
        };
        assert_eq!(path.duration(), 3.0);
        assert_eq!(path.at(1.0).unwrap().eye[0], 2.0);
        assert_eq!(path.at(2.5).unwrap().eye[0], 4.5);
        assert_eq!(path.at(99.0).unwrap().eye[0], 5.0);
        assert!(CameraPath::default().at(0.0).is_none());
    }
}
