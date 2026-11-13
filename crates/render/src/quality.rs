//! The presets: Low, Medium, and High. A preset sets how much the frame asks of the GPU
//! before the governor gives anything up; the governor then holds the frame time within
//! it. Nothing inside the reveal distance differs between them.

/// A preset, by name.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Preset {
    Low,
    Medium,
    High,
}

impl Preset {
    pub fn parse(name: &str) -> Option<Self> {
        match name.to_ascii_lowercase().as_str() {
            "low" => Some(Self::Low),
            "medium" => Some(Self::Medium),
            "high" => Some(Self::High),
            _ => None,
        }
    }

    /// What the preset asks for.
    pub fn quality(self) -> Quality {
        match self {
            // For weaker machines: 60 on the M1 with room to spare.
            Self::Low => Quality {
                grass_density: 80.0,
                grass_near: 6.0,
                shadowed_lights: 1,
                particles: 0.6,
            },
            // The Showcase's target: 60 on the M1, about 150,000 blades in view.
            Self::Medium => Quality {
                grass_density: 120.0,
                grass_near: 7.0,
                shadowed_lights: 2,
                particles: 0.8,
            },
            // For stronger machines, judged by eye and by captures.
            Self::High => Quality {
                grass_density: 160.0,
                grass_near: 8.0,
                shadowed_lights: 3,
                particles: 1.0,
            },
        }
    }
}

/// What a preset asks of the frame.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Quality {
    /// Grass blades per square metre near the eye, and how far that density holds.
    pub grass_density: f32,
    pub grass_near: f32,
    /// How many lights may cast shadows at once.
    pub shadowed_lights: usize,
    /// The share of particles drawn.
    pub particles: f32,
}

impl Default for Quality {
    fn default() -> Self {
        Preset::High.quality()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_preset_asks_no_more_than_the_next() {
        let (l, m, h) = (
            Preset::Low.quality(),
            Preset::Medium.quality(),
            Preset::High.quality(),
        );
        for (a, b) in [(l, m), (m, h)] {
            assert!(a.grass_density <= b.grass_density && a.grass_near <= b.grass_near);
            assert!(a.shadowed_lights <= b.shadowed_lights && a.particles <= b.particles);
        }
        assert!(h.shadowed_lights <= crate::lights::SHADOWED_LIGHTS);
        assert!(h.grass_density <= crate::grass::MAX_DENSITY);
        assert_eq!(Preset::parse("Medium"), Some(Preset::Medium));
    }
}
