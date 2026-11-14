//! A plain-text log of every run, for support: what setup did, step by
//! step, and why it failed. It goes to `%TEMP%\<app id>-setup.log` (or
//! `--log FILE`), replacing the last run's, and never holds up setup: a log
//! that can't be written is simply skipped.

use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static LOG: Mutex<Option<File>> = Mutex::new(None);

/// Where a run's log goes when `--log` doesn't say.
pub fn default_path(app_id: &str) -> PathBuf {
    std::env::temp_dir().join(format!("{}-setup.log", app_id.replace(' ', "-")))
}

/// Starts the log at `path`, with a first line naming the run.
pub fn start(path: &Path, first_line: &str) {
    let Ok(mut file) = File::create(path) else {
        return;
    };
    // A byte order mark, so every editor reads the log as UTF-8.
    let _ = file.write_all("\u{feff}".as_bytes());
    if let Ok(mut log) = LOG.lock() {
        *log = Some(file);
    }
    line(first_line);
}

/// Adds a line, with the seconds since setup's log began.
pub fn line(text: &str) {
    let Ok(mut log) = LOG.lock() else {
        return;
    };
    if let Some(file) = log.as_mut() {
        let now = SystemTime::now().duration_since(UNIX_EPOCH).map(|time| time.as_secs()).unwrap_or(0);
        let _ = writeln!(file, "[{now}] {text}");
        let _ = file.flush();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn lines_land_in_the_file() {
        let path = std::env::temp_dir().join(format!("bach-log-test-{}.log", std::process::id()));
        start(&path, "Installing Hello 1.0");
        line("Copying Hello");
        let text = std::fs::read_to_string(&path).unwrap();
        assert!(text.contains("Installing Hello 1.0") && text.contains("Copying Hello"), "{text}");
        *LOG.lock().unwrap() = None;
        std::fs::remove_file(path).unwrap();
    }
}
