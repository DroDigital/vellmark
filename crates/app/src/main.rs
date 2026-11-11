//! Vellmark's entry point and frame loop.

mod args;
mod boot;
#[cfg(feature = "dev")]
mod dev;
mod game;
mod log;
mod paths;
mod perf;

use std::process::ExitCode;

fn main() -> ExitCode {
    let launched = std::time::Instant::now();
    log::open();
    log::install_panic_hook();

    let args = match args::Args::parse() {
        Ok(a) => a,
        Err(e) => {
            log::line(&format!("{e}"));
            return ExitCode::from(2);
        }
    };

    #[cfg(feature = "dev")]
    if let Some(script) = &args.capture {
        return dev::capture::run(script, &args);
    }
    #[cfg(not(feature = "dev"))]
    if args.capture.is_some() {
        log::line("this build has no capture tool");
        return ExitCode::from(2);
    }

    match game::run(&args, launched) {
        Ok(()) => ExitCode::SUCCESS,
        Err(()) => ExitCode::FAILURE,
    }
}
