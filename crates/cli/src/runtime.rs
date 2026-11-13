//! Finding the setup runtime (`bach-setup.exe`) every installer is built on.
//!
//! In order: `--runtime` (or `BACH_RUNTIME`), the file next to `bach`, the
//! copy a `cargo build` left in this checkout, a copy downloaded earlier,
//! and finally the one published with this version of Bach on GitHub,
//! checked against its SHA-256 before it's used. So `bach build` works from
//! a release zip, from source, and on macOS or Linux CI with nothing but
//! the `bach` binary.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use sha2::{Digest, Sha256};

/// The runtime's file name, next to `bach.exe` in a release and in
/// `target\<profile>` after `cargo build`.
pub const FILE_NAME: &str = "bach-setup.exe";

/// Where releases live. A fork can point `BACH_RELEASES` elsewhere.
const RELEASES: &str = "https://github.com/usebach/bach/releases/download";

/// Where the runtime was found, for `bach doctor`.
pub enum Found {
    Here(PathBuf),
    /// Not on disk yet; this URL would be downloaded.
    Downloadable(String),
}

/// The runtime to use, downloading it when there's no other way.
pub fn find(given: Option<&Path>) -> Result<PathBuf> {
    match locate(given)? {
        Found::Here(path) => Ok(path),
        Found::Downloadable(url) => download(&url),
    }
}

/// Like [`find`], without downloading.
pub fn locate(given: Option<&Path>) -> Result<Found> {
    if let Some(path) = given {
        if !path.is_file() {
            bail!("the setup runtime {} doesn't exist", path.display());
        }
        return Ok(Found::Here(path.to_path_buf()));
    }
    let here = std::env::current_exe().context("couldn't find where bach is")?;
    let beside = here.parent().map(|dir| dir.join(FILE_NAME));
    let candidates = beside.into_iter().chain(checkout_builds(&here)).chain(std::iter::once(cached()?));
    for candidate in candidates {
        if candidate.is_file() {
            return Ok(Found::Here(candidate));
        }
    }
    Ok(Found::Downloadable(url()))
}

/// In a checkout, `bach` may run from `target\debug` while the runtime was
/// built in `target\release`, or the other way round.
fn checkout_builds(bach: &Path) -> Vec<PathBuf> {
    let Some(target) = bach.parent().and_then(Path::parent) else {
        return Vec::new();
    };
    if target.file_name().is_none_or(|name| name != "target") {
        return Vec::new();
    }
    ["release", "debug"].iter().map(|profile| target.join(profile).join(FILE_NAME)).collect()
}

/// The download cache: `%LOCALAPPDATA%\bach\runtime\<version>` on Windows,
/// `$XDG_CACHE_HOME/bach/runtime/<version>` (or `~/.cache/…`) elsewhere.
fn cached() -> Result<PathBuf> {
    let base = if cfg!(windows) {
        std::env::var_os("LOCALAPPDATA").map(PathBuf::from)
    } else {
        std::env::var_os("XDG_CACHE_HOME")
            .map(PathBuf::from)
            .or_else(|| std::env::var_os("HOME").map(|home| PathBuf::from(home).join(".cache")))
    };
    let base = base.unwrap_or_else(std::env::temp_dir);
    Ok(base.join("bach").join("runtime").join(env!("CARGO_PKG_VERSION")).join(FILE_NAME))
}

fn url() -> String {
    let releases = std::env::var("BACH_RELEASES").unwrap_or_else(|_| RELEASES.to_string());
    format!("{}/v{}/{FILE_NAME}", releases.trim_end_matches('/'), env!("CARGO_PKG_VERSION"))
}

/// Downloads the runtime and its checksum, checks both, and keeps it in the
/// cache for next time.
fn download(url: &str) -> Result<PathBuf> {
    if std::env::var_os("BACH_NO_DOWNLOAD").is_some() {
        bail!("{FILE_NAME} isn't next to bach and BACH_NO_DOWNLOAD is set; pass --runtime or set BACH_RUNTIME");
    }
    let target = cached()?;
    let dir = target.parent().context("the cache has no folder")?;
    std::fs::create_dir_all(dir).with_context(|| format!("couldn't create {}", dir.display()))?;
    eprintln!("  Downloading the setup runtime for Bach {} (once)…", env!("CARGO_PKG_VERSION"));
    let partial = dir.join(format!("{FILE_NAME}.partial"));
    let checksum = dir.join(format!("{FILE_NAME}.sha256"));
    let result = (|| -> Result<()> {
        fetch(url, &partial)?;
        fetch(&format!("{url}.sha256"), &checksum)?;
        let expected = std::fs::read_to_string(&checksum)?;
        let expected = expected.split_whitespace().next().unwrap_or_default().to_ascii_lowercase();
        let bytes = std::fs::read(&partial)?;
        let actual = format!("{:x}", Sha256::digest(&bytes));
        if expected.len() != 64 || expected != actual {
            bail!("the downloaded runtime doesn't match its published SHA-256, so it wasn't used");
        }
        check_bare(&bytes, &partial)?;
        std::fs::rename(&partial, &target)?;
        Ok(())
    })();
    let _ = std::fs::remove_file(&partial);
    let _ = std::fs::remove_file(&checksum);
    result.with_context(|| {
        format!(
            "couldn't get {FILE_NAME} from {url}. Download the Bach {} zip from \
             https://github.com/usebach/bach/releases and keep bach-setup.exe next to bach, \
             or pass --runtime",
            env!("CARGO_PKG_VERSION")
        )
    })?;
    Ok(target)
}

/// Fetches `url` into `to` with curl (built into Windows 10+, macOS and
/// most Linux), or PowerShell on a Windows without it.
fn fetch(url: &str, to: &Path) -> Result<()> {
    let curl = Command::new("curl")
        .args(["--fail", "--location", "--silent", "--show-error", "--retry", "2", "--output"])
        .arg(to)
        .arg(url)
        .status();
    match curl {
        Ok(status) if status.success() => return Ok(()),
        Ok(status) => bail!("curl couldn't download {url} ({status})"),
        Err(_) if cfg!(windows) => {}
        Err(_) => bail!("downloading needs curl, which isn't installed"),
    }
    let script = format!(
        "$ProgressPreference='SilentlyContinue'; Invoke-WebRequest -UseBasicParsing -Uri '{url}' -OutFile '{}'",
        to.display().to_string().replace('\'', "''")
    );
    let status = Command::new("powershell").args(["-NoProfile", "-NonInteractive", "-Command", &script]).status()?;
    if !status.success() {
        bail!("PowerShell couldn't download {url}");
    }
    Ok(())
}

/// The runtime's bytes, checked to be a bare runtime and not a setup file
/// that already carries a payload.
pub fn read(path: &Path) -> Result<Vec<u8>> {
    let bytes = std::fs::read(path).with_context(|| format!("couldn't read {}", path.display()))?;
    check_bare(&bytes, path)?;
    Ok(bytes)
}

fn check_bare(bytes: &[u8], path: &Path) -> Result<()> {
    if !bytes.starts_with(b"MZ") {
        bail!("{} isn't a Windows executable", path.display());
    }
    let mut file = std::fs::File::open(path).with_context(|| format!("couldn't open {}", path.display()))?;
    if bach_pack::container::locate(&mut file)?.is_some() {
        bail!("{} is already an installer; point --runtime at a plain bach-setup.exe", path.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkouts_look_in_both_profiles() {
        let bach = Path::new("repo").join("target").join("debug").join("bach.exe");
        let found = checkout_builds(&bach);
        assert_eq!(found[0], Path::new("repo").join("target").join("release").join(FILE_NAME));
        assert!(checkout_builds(Path::new("tools/bach.exe")).is_empty());
    }

    #[test]
    fn downloads_name_this_version() {
        assert!(url().ends_with(&format!("/v{}/bach-setup.exe", env!("CARGO_PKG_VERSION"))));
    }
}
