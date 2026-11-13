use crate::args::Args;
use crate::paths::repo_root;
use std::path::Path;
use std::process::ExitCode;
use vellmark_gpu::GpuOptions;
use vellmark_harness::capture::{Options, RefMode, Script, run as run_capture};
use vellmark_render::ShaderLibrary;

/// `vellmark --capture <script.ron> [--out DIR] [--check-refs | --accept-refs]`
pub fn run(script_path: &Path, args: &Args) -> ExitCode {
    let script = match Script::load(script_path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("{e}");
            return ExitCode::from(2);
        }
    };
    let Some(root) = repo_root() else {
        eprintln!("the capture tool runs from the repository");
        return ExitCode::from(2);
    };
    let options = Options {
        out: Some(
            args.out
                .clone()
                .unwrap_or_else(|| root.join("build/captures")),
        ),
        refs_root: root.join("refs"),
        ref_mode: if args.accept_refs {
            RefMode::Accept
        } else if args.check_refs {
            RefMode::Check
        } else {
            RefMode::Ignore
        },
        shaders: ShaderLibrary::with_folder(root.join("shaders")),
        gpu: GpuOptions {
            software: args.software,
            ..Default::default()
        },
    };
    match run_capture(&script, &options) {
        Ok(report) => {
            print!("{}", report.summary());
            if report.passed() {
                ExitCode::SUCCESS
            } else {
                ExitCode::FAILURE
            }
        }
        Err(e) => {
            eprintln!("capture failed: {e}");
            ExitCode::FAILURE
        }
    }
}
