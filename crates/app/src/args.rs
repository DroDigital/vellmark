use std::path::PathBuf;

#[derive(Debug, Default)]
pub struct Args {
    /// Use a software renderer.
    pub software: bool,
    pub windowed_size: Option<(u32, u32)>,
    pub fullscreen: bool,
    /// Close the window by itself after this many frames. For smoke tests.
    pub frames: Option<u64>,
    pub perf_run: bool,
    pub out: Option<PathBuf>,
    pub capture: Option<PathBuf>,
    pub check_refs: bool,
    pub accept_refs: bool,
    /// Development builds: start with the profiler view open.
    pub profiler: bool,
    /// The scene to start in, by name: `yard` or `valley`.
    pub scene: Option<String>,
}

impl Args {
    pub fn parse() -> Result<Self, String> {
        let mut args = Self::default();
        let mut it = std::env::args().skip(1);
        while let Some(arg) = it.next() {
            let mut value = |name: &str| it.next().ok_or(format!("{name} needs a value"));
            match arg.as_str() {
                "--software" => args.software = true,
                "--fullscreen" => args.fullscreen = true,
                "--size" => {
                    let v = value("--size")?;
                    let (w, h) = v.split_once('x').ok_or("--size takes WIDTHxHEIGHT")?;
                    args.windowed_size =
                        Some((w.parse().map_err(|_| "bad width")?, h.parse().map_err(|_| "bad height")?));
                }
                "--frames" => args.frames = Some(value("--frames")?.parse().map_err(|_| "bad frame count")?),
                "--perf-run" => args.perf_run = true,
                "--out" => args.out = Some(value("--out")?.into()),
                "--capture" => args.capture = Some(value("--capture")?.into()),
                "--check-refs" => args.check_refs = true,
                "--accept-refs" => args.accept_refs = true,
                "--profiler" => args.profiler = true,
                "--scene" => args.scene = Some(value("--scene")?),
                other => return Err(format!("unknown argument `{other}`")),
            }
        }
        Ok(args)
    }
}
