//! Downloads with Windows' own `curl.exe` (never one found on `PATH`), https
//! only, redirects included. Windows 10 1803 and later ship it, so the
//! runtime carries no network code of its own.

use std::path::{Path, PathBuf};
use std::process::Command;

/// How one download behaves.
#[derive(Clone, Copy, Debug)]
pub struct Fetch {
    /// Continue a partial file instead of starting over.
    pub resume: bool,
    /// Give up after this many seconds in all.
    pub max_seconds: u32,
    /// Refuse a response bigger than this many bytes (the feed is tiny).
    pub max_bytes: Option<u64>,
}

impl Fetch {
    /// The small JSON feed: quick, and never more than a megabyte.
    pub const FEED: Fetch = Fetch { resume: false, max_seconds: 60, max_bytes: Some(1 << 20) };
    /// A setup file: as long as it takes, resuming where an earlier try stopped.
    pub const SETUP: Fetch = Fetch { resume: true, max_seconds: 3 * 3600, max_bytes: None };
}

fn curl_path() -> Result<PathBuf, String> {
    let system = std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    let curl = system.join("System32").join("curl.exe");
    if curl.is_file() {
        Ok(curl)
    } else {
        Err("this Windows has no curl.exe (Windows 10 1803 or later has it)".into())
    }
}

/// The arguments for one download, apart from the file and the link.
fn arguments(how: Fetch) -> Vec<String> {
    let mut args: Vec<String> = [
        "--fail", "--location", "--silent", "--show-error", "--proto", "=https", "--proto-redir", "=https",
        "--retry", "3", "--retry-delay", "2", "--connect-timeout", "20", "--header", "Cache-Control: no-cache",
    ]
    .iter()
    .map(|arg| arg.to_string())
    .collect();
    args.push("--max-time".into());
    args.push(how.max_seconds.to_string());
    if how.resume {
        args.extend(["--continue-at".to_string(), "-".to_string()]);
    }
    if let Some(bytes) = how.max_bytes {
        args.push("--max-filesize".into());
        args.push(bytes.to_string());
    }
    args
}

/// Fetches an https `url` into `to`.
pub fn download(url: &str, to: &Path, how: Fetch) -> Result<(), String> {
    let curl = curl_path()?;
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let output = Command::new(curl)
        .args(arguments(how))
        .arg("--output")
        .arg(to)
        .arg(url)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|err| err.to_string())?;
    if output.status.success() {
        return Ok(());
    }
    // A partial file that was complete already: curl answers 416 ("range
    // not satisfiable"), which here means there is nothing left to fetch.
    let reason = String::from_utf8_lossy(&output.stderr).trim().trim_start_matches("curl: ").to_string();
    if how.resume && reason.contains("416") {
        return Ok(());
    }
    Err(if reason.is_empty() { format!("{url} couldn't be downloaded") } else { reason })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_http_is_refused() {
        let to = std::env::temp_dir().join(format!("bach-curl-{}", std::process::id()));
        assert!(download("http://example.com/", &to, Fetch::FEED).is_err());
        let _ = std::fs::remove_file(to);
    }

    #[test]
    fn arguments_never_allow_other_schemes() {
        for how in [Fetch::FEED, Fetch::SETUP] {
            let args = arguments(how).join(" ");
            assert!(args.contains("--proto =https") && args.contains("--proto-redir =https"), "{args}");
        }
        assert!(arguments(Fetch::SETUP).join(" ").contains("--continue-at -"));
        assert!(arguments(Fetch::FEED).join(" ").contains("--max-filesize"));
    }
}
