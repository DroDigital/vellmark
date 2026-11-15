//! The performance run. One command on the owner's Mac flies the tour's camera path
//! twice and writes every number in the Targets table of `docs/TECH.md` to one report.
//!
//! A report from a software renderer says so in its first lines and is not evidence.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use vellmark_content::CameraPath;
use vellmark_gpu::PassTiming;

/// Frames at the start of the run that are not counted, while caches settle.
const WARM_UP_FRAMES: usize = 30;
const LAPS: u32 = 2;

pub struct PerfRun {
    pub path: CameraPath,
    started: Option<Instant>,
    frame_ms: Vec<f32>,
    passes: BTreeMap<&'static str, Vec<f32>>,
    pass_order: Vec<&'static str>,
    peak_memory_mb: Option<f32>,
    last_memory_sample: Instant,
    min_scene_scale: f32,
    pub launch_to_first_frame: Duration,
    out: PathBuf,
}

/// Where the camera is, or that the run has ended.
pub enum PerfStep {
    Camera { eye: [f32; 3], yaw: f32, pitch: f32, hour: f32 },
    Finished,
}

impl PerfRun {
    pub fn new(path: CameraPath, out: Option<PathBuf>, launch_to_first_frame: Duration) -> Self {
        let out = out.unwrap_or_else(default_report_path);
        Self {
            path,
            started: None,
            frame_ms: Vec::new(),
            passes: BTreeMap::new(),
            pass_order: Vec::new(),
            peak_memory_mb: None,
            last_memory_sample: Instant::now(),
            min_scene_scale: 1.0,
            launch_to_first_frame,
            out,
        }
    }

    /// Call once a frame with the time the previous frame took.
    pub fn frame(&mut self, frame_time: Duration, passes: &[PassTiming], scene_scale: f32) -> PerfStep {
        let now = Instant::now();
        let started = *self.started.get_or_insert(now);
        self.frame_ms.push(frame_time.as_secs_f32() * 1000.0);
        for p in passes {
            if !self.pass_order.contains(&p.name) {
                self.pass_order.push(p.name);
            }
            self.passes.entry(p.name).or_default().push(p.milliseconds);
        }
        self.min_scene_scale = self.min_scene_scale.min(scene_scale);
        if now.duration_since(self.last_memory_sample) > Duration::from_secs(2) || self.peak_memory_mb.is_none() {
            self.last_memory_sample = now;
            if let Some(mb) = memory_in_use_mb() {
                self.peak_memory_mb = Some(self.peak_memory_mb.unwrap_or(0.0).max(mb));
            }
        }

        let lap = self.path.duration().max(0.001);
        let elapsed = now.duration_since(started).as_secs_f32();
        if elapsed >= lap * LAPS as f32 {
            return PerfStep::Finished;
        }
        match self.path.at(elapsed % lap) {
            Some(k) => PerfStep::Camera { eye: k.eye, yaw: k.yaw, pitch: k.pitch, hour: k.hour },
            None => PerfStep::Finished,
        }
    }

    pub fn write_report(&self, adapter: &str, software: bool, resolution: (u32, u32), present_mode: &str, pipelines_during_play: u32, preset: &str, governor: &[String]) -> PathBuf {
        let mut report = self.report(adapter, software, resolution, present_mode, pipelines_during_play);
        use std::fmt::Write;
        let _ = writeln!(report, "preset {preset}; lowest scene resolution {:.0}%", self.min_scene_scale * 100.0);
        if governor.is_empty() {
            let _ = writeln!(report, "nothing was given up");
        }
        for line in governor {
            let _ = writeln!(report, "{line}");
        }
        if let Some(parent) = self.out.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        if let Err(e) = std::fs::write(&self.out, &report) {
            crate::log::line(&format!("could not write {}: {e}", self.out.display()));
        }
        print!("{report}");
        self.out.clone()
    }

    fn report(&self, adapter: &str, software: bool, resolution: (u32, u32), present_mode: &str, pipelines_during_play: u32) -> String {
        let mut frames: Vec<f32> = self.frame_ms.iter().skip(WARM_UP_FRAMES).copied().collect();
        let count = frames.len().max(1);
        let total: f32 = frames.iter().sum();
        let average = total / count as f32;
        let longest = frames.iter().copied().fold(0.0f32, f32::max);
        frames.sort_by(|a, b| a.total_cmp(b));
        let percentile = |p: f32| frames.get(((count as f32 * p) as usize).min(count - 1)).copied().unwrap_or(0.0);
        let p99 = percentile(0.99);

        let mut r = String::new();
        let verdict = |ok: bool| if ok { "ok" } else { "OVER" };
        let _ = writeln!(r, "Vellmark performance report");
        let _ = writeln!(r, "version: {}", env!("CARGO_PKG_VERSION"));
        let _ = writeln!(r, "build: {}", if cfg!(debug_assertions) { "development (not for measuring)" } else { "release" });
        let _ = writeln!(r, "graphics: {adapter}");
        if software {
            let _ = writeln!(r, "NOT EVIDENCE: this run used a software renderer. Only the owner's Mac proves speed.");
        }
        let _ = writeln!(r, "resolution: {}x{}", resolution.0, resolution.1);
        let _ = writeln!(r, "present mode: {present_mode}");
        let _ = writeln!(r, "frames measured: {} over {:.1} s, after {WARM_UP_FRAMES} warm-up frames", frames.len(), total / 1000.0);
        let _ = writeln!(r);
        let _ = writeln!(r, "[targets]                          measured     target       ");
        let _ = writeln!(r, "average frame rate               : {:>8.1} fps  >= 60 fps    {}", 1000.0 / average.max(0.001), verdict(average <= 1000.0 / 60.0));
        let _ = writeln!(r, "slowest 1% of frames             : {:>8.2} ms   <= 20 ms     {}", p99, verdict(p99 <= 20.0));
        let _ = writeln!(r, "longest frame after loading      : {:>8.2} ms   <= 33 ms     {}", longest, verdict(longest <= 33.0));
        let _ = writeln!(r, "lowest scene resolution          : {:>8.0} %    >= 85 %      {}", self.min_scene_scale * 100.0, verdict(self.min_scene_scale >= 0.85));
        match self.peak_memory_mb {
            Some(mb) => {
                let _ = writeln!(r, "peak memory in use               : {:>8.0} MB   <= 4096 MB   {}", mb, verdict(mb <= 4096.0));
            }
            None => {
                let _ = writeln!(r, "peak memory in use               : not measured on this system");
            }
        }
        let _ = writeln!(r, "launch to first frame            : {:>8.2} s    <= 8 s to the title", self.launch_to_first_frame.as_secs_f32());
        let _ = writeln!(r, "title to play                    : not built yet (Phase S06)");
        let _ = writeln!(r, "pipelines created during play    : {:>8}      0            {}", pipelines_during_play, verdict(pipelines_during_play == 0));
        let _ = writeln!(r);
        let _ = writeln!(r, "[frame time]");
        for (name, p) in [("median", 0.5), ("90%", 0.9), ("95%", 0.95), ("99%", 0.99), ("99.9%", 0.999)] {
            let _ = writeln!(r, "{name:<8}: {:.2} ms", percentile(p));
        }
        let _ = writeln!(r);
        let _ = writeln!(r, "[gpu passes]   average ms   worst ms");
        if self.pass_order.is_empty() {
            let _ = writeln!(r, "this graphics card cannot time passes");
        }
        for name in &self.pass_order {
            let v = &self.passes[name];
            let avg = v.iter().sum::<f32>() / v.len().max(1) as f32;
            let worst = v.iter().copied().fold(0.0f32, f32::max);
            let _ = writeln!(r, "{name:<14} {avg:>8.3}   {worst:>8.3}");
        }
        let _ = writeln!(r);
        let _ = writeln!(r, "[governor]");
        r
    }
}

fn default_report_path() -> PathBuf {
    let home = std::env::var_os("HOME").or_else(|| std::env::var_os("USERPROFILE")).map(PathBuf::from);
    match home {
        Some(h) if h.join("Desktop").is_dir() => h.join("Desktop/vellmark-perf-report.txt"),
        _ => crate::paths::user_data_dir().join("vellmark-perf-report.txt"),
    }
}

/// Resident memory of this process in megabytes, where the system makes it easy to ask.
pub fn memory_in_use_mb() -> Option<f32> {
    if cfg!(target_os = "linux") {
        let statm = std::fs::read_to_string("/proc/self/statm").ok()?;
        let pages: f32 = statm.split_whitespace().nth(1)?.parse().ok()?;
        Some(pages * 4096.0 / (1024.0 * 1024.0))
    } else if cfg!(target_os = "macos") {
        // `ps` reports resident kilobytes. It is asked every two seconds, off the frame.
        let out = std::process::Command::new("ps")
            .args(["-o", "rss=", "-p", &std::process::id().to_string()])
            .output()
            .ok()?;
        let kb: f32 = String::from_utf8_lossy(&out.stdout).trim().parse().ok()?;
        Some(kb / 1024.0)
    } else {
        None
    }
}
