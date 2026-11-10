//! The capture tool against the software renderer.

use vellmark_harness::capture::{run, Options, RefMode, Script};
use vellmark_harness::repo_root;
use vellmark_render::ShaderLibrary;

fn options(ref_mode: RefMode) -> Options {
    Options {
        out: None,
        refs_root: repo_root().join("refs"),
        ref_mode,
        shaders: ShaderLibrary::embedded(),
        gpu: Default::default(),
    }
}

/// A machine with no graphics adapter at all cannot run these. That is allowed only
/// where it is declared, so that a missing driver in CI is noticed and not skipped.
fn no_adapter_allowed(error: &str) -> bool {
    error.contains("no usable graphics adapter") && std::env::var_os("VELLMARK_ALLOW_NO_GPU").is_some()
}

#[test]
fn two_runs_produce_identical_frames() {
    let script = Script::load(&repo_root().join("captures/s01_yard.ron")).unwrap();
    let first = match run(&script, &options(RefMode::Ignore)) {
        Ok(r) => r,
        Err(e) if no_adapter_allowed(&e) => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    let second = run(&script, &options(RefMode::Ignore)).unwrap();
    assert_eq!(first.shots.len(), second.shots.len());
    for (a, b) in first.shots.iter().zip(&second.shots) {
        assert_eq!(a.frames.len(), b.frames.len());
        for (i, (x, y)) in a.frames.iter().zip(&b.frames).enumerate() {
            assert!(x == y, "shot {} frame {i} differs between runs", a.name);
        }
    }
    assert!(first.passed(), "{}", first.summary());
}

#[test]
fn captures_match_their_reference_images() {
    // References are rendered by Mesa's software rasteriser, which is what cloud
    // sessions and the Linux CI runner have. Other machines draw the same picture with
    // different rounding, so only that one is held to the references.
    let script = Script::load(&repo_root().join("captures/s01_yard.ron")).unwrap();
    let report = match run(&script, &options(RefMode::Check)) {
        Ok(r) => r,
        Err(e) if no_adapter_allowed(&e) => return eprintln!("skipped: {e}"),
        Err(e) => panic!("{e}"),
    };
    if !report.adapter.contains("llvmpipe") {
        return eprintln!("skipped: references are checked on llvmpipe, this is {}", report.adapter);
    }
    assert!(report.passed(), "{}", report.summary());
}
