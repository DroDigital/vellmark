//! The valley's ambience, synthesized: wind that gusts, leaves that hiss in it near trees,
//! rain with drops striking near and far, the river's run and burble, and birds by day.
//! Every layer varies by chance, so that nothing repeats the same way twice. Voices live
//! in fixed pools; nothing is allocated once made.

use crate::dsp::{pan, Pink, Rng, Sine, Smooth, Svf};
use crate::SAMPLE_RATE;

const SR: f32 = SAMPLE_RATE as f32;

/// What the listener's surroundings ask of the ambience, each 0 to 1.
#[derive(Clone, Copy, Debug, Default)]
pub struct Settings {
    pub wind: f32,
    pub rain: f32,
    pub water: f32,
    pub trees: f32,
    pub daylight: f32,
    pub open: f32,
}

pub struct Ambience {
    rng: Rng,
    wind: Smooth,
    rain: Smooth,
    water: Smooth,
    trees: Smooth,
    birdsong: Smooth,
    target: Settings,
    air: Air,
    leaves: Leaves,
    rainfall: Rainfall,
    river: River,
    birds: Birds,
}

impl Ambience {
    pub fn new(seed: u32) -> Self {
        Self {
            rng: Rng::new(seed ^ 0xa11),
            wind: Smooth::new(0.0, 0.8),
            rain: Smooth::new(0.0, 1.5),
            water: Smooth::new(0.0, 0.6),
            trees: Smooth::new(0.0, 0.8),
            birdsong: Smooth::new(0.0, 2.0),
            target: Settings::default(),
            air: Air::new(seed),
            leaves: Leaves::new(seed),
            rainfall: Rainfall::new(seed),
            river: River::new(seed),
            birds: Birds::new(seed),
        }
    }

    pub fn set(&mut self, s: Settings) {
        self.target = s;
    }

    /// The next stereo frame.
    pub fn next(&mut self) -> (f32, f32) {
        let t = self.target;
        let wind = self.wind.next(t.wind.clamp(0.0, 1.0));
        let rain = self.rain.next(t.rain.clamp(0.0, 1.0));
        let water = self.water.next(t.water.clamp(0.0, 1.0));
        let trees = self.trees.next(t.trees.clamp(0.0, 1.0));
        // Birds sing by day, in the open, and fall quiet in rain and strong wind.
        let birdsong = self.birdsong.next(t.daylight.clamp(0.0, 1.0) * t.open.clamp(0.0, 1.0) * (1.0 - t.rain.clamp(0.0, 1.0)).powi(2) * (1.0 - 0.6 * t.wind.clamp(0.0, 1.0)));
        let (gust, a) = self.air.next(wind, &mut self.rng);
        let b = self.leaves.next(wind * trees, gust);
        let c = self.rainfall.next(rain, &mut self.rng);
        let d = self.river.next(water, &mut self.rng);
        let e = self.birds.next(birdsong, &mut self.rng);
        (a.0 + b.0 + c.0 + d.0 + e.0, a.1 + b.1 + c.1 + d.1 + e.1)
    }

    /// For tests: how many bird calls have begun.
    pub fn calls_begun(&self) -> u32 {
        self.birds.calls
    }
}

/// Wind: air rushing through a band that rises as the gust grows, a low roll beneath,
/// and in a strong gust a faint whistle. Each ear hears its own air.
struct Air {
    noise: [Pink; 2],
    band: [Svf; 2],
    low: [Svf; 2],
    whistle: Svf,
    gust: Smooth,
    gust_target: f32,
    until_change: u32,
}

impl Air {
    fn new(seed: u32) -> Self {
        Self {
            noise: [Pink::new(seed ^ 0x11), Pink::new(seed ^ 0x22)],
            band: Default::default(),
            low: Default::default(),
            whistle: Svf::default(),
            gust: Smooth::new(0.5, 1.6),
            gust_target: 0.5,
            until_change: 0,
        }
    }

    /// The gust's strength now, and the frame.
    fn next(&mut self, wind: f32, rng: &mut Rng) -> (f32, (f32, f32)) {
        if self.until_change == 0 {
            self.gust_target = rng.range(0.25, 1.0);
            self.until_change = (rng.range(1.2, 4.5) * SR) as u32;
        }
        self.until_change -= 1;
        let gust = self.gust.next(self.gust_target);
        if wind < 1.0e-4 {
            return (gust, (0.0, 0.0));
        }
        let level = wind.powf(1.4) * (0.3 + 0.7 * gust);
        let centre = 220.0 + 700.0 * gust * wind;
        let mut out = [0.0; 2];
        for ear in 0..2 {
            let n = self.noise[ear].next();
            let rush = self.band[ear].next(n, centre * (1.0 + 0.06 * ear as f32), 1.1).band;
            let roll = self.low[ear].next(n, 110.0, 0.7).low;
            out[ear] = (rush * 0.9 + roll * 0.6) * level;
        }
        let w = self.whistle.next(self.noise[0].next(), 650.0 + 900.0 * gust, 14.0).band;
        let whistle = w * (wind * gust).powi(3) * 0.35;
        (gust, (out[0] + whistle * 0.7, out[1] + whistle))
    }
}

/// Leaves: a bright hiss that flutters as the gusts move the trees.
struct Leaves {
    noise: [Pink; 2],
    high: [Svf; 2],
    flutter_noise: Pink,
    flutter: Svf,
}

impl Leaves {
    fn new(seed: u32) -> Self {
        Self { noise: [Pink::new(seed ^ 0x33), Pink::new(seed ^ 0x44)], high: Default::default(), flutter_noise: Pink::new(seed ^ 0x55), flutter: Svf::default() }
    }

    fn next(&mut self, amount: f32, gust: f32) -> (f32, f32) {
        if amount < 1.0e-4 {
            return (0.0, 0.0);
        }
        let flutter = self.flutter.next(self.flutter_noise.next(), 7.0, 0.7).low.abs() * 6.0;
        let level = amount * gust * gust * (0.4 + flutter.min(1.5)) * 0.5;
        let l = self.high[0].next(self.noise[0].next(), 4200.0, 0.8).high;
        let r = self.high[1].next(self.noise[1].next(), 4600.0, 0.8).high;
        (l * level, r * level)
    }
}

const DROPS: usize = 12;

#[derive(Clone, Default)]
struct Drop {
    filter: Svf,
    frequency: f32,
    left: u32,
    level: f32,
    pan: f32,
    /// The strike that sets it ringing, on its first sample.
    struck: bool,
}

/// Rain: a hiss as wide as the sky, and drops striking leaves, ground, and stone about.
struct Rainfall {
    noise: [Pink; 2],
    hiss: [Svf; 2],
    drops: [Drop; DROPS],
}

impl Rainfall {
    fn new(seed: u32) -> Self {
        Self { noise: [Pink::new(seed ^ 0x66), Pink::new(seed ^ 0x77)], hiss: Default::default(), drops: Default::default() }
    }

    fn next(&mut self, rain: f32, rng: &mut Rng) -> (f32, f32) {
        if rain < 1.0e-4 {
            return (0.0, 0.0);
        }
        let mut out = (0.0, 0.0);
        for ear in 0..2 {
            let h = self.hiss[ear].next(self.noise[ear].next(), 3200.0, 0.6);
            let v = (h.band * 0.6 + h.high * 0.3) * rain.powf(0.8) * 0.36;
            if ear == 0 {
                out.0 += v;
            } else {
                out.1 += v;
            }
        }
        // New drops, as often as the rain is heavy.
        if rng.unit() < rain * 140.0 / SR {
            if let Some(d) = self.drops.iter_mut().find(|d| d.left == 0) {
                d.frequency = rng.range(900.0, 5200.0);
                d.left = (rng.range(0.012, 0.04) * SR) as u32;
                d.level = rng.range(0.15, 1.0) * 0.6;
                d.pan = rng.signed();
                d.filter = Svf::default();
                d.struck = false;
            }
        }
        for d in self.drops.iter_mut().filter(|d| d.left > 0) {
            d.left -= 1;
            let strike = if d.struck { 0.0 } else { 1.0 };
            d.struck = true;
            let v = d.filter.next(strike, d.frequency, 9.0).band * d.level;
            let (l, r) = pan(v, d.pan);
            out.0 += l;
            out.1 += r;
        }
        out
    }
}

const BURBLES: usize = 5;

/// The river: a low run of water, and over it burbles, resonances that jump from pitch
/// to pitch as the water turns over stones.
struct River {
    noise: [Pink; 2],
    run: [Svf; 2],
    burble_noise: Pink,
    burbles: [Svf; BURBLES],
    pitch: [Smooth; BURBLES],
    target: [f32; BURBLES],
    until: [u32; BURBLES],
}

impl River {
    fn new(seed: u32) -> Self {
        Self {
            noise: [Pink::new(seed ^ 0x88), Pink::new(seed ^ 0x99)],
            run: Default::default(),
            burble_noise: Pink::new(seed ^ 0xaa),
            burbles: Default::default(),
            pitch: std::array::from_fn(|_| Smooth::new(600.0, 0.008)),
            target: [600.0; BURBLES],
            until: [0; BURBLES],
        }
    }

    fn next(&mut self, water: f32, rng: &mut Rng) -> (f32, f32) {
        if water < 1.0e-4 {
            return (0.0, 0.0);
        }
        let l = self.run[0].next(self.noise[0].next(), 480.0, 0.8).low;
        let r = self.run[1].next(self.noise[1].next(), 520.0, 0.8).low;
        let n = self.burble_noise.next();
        let mut burble = [0.0f32; 2];
        for k in 0..BURBLES {
            if self.until[k] == 0 {
                self.target[k] = rng.range(320.0, 1900.0);
                self.until[k] = (rng.range(0.03, 0.16) * SR) as u32;
            }
            self.until[k] -= 1;
            let f = self.pitch[k].next(self.target[k]);
            let b = self.burbles[k].next(n, f, 7.0).band;
            burble[k % 2] += b;
        }
        let level = water * 0.38;
        ((l * 0.7 + burble[0] * 0.5) * level, (r * 0.7 + burble[1] * 0.5) * level)
    }
}

/// Birdsong: calls of a few notes from one of three kinds of bird somewhere about,
/// each note a sweep or a trill, with silence between calls as long as it likes.
struct Birds {
    tone: Sine,
    trill: Sine,
    /// Samples until the next call begins.
    wait: u32,
    /// The call under way: notes left, and the note's state.
    notes: u32,
    kind: u32,
    note_at: u32,
    note_len: u32,
    gap: u32,
    from: f32,
    to: f32,
    level: f32,
    pan: f32,
    calls: u32,
}

impl Birds {
    fn new(seed: u32) -> Self {
        let mut rng = Rng::new(seed ^ 0xb1);
        Self {
            tone: Sine::default(),
            trill: Sine::default(),
            wait: (rng.range(0.3, 2.0) * SR) as u32,
            notes: 0,
            kind: 0,
            note_at: 0,
            note_len: 0,
            gap: 0,
            from: 0.0,
            to: 0.0,
            level: 0.0,
            pan: 0.0,
            calls: 0,
        }
    }

    fn begin_note(&mut self, rng: &mut Rng) {
        let (from, to, len) = match self.kind {
            // A falling whistle.
            0 => (rng.range(3800.0, 4600.0), rng.range(2400.0, 2900.0), rng.range(0.12, 0.22)),
            // A quick rising chirp.
            1 => (rng.range(2300.0, 2900.0), rng.range(4800.0, 5600.0), rng.range(0.05, 0.09)),
            // A trilled note, nearly level.
            _ => {
                let f = rng.range(3200.0, 3900.0);
                (f, f * rng.range(0.95, 1.05), rng.range(0.18, 0.35))
            }
        };
        self.from = from;
        self.to = to;
        self.note_len = (len * SR) as u32;
        self.note_at = 0;
        self.gap = (rng.range(0.04, 0.14) * SR) as u32;
    }

    fn next(&mut self, song: f32, rng: &mut Rng) -> (f32, f32) {
        if self.notes == 0 {
            if self.wait > 0 {
                self.wait -= 1;
                return (0.0, 0.0);
            }
            // The next call, sooner the more there is to sing about.
            self.wait = (rng.range(0.8, 5.0) / song.max(0.05) * SR) as u32;
            if song < 0.05 {
                return (0.0, 0.0);
            }
            self.kind = rng.next_u32() % 3;
            self.notes = 2 + rng.next_u32() % 5;
            self.level = rng.range(0.04, 0.12) * song;
            self.pan = rng.signed() * 0.85;
            self.calls += 1;
            self.begin_note(rng);
        }
        if self.note_at < self.note_len {
            let t = self.note_at as f32 / self.note_len as f32;
            let f = self.from + (self.to - self.from) * t;
            let envelope = (t * std::f32::consts::PI).sin().powf(0.7);
            let mut v = self.tone.next(f) * envelope;
            if self.kind == 2 {
                v *= 0.5 + 0.5 * self.trill.next(32.0);
            }
            self.note_at += 1;
            return pan(v * self.level, self.pan);
        }
        if self.gap > 0 {
            self.gap -= 1;
            return (0.0, 0.0);
        }
        self.notes -= 1;
        if self.notes > 0 {
            self.begin_note(rng);
        }
        (0.0, 0.0)
    }
}
