//! Updates for an installed app, started by the app itself:
//!
//! ```text
//! "<install folder>\uninstall.exe" --check-update   exit 10: a newer version is out, 0: up to date
//! "<install folder>\uninstall.exe" --update          download it and open its setup window
//! "<install folder>\uninstall.exe" --update --silent and install it without a window
//! ```
//!
//! The installed uninstaller is this runtime with the app's manifest, so it
//! knows the feed (`[updates] feed`). It reads the feed's JSON, and when it
//! names a newer version, downloads that setup file over https only, checks
//! its SHA-256 and that it really is a setup file for this same app, then
//! runs it. Downloads use Windows' own curl.exe.

use std::fs::File;
use std::path::{Path, PathBuf};
use std::process::Command;

use bach_pack::container;
use bach_pack::i18n::Key;
use bach_pack::manifest::FeedEntry;
use sha2::{Digest, Sha256};

use crate::launch::Args;
use crate::package::Package;
use crate::system::host;
use crate::texts::Texts;

/// Exit code of `--check-update` when a newer version is out.
pub const UPDATE_AVAILABLE: i32 = 10;

pub fn run(args: &Args, package: &Package) -> i32 {
    host::attach_parent_console();
    let manifest = &package.manifest;
    let texts = Texts::new(manifest, args.lang.as_deref());
    let Some(updates) = &manifest.updates else {
        eprintln!("{} has no update feed ([updates] feed in its bach.toml).", manifest.app.name);
        return 2;
    };
    let work = std::env::temp_dir().join(format!("{}-update", manifest.app.id.replace(' ', "-")));
    let result = (|| -> Result<i32, String> {
        std::fs::create_dir_all(&work).map_err(|err| err.to_string())?;
        let feed_file = work.join("feed.json");
        download(&updates.feed, &feed_file)?;
        let text = std::fs::read_to_string(&feed_file).map_err(|err| err.to_string())?;
        let entry: FeedEntry = serde_json::from_str(&text).map_err(|_| "the update feed isn't valid".to_string())?;
        let newer = bach_pack::version::compare(&entry.version, &manifest.app.version) == Some(std::cmp::Ordering::Greater);
        if !newer {
            println!("{} {} is up to date.", manifest.app.name, manifest.app.version);
            return Ok(0);
        }
        if args.check_update {
            println!("{} {} is available.", manifest.app.name, entry.version);
            return Ok(UPDATE_AVAILABLE);
        }
        eprintln!("{}", texts.get(Key::step_downloading));
        let setup = fetch_setup(&entry, &work, &manifest.app.id)?;
        let mut command = Command::new(&setup);
        if args.silent {
            command.arg("--silent");
        }
        if let Some(lang) = &args.lang {
            command.args(["--lang", lang]);
        }
        if args.silent {
            // Whoever asked for a silent update can wait for its result.
            let status = command.status().map_err(|err| err.to_string())?;
            Ok(status.code().unwrap_or(1))
        } else {
            command.spawn().map_err(|err| err.to_string())?;
            Ok(0)
        }
    })();
    let _ = std::fs::remove_file(work.join("feed.json"));
    match result {
        Ok(code) => code,
        Err(error) => {
            eprintln!("{}", texts.with(Key::error_download, &[("error", &error)]));
            1
        }
    }
}

/// Downloads the setup file the feed names, and checks it before use.
fn fetch_setup(entry: &FeedEntry, work: &Path, app_id: &str) -> Result<PathBuf, String> {
    if !bach_pack::manifest::extras::is_https(&entry.url) {
        return Err("the update feed points at a link that isn't https".into());
    }
    let expected = entry.sha256.trim().to_ascii_lowercase();
    if expected.len() != 64 || !expected.chars().all(|c| c.is_ascii_hexdigit()) {
        return Err("the update feed has no valid SHA-256".into());
    }
    let partial = work.join("setup.exe.partial");
    download(&entry.url, &partial)?;
    let bytes = std::fs::read(&partial).map_err(|err| err.to_string())?;
    if format!("{:x}", Sha256::digest(&bytes)) != expected {
        let _ = std::fs::remove_file(&partial);
        return Err("the download doesn't match the feed's SHA-256".into());
    }
    let same_app = File::open(&partial)
        .ok()
        .and_then(|mut file| {
            let layout = container::locate(&mut file).ok()??;
            let manifest = container::read_manifest(&mut file, &layout).ok()?;
            Some(layout.has_archive() && manifest.app.id == app_id)
        })
        .unwrap_or(false);
    if !same_app {
        let _ = std::fs::remove_file(&partial);
        return Err("the download isn't a setup file for this app".into());
    }
    let version: String = entry.version.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-')).collect();
    let setup = work.join(format!("Setup-{version}.exe"));
    let _ = std::fs::remove_file(&setup);
    std::fs::rename(&partial, &setup).map_err(|err| err.to_string())?;
    Ok(setup)
}

/// Fetches an https URL with Windows' own curl.exe (never one on PATH),
/// refusing anything but https, redirects included.
fn download(url: &str, to: &Path) -> Result<(), String> {
    let system = std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    let curl = system.join("System32").join("curl.exe");
    if !curl.is_file() {
        return Err("this Windows has no curl.exe (Windows 10 1803 or later has it)".into());
    }
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let output = Command::new(curl)
        .args(["--fail", "--location", "--silent", "--show-error", "--proto", "=https", "--proto-redir", "=https"])
        .args(["--retry", "2", "--max-time", "900", "--output"])
        .arg(to)
        .arg(url)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .map_err(|err| err.to_string())?;
    if output.status.success() {
        Ok(())
    } else {
        let reason = String::from_utf8_lossy(&output.stderr).trim().trim_start_matches("curl: ").to_string();
        Err(if reason.is_empty() { format!("{url} couldn't be downloaded") } else { reason })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_http_is_refused() {
        let to = std::env::temp_dir().join(format!("bach-updater-{}", std::process::id()));
        assert!(download("http://example.com/", &to).is_err());
        let _ = std::fs::remove_file(to);
    }
}
