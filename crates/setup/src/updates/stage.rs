//! Keeping a downloaded update ready.
//!
//! A newer setup file is downloaded in the background into
//! `%LOCALAPPDATA%\Bach\updates\<app id>\` and stays there until it is
//! installed, so closing the app (or the PC) never loses it. A marker file,
//! `staged.json`, names what is ready. Nothing in this folder is trusted:
//! before the file runs, it is hashed again and read as a setup file for
//! this very app and version (see [`verify`]).
//!
//! A download that was cut off continues where it stopped, and one that
//! fails its hash is thrown away and fetched once more from the start.

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};

use bach_pack::container;
use bach_pack::manifest::FeedEntry;
use serde_json::json;
use sha2::{Digest, Sha256};

use super::curl::{self, Fetch};
use crate::system::host;

/// Room left over after the download, for installing it.
const SPARE_BYTES: u64 = 64 << 20;
const MARKER: &str = "staged.json";

/// An update that has been downloaded and checked, waiting to be installed.
#[derive(Clone, Debug, PartialEq)]
pub struct Staged {
    pub version: String,
    pub sha256: String,
    pub notes: Option<String>,
    /// The setup file.
    pub file: PathBuf,
}

/// Where an app's staged update lives.
pub fn folder(app_id: &str) -> PathBuf {
    let base = std::env::var_os("LOCALAPPDATA").map(PathBuf::from).unwrap_or_else(std::env::temp_dir);
    base.join("Bach").join("updates").join(app_id.replace(' ', "-"))
}

/// The marker's staged update, if one is recorded and its file is there.
/// This doesn't hash the file; [`verify`] does that.
pub fn read(folder: &Path) -> Option<Staged> {
    let text = fs::read_to_string(folder.join(MARKER)).ok()?;
    let value: serde_json::Value = serde_json::from_str(text.trim_start_matches('\u{feff}')).ok()?;
    let text_of = |key: &str| value.get(key).and_then(|v| v.as_str()).map(String::from);
    let file = text_of("file").filter(|name| is_plain_name(name))?;
    let file = folder.join(file);
    file.is_file().then(|| Staged {
        version: text_of("version").unwrap_or_default(),
        sha256: text_of("sha256").unwrap_or_default(),
        notes: text_of("notes"),
        file,
    })
}

/// A file name with nothing in it that could lead out of the folder.
fn is_plain_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-' | '_'))
}

/// Checks a staged file right before it is used: its hash is the one
/// recorded, and it is a setup file for this app, of the recorded version.
pub fn verify(staged: &Staged, app_id: &str) -> Result<(), String> {
    let hash = hash_file(&staged.file).map_err(|err| format!("couldn't read the downloaded update: {err}"))?;
    if !hash.eq_ignore_ascii_case(&staged.sha256) {
        return Err("the downloaded update doesn't match its SHA-256".into());
    }
    inspect(&staged.file, app_id, &staged.version)
}

/// Reads `path` as a setup file and checks it belongs to `app_id` and is
/// `version`, so a wrong file or a feed that names another version never runs.
fn inspect(path: &Path, app_id: &str, version: &str) -> Result<(), String> {
    let wrong = || "the download isn't a setup file for this app".to_string();
    let mut file = File::open(path).map_err(|err| err.to_string())?;
    let layout = container::locate(&mut file).ok().flatten().ok_or_else(wrong)?;
    let manifest = container::read_manifest(&mut file, &layout).map_err(|_| wrong())?;
    if !layout.has_archive() || manifest.app.id != app_id {
        return Err(wrong());
    }
    if bach_pack::version::compare(&manifest.app.version, version) != Some(std::cmp::Ordering::Equal) {
        return Err(format!("the download is version {}, not {version} as the update feed says", manifest.app.version));
    }
    Ok(())
}

fn hash_file(path: &Path) -> io::Result<String> {
    let mut hasher = Sha256::new();
    io::copy(&mut File::open(path)?, &mut hasher)?;
    Ok(format!("{:x}", hasher.finalize()))
}

/// Downloads `entry`'s setup file (unless it is already staged and still
/// good) and records it as ready.
pub fn ensure(entry: &FeedEntry, app_id: &str) -> Result<Staged, String> {
    let dir = folder(app_id);
    fs::create_dir_all(&dir).map_err(|err| format!("couldn't create {}: {err}", dir.display()))?;
    let expected = entry.sha256.trim().to_ascii_lowercase();

    if let Some(staged) = read(&dir) {
        if staged.version == entry.version && staged.sha256 == expected && verify(&staged, app_id).is_ok() {
            return Ok(staged);
        }
    }
    if let Some(size) = entry.size {
        if host::free_space(&dir).is_some_and(|free| free < size.saturating_add(SPARE_BYTES)) {
            return Err("there isn't enough free space on this drive to download the update".into());
        }
    }

    let stem = format!("Setup-{}", safe_part(&entry.version));
    let partial = dir.join(format!("{stem}-{}.partial", expected.get(..12).unwrap_or(&expected)));
    for attempt in 0..2 {
        let how = if attempt == 0 { Fetch::SETUP } else { Fetch { resume: false, ..Fetch::SETUP } };
        if attempt > 0 {
            let _ = fs::remove_file(&partial);
        }
        // A failed download keeps what it got, to continue next time.
        curl::download(&entry.url, &partial, how)?;
        let hash = hash_file(&partial).map_err(|err| err.to_string())?;
        if hash == expected {
            break;
        }
        if attempt > 0 {
            let _ = fs::remove_file(&partial);
            return Err("the download doesn't match the feed's SHA-256".into());
        }
    }
    if let Err(problem) = inspect(&partial, app_id, &entry.version) {
        let _ = fs::remove_file(&partial);
        return Err(problem);
    }

    let file_name = format!("{stem}.exe");
    let file = dir.join(&file_name);
    let _ = fs::remove_file(&file);
    fs::rename(&partial, &file).map_err(|err| err.to_string())?;
    let staged = Staged { version: entry.version.clone(), sha256: expected, notes: entry.notes.clone(), file };
    write_marker(&dir, &staged, &file_name, &entry.url)?;
    prune(&dir, &file_name);
    Ok(staged)
}

/// The marker goes in through a temporary file, so a cut power never leaves
/// half a marker (and a missing marker only means "nothing staged").
fn write_marker(dir: &Path, staged: &Staged, file_name: &str, url: &str) -> Result<(), String> {
    let marker = json!({
        "version": staged.version,
        "sha256": staged.sha256,
        "notes": staged.notes,
        "file": file_name,
        "url": url,
    });
    let text = serde_json::to_string_pretty(&marker).map_err(|err| err.to_string())?;
    let partial = dir.join(format!("{MARKER}.partial"));
    let written = File::create(&partial)
        .and_then(|mut file| {
            io::Write::write_all(&mut file, text.as_bytes())?;
            file.sync_all()
        })
        .and_then(|()| fs::rename(&partial, dir.join(MARKER)));
    written.map_err(|err| {
        let _ = fs::remove_file(&partial);
        format!("couldn't record the downloaded update: {err}")
    })
}

/// A version as part of a file name.
fn safe_part(version: &str) -> String {
    version.chars().filter(|c| c.is_ascii_alphanumeric() || matches!(c, '.' | '-')).collect()
}

/// Deletes everything in the folder but the marker and `keep`.
fn prune(dir: &Path, keep: &str) {
    let Ok(entries) = fs::read_dir(dir) else {
        return;
    };
    for entry in entries.flatten() {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name != keep && name != MARKER {
            let _ = fs::remove_file(entry.path());
        }
    }
}

/// Forgets the staged update and deletes its files.
pub fn clear(app_id: &str) {
    let dir = folder(app_id);
    let _ = fs::remove_file(dir.join(MARKER));
    prune(&dir, "");
    let _ = fs::remove_dir(&dir);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bach-stage-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn a_marker_names_a_file_in_its_folder() {
        let dir = temp("marker");
        fs::write(dir.join("Setup-2.0.exe"), b"x").unwrap();
        let staged = Staged {
            version: "2.0".into(),
            sha256: "ab".repeat(32),
            notes: Some("New things".into()),
            file: dir.join("Setup-2.0.exe"),
        };
        write_marker(&dir, &staged, "Setup-2.0.exe", "https://x/y.exe").unwrap();
        assert_eq!(read(&dir), Some(staged));
        assert!(!dir.join(format!("{MARKER}.partial")).exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_marker_can_not_point_outside_its_folder() {
        let dir = temp("outside");
        for name in ["..\\evil.exe", "C:\\Windows\\notepad.exe", "sub/x.exe", ""] {
            let marker = json!({ "version": "1", "sha256": "x", "file": name });
            fs::write(dir.join(MARKER), marker.to_string()).unwrap();
            assert_eq!(read(&dir), None, "{name:?}");
        }
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn no_marker_means_nothing_staged() {
        let dir = temp("none");
        assert_eq!(read(&dir), None);
        fs::write(dir.join(MARKER), "{ half a marker").unwrap();
        assert_eq!(read(&dir), None);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_file_that_isnt_a_setup_file_fails_verification() {
        let dir = temp("verify");
        let file = dir.join("Setup-1.exe");
        fs::write(&file, b"MZ not a setup file").unwrap();
        let hash = hash_file(&file).unwrap();
        let staged = Staged { version: "1".into(), sha256: hash, notes: None, file: file.clone() };
        assert!(verify(&staged, "Hello").unwrap_err().contains("isn't a setup file"));
        let wrong_hash = Staged { sha256: "00".repeat(32), ..staged };
        assert!(verify(&wrong_hash, "Hello").unwrap_err().contains("SHA-256"));
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn pruning_keeps_the_marker_and_the_named_file() {
        let dir = temp("prune");
        for name in [MARKER, "Setup-2.exe", "Setup-1.exe", "Setup-1-abc.partial"] {
            fs::write(dir.join(name), b"x").unwrap();
        }
        prune(&dir, "Setup-2.exe");
        let mut left: Vec<_> = fs::read_dir(&dir).unwrap().map(|e| e.unwrap().file_name().to_string_lossy().into_owned()).collect();
        left.sort();
        assert_eq!(left, ["Setup-2.exe", MARKER]);
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn versions_make_safe_file_names() {
        assert_eq!(safe_part("2.0.0-beta.1+build/7"), "2.0.0-beta.1build7");
        assert_eq!(safe_part("..\\.."), "....");
    }
}
