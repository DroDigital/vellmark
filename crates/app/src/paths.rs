use std::path::PathBuf;

/// Where saves, settings, and logs go: each platform's standard place for user data.
pub fn user_data_dir() -> PathBuf {
    let home = || std::env::var_os("HOME").map(PathBuf::from).unwrap_or_else(|| PathBuf::from("."));
    let dir = if cfg!(target_os = "windows") {
        std::env::var_os("APPDATA").map(PathBuf::from).unwrap_or_else(home).join("Vellmark")
    } else if cfg!(target_os = "macos") {
        home().join("Library/Application Support/Vellmark")
    } else {
        std::env::var_os("XDG_DATA_HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| home().join(".local/share"))
            .join("vellmark")
    };
    let _ = std::fs::create_dir_all(&dir);
    dir
}

/// Folders that may hold the packs, most likely first.
pub fn pack_dirs() -> Vec<PathBuf> {
    let mut dirs = Vec::new();
    if let Ok(exe) = std::env::current_exe() {
        if let Some(folder) = exe.parent() {
            dirs.push(folder.join("packs"));
            // Inside a macOS app: Vellmark.app/Contents/MacOS/vellmark.
            dirs.push(folder.join("../Resources/packs"));
        }
    }
    #[cfg(feature = "dev")]
    dirs.push(repo_root().join("build/packs"));
    dirs
}

/// The repository this development build was compiled from.
#[cfg(feature = "dev")]
pub fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../..")
}
