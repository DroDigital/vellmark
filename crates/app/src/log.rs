//! The log file and the crash log, in the user data folder.

use std::io::Write;
use std::sync::Mutex;

static LOG: Mutex<Option<std::fs::File>> = Mutex::new(None);

pub fn open() {
    let path = crate::paths::user_data_dir().join("vellmark.log");
    *LOG.lock().unwrap() = std::fs::File::create(path).ok();
}

pub fn line(text: &str) {
    eprintln!("{text}");
    if let Ok(mut log) = LOG.lock() {
        if let Some(file) = log.as_mut() {
            let _ = writeln!(file, "{text}");
        }
    }
}

/// Writes a crash log beside the saves when the game panics, then lets the default
/// handler print as usual. Once saving exists, the emergency save is made here too.
pub fn install_panic_hook() {
    let default = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        let stamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let path = crate::paths::user_data_dir().join(format!("crash-{stamp}.log"));
        let backtrace = std::backtrace::Backtrace::force_capture();
        let report = format!(
            "Vellmark {} crashed.\n{info}\n\n{backtrace}\n",
            env!("CARGO_PKG_VERSION")
        );
        let _ = std::fs::write(&path, &report);
        line(&format!("crash log written to {}", path.display()));
        default(info);
    }));
}
