//! The work behind the setup window: installing, updating and removing.
//! Each job runs on its own thread (file copies and COM block) and reports
//! progress as it goes; the view only draws what it hears.

mod install;
mod preview;
mod remove;

use std::fs;
use std::io;
use std::path::Path;
use std::time::Duration;

pub use install::{install, InstallPlan};
pub use preview::pretend;
pub use remove::{remove, RemovePlan};

/// What a job tells the view.
pub enum Event {
    /// How far along (0..1) and what it's doing, as a short label.
    Progress(f32, String),
    /// The end: a readable sentence when it failed.
    Finished(Result<(), String>),
}

/// How a job reports progress.
pub type Report<'a> = &'a dyn Fn(f32, &str);

/// Sharing violation or access denied: on a program file, that's a running
/// copy holding it.
fn is_in_use(err: &io::Error) -> bool {
    matches!(err.raw_os_error(), Some(32) | Some(5))
}

fn still_open(name: &str) -> String {
    format!("{name} is still open. Close it, then try again.")
}

fn same_file(a: &Path, b: &Path) -> bool {
    match (fs::canonicalize(a), fs::canonicalize(b)) {
        (Ok(a), Ok(b)) => a == b,
        _ => false,
    }
}

/// Removes a file; one that isn't there is already gone.
fn remove_file_if_present(path: &Path) -> io::Result<()> {
    match fs::remove_file(path) {
        Err(err) if err.kind() != io::ErrorKind::NotFound => Err(err),
        _ => Ok(()),
    }
}

/// Deletes a file, waiting a moment for a program that is just exiting
/// (the app, or the uninstaller that started a temp copy) to let go of it.
fn remove_waiting(path: &Path, name: &str) -> Result<(), String> {
    let mut last = None;
    for _ in 0..20 {
        match remove_file_if_present(path) {
            Ok(()) => return Ok(()),
            Err(err) if is_in_use(&err) => last = Some(err),
            Err(err) => return Err(format!("Couldn't delete {}. {}", path.display(), describe(&err))),
        }
        std::thread::sleep(Duration::from_millis(150));
    }
    match last {
        Some(_) => Err(still_open(name)),
        None => Ok(()),
    }
}

/// Removes the now-empty folders above each removed file, deepest first,
/// stopping at `root` (which stays).
fn remove_empty_parents<'a>(root: &Path, files: impl Iterator<Item = &'a str>) {
    let unique: std::collections::BTreeSet<&str> = files
        .flat_map(|file| file.match_indices('/').map(move |(at, _)| &file[..at]))
        .collect();
    let mut folders: Vec<&str> = unique.into_iter().collect();
    folders.sort_by_key(|folder| std::cmp::Reverse(folder.matches('/').count()));
    for folder in folders {
        // Fails while the folder still holds something; that's the point.
        let _ = fs::remove_dir(bach_pack::paths::under(root, folder));
    }
}

/// An OS error as a sentence, without the "(os error 5)" tail.
fn describe(err: &io::Error) -> String {
    let text = err.to_string();
    let text = match text.rfind(" (os error") {
        Some(at) => text[..at].trim_end().to_string(),
        None => text,
    };
    sentence(text)
}

fn plain(err: &anyhow::Error) -> String {
    sentence(err.to_string())
}

fn sentence(text: String) -> String {
    let text = text.trim().to_string();
    if text.ends_with(['.', '!', '?']) {
        text
    } else {
        format!("{text}.")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn os_errors_read_as_sentences() {
        let text = describe(&io::Error::from_raw_os_error(5));
        assert!(!text.contains("os error"), "{text}");
        assert!(text.ends_with('.'));
    }

    #[test]
    fn in_use_errors_are_recognised() {
        assert!(is_in_use(&io::Error::from_raw_os_error(32)));
        assert!(!is_in_use(&io::Error::from_raw_os_error(2)));
    }

    #[test]
    fn empty_parents_go_and_the_root_stays() {
        let root = std::env::temp_dir().join(format!("bach-setup-parents-{}", std::process::id()));
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(root.join("a/b")).unwrap();
        fs::create_dir_all(root.join("keep")).unwrap();
        fs::write(root.join("keep/mine.txt"), b"user file").unwrap();
        remove_empty_parents(&root, ["a/b/x.dll", "keep/y.dll"].into_iter());
        assert!(!root.join("a").exists());
        assert!(root.join("keep/mine.txt").exists());
        assert!(root.exists());
        fs::remove_dir_all(&root).unwrap();
    }
}
