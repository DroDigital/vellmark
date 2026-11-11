//! Proof tooling: captures, reference images, the shimmer check, headless bots, and the
//! checks that read the source tree. Everything here is a development tool and none of
//! it ships.

pub mod capture;
pub mod image;
pub mod refs;
pub mod shimmer;
pub mod source_checks;

use std::path::PathBuf;

/// The root of the repository, for tools and tests that read its files.
pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..").canonicalize().expect("repository root")
}
