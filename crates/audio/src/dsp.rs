//! The few building blocks the synthesized sounds are made of. Each is a small struct
//! stepped one sample at a time, with no allocation.

use crate::SAMPLE_RATE;

const TAU: f32 = std::f32::consts::TAU;

/// A fast, deterministic random source.
#[derive(Clone)]
pub struct Rng(u32);

impl Rng {
    pub fn new(seed: u32) -> Self {
        Self(seed.wrapping_mul(0x9e37_79b9) | 1)
    }

    pub fn next_u32(&mut self) -> u32 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;
        self.0 = x;
        x
    }

    /// 0 to 1.
    pub fn unit(&mut self) -> f32 {
        (self.next_u32() >> 8) as f32 / 16_777_216.0
    }

    /// -1 to 1.
    pub fn signed(&mut self) -> f32 {
        self.unit() * 2.0 - 1.0
    }

    pub fn range(&mut self, a: f32, b: f32) -> f32 {
        a + (b - a) * self.unit()
    }
}

/// Moves towards a target with a time constant, so that changes never click.
#[derive(Clone)]
pub struct Smooth {
    value: f32,
    k: f32,
}

impl Smooth {
    /// `seconds`: how long it takes to get most of the way there.
    pub fn new(value: f32, seconds: f32) -> Self {
        Self {
            value,
            k: 1.0 - (-1.0 / (seconds.max(1.0e-4) * SAMPLE_RATE as f32)).exp(),
        }
    }

    pub fn next(&mut self, target: f32) -> f32 {
        self.value += (target - self.value) * self.k;
        self.value
    }

    pub fn value(&self) -> f32 {
        self.value
    }
}

/// Pink noise, by Paul Kellet's economical filter: the sound of air.
#[derive(Clone)]
pub struct Pink {
    rng: Rng,
    b: [f32; 3],
}

impl Pink {
    pub fn new(seed: u32) -> Self {
        Self {
            rng: Rng::new(seed),
            b: [0.0; 3],
        }
    }

    pub fn next(&mut self) -> f32 {
        let white = self.rng.signed();
        self.b[0] = 0.99765 * self.b[0] + white * 0.0990460;
        self.b[1] = 0.96300 * self.b[1] + white * 0.2965164;
        self.b[2] = 0.57000 * self.b[2] + white * 1.0526913;
        (self.b[0] + self.b[1] + self.b[2] + white * 0.1848) * 0.2
    }
}

/// A state-variable filter: low, band, and high pass at once.
#[derive(Clone, Default)]
pub struct Svf {
    low: f32,
    band: f32,
}

pub struct SvfOut {
    pub low: f32,
    pub band: f32,
    pub high: f32,
}

impl Svf {
    /// `frequency` in hertz; `q` above about 0.5.
    pub fn next(&mut self, input: f32, frequency: f32, q: f32) -> SvfOut {
        let f = 2.0
            * (std::f32::consts::PI * frequency.clamp(10.0, SAMPLE_RATE as f32 * 0.24)
                / SAMPLE_RATE as f32)
                .sin();
        let damp = 1.0 / q.max(0.5);
        self.low += f * self.band;
        let high = input - self.low - damp * self.band;
        self.band += f * high;
        // Never let a filter run away.
        if !self.band.is_finite() || !self.low.is_finite() {
            self.low = 0.0;
            self.band = 0.0;
        }
        SvfOut {
            low: self.low,
            band: self.band,
            high,
        }
    }
}

/// A sine oscillator whose frequency may change every sample.
#[derive(Clone, Default)]
pub struct Sine {
    phase: f32,
}

impl Sine {
    pub fn next(&mut self, frequency: f32) -> f32 {
        self.phase = (self.phase + frequency / SAMPLE_RATE as f32).fract();
        (self.phase * TAU).sin()
    }
}

/// Equal-power panning: -1 left, 1 right.
pub fn pan(x: f32, position: f32) -> (f32, f32) {
    let a = (position.clamp(-1.0, 1.0) + 1.0) * std::f32::consts::FRAC_PI_4;
    (x * a.cos(), x * a.sin())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noise_and_filters_stay_finite_and_bounded() {
        let mut p = Pink::new(3);
        let mut f = Svf::default();
        let mut peak = 0.0f32;
        for i in 0..SAMPLE_RATE {
            let o = f.next(p.next(), 200.0 + (i % 5000) as f32, 4.0);
            assert!(o.band.is_finite() && o.low.is_finite() && o.high.is_finite());
            peak = peak.max(o.band.abs());
        }
        assert!(peak < 4.0, "{peak}");
    }

    #[test]
    fn smoothing_reaches_its_target() {
        let mut s = Smooth::new(0.0, 0.01);
        for _ in 0..SAMPLE_RATE / 10 {
            s.next(1.0);
        }
        assert!((s.value() - 1.0).abs() < 1.0e-3);
    }
}
