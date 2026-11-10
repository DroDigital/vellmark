//! Hot reload. Shaders, data sheets, and text are watched on disk, and a change shows
//! in the running game in well under a second.

use notify::{RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc::{channel, Receiver};
use std::time::Instant;

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Changes {
    pub shaders: bool,
    /// Text or data sheets under `assets`.
    pub data: bool,
}

pub struct HotReload {
    _watcher: RecommendedWatcher,
    events: Receiver<(PathBuf, Instant)>,
    shaders: PathBuf,
    assets: PathBuf,
}

impl HotReload {
    pub fn watch(repo: &Path) -> Result<Self, String> {
        let (tx, events) = channel();
        let mut watcher = notify::recommended_watcher(move |result: notify::Result<notify::Event>| {
            if let Ok(event) = result {
                if event.kind.is_modify() || event.kind.is_create() {
                    let now = Instant::now();
                    for path in event.paths {
                        let _ = tx.send((path, now));
                    }
                }
            }
        })
        .map_err(|e| e.to_string())?;
        let shaders = repo.join("shaders").canonicalize().map_err(|e| e.to_string())?;
        let assets = repo.join("assets").canonicalize().map_err(|e| e.to_string())?;
        watcher.watch(&shaders, RecursiveMode::Recursive).map_err(|e| e.to_string())?;
        watcher.watch(&assets, RecursiveMode::Recursive).map_err(|e| e.to_string())?;
        Ok(Self { _watcher: watcher, events, shaders, assets })
    }

    /// What changed since the last call, and when the first change was seen. Never
    /// waits.
    pub fn poll(&mut self) -> (Changes, Option<Instant>) {
        let mut changes = Changes::default();
        let mut first = None;
        while let Ok((path, at)) = self.events.try_recv() {
            let path = path.canonicalize().unwrap_or(path);
            if path.starts_with(&self.shaders) && path.extension().is_some_and(|e| e == "wgsl") {
                changes.shaders = true;
            } else if path.starts_with(&self.assets) {
                changes.data = true;
            } else {
                continue;
            }
            first.get_or_insert(at);
        }
        (changes, first)
    }
}
