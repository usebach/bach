//! Putting files in place all or nothing.
//!
//! Every file is first unpacked next to its target as `<name>.bach-new`.
//! Only when all of them are on disk are they moved into place; a file
//! being replaced steps aside as `<name>.bach-old` first. If a move fails
//! (a file in use), every move so far is undone, so an update either
//! completes or leaves the earlier version exactly as it was. A full disk
//! or a damaged archive fails while unpacking, before anything changed.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use bach_pack::archive;

const NEW: &str = ".bach-new";
const OLD: &str = ".bach-old";

pub fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// Removes what an interrupted earlier run may have left next to `target`.
pub fn clear_leftovers(target: &Path) {
    let _ = fs::remove_file(with_suffix(target, NEW));
    let _ = fs::remove_file(with_suffix(target, OLD));
}

/// Files unpacked and waiting to be moved into place.
#[derive(Default)]
pub struct Transaction {
    staged: Vec<PathBuf>,
}

/// Why a move into place failed, and on which file.
#[derive(Debug)]
pub struct CommitError {
    pub target: PathBuf,
    pub error: io::Error,
}

impl Transaction {
    /// Unpacks one file next to `target`.
    pub fn stage(&mut self, target: &Path, body: &mut dyn Read, progress: &mut dyn FnMut(u64)) -> io::Result<()> {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        let partial = with_suffix(target, NEW);
        let written = (|| -> io::Result<()> {
            let mut out = File::create(&partial)?;
            archive::copy_counting(body, &mut out, progress)?;
            out.sync_all()
        })();
        match written {
            Ok(()) => {
                self.staged.push(target.to_path_buf());
                Ok(())
            }
            Err(err) => {
                let _ = fs::remove_file(&partial);
                Err(err)
            }
        }
    }

    /// Deletes every unpacked file; nothing was moved yet.
    pub fn discard(self) {
        for target in &self.staged {
            let _ = fs::remove_file(with_suffix(target, NEW));
        }
    }

    /// Moves every unpacked file into place, or none.
    pub fn commit(self) -> Result<(), CommitError> {
        let mut done: Vec<(PathBuf, Option<PathBuf>)> = Vec::with_capacity(self.staged.len());
        for (index, target) in self.staged.iter().enumerate() {
            match swap_in(target) {
                Ok(backup) => done.push((target.clone(), backup)),
                Err(error) => {
                    undo(&done);
                    for rest in &self.staged[index..] {
                        let _ = fs::remove_file(with_suffix(rest, NEW));
                    }
                    return Err(CommitError { target: target.clone(), error });
                }
            }
        }
        // Old copies a running program still holds stay until next time.
        for (_, backup) in done {
            if let Some(backup) = backup {
                let _ = fs::remove_file(backup);
            }
        }
        Ok(())
    }
}

/// Moves `<target>.bach-new` to `target`, keeping the file it replaces as
/// `<target>.bach-old`. On failure the target is as it was.
fn swap_in(target: &Path) -> io::Result<Option<PathBuf>> {
    let staged = with_suffix(target, NEW);
    let backup = if fs::symlink_metadata(target).is_ok() {
        let backup = with_suffix(target, OLD);
        let _ = fs::remove_file(&backup);
        fs::rename(target, &backup)?;
        Some(backup)
    } else {
        None
    };
    if let Err(err) = fs::rename(&staged, target) {
        if let Some(backup) = &backup {
            let _ = fs::rename(backup, target);
        }
        return Err(err);
    }
    Ok(backup)
}

/// Puts back what [`swap_in`] replaced, newest first.
fn undo(done: &[(PathBuf, Option<PathBuf>)]) {
    for (target, backup) in done.iter().rev() {
        let _ = fs::remove_file(target);
        if let Some(backup) = backup {
            let _ = fs::rename(backup, target);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn temp(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bach-place-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn files_are_placed_whole() {
        let dir = temp("whole");
        let target = dir.join("sub").join("file.bin");
        fs::create_dir_all(dir.join("sub")).unwrap();
        fs::write(&target, b"v1").unwrap();
        let mut total = 0;
        let mut transaction = Transaction::default();
        transaction.stage(&target, &mut &b"payload"[..], &mut |n| total += n).unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"v1", "nothing moves before the commit");
        assert!(transaction.commit().is_ok());
        assert_eq!(fs::read(&target).unwrap(), b"payload");
        assert_eq!(total, 7);
        assert!(!with_suffix(&target, NEW).exists() && !with_suffix(&target, OLD).exists());
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_failed_move_puts_everything_back() {
        let dir = temp("rollback");
        let first = dir.join("a.txt");
        let blocked = dir.join("b.txt");
        fs::write(&first, b"old a").unwrap();
        fs::write(&blocked, b"old b").unwrap();
        let mut transaction = Transaction::default();
        transaction.stage(&first, &mut &b"new a"[..], &mut |_| {}).unwrap();
        transaction.stage(&blocked, &mut &b"new b"[..], &mut |_| {}).unwrap();
        // Something holds b.txt open without sharing, like a running app.
        let holder = {
            use std::os::windows::fs::OpenOptionsExt;
            fs::OpenOptions::new().read(true).share_mode(0).open(&blocked).unwrap()
        };
        let err = transaction.commit().expect_err("b.txt is held");
        drop(holder);
        assert_eq!(err.target, blocked);
        assert_eq!(fs::read(&first).unwrap(), b"old a", "a.txt is rolled back");
        assert_eq!(fs::read(&blocked).unwrap(), b"old b");
        let leftovers: Vec<_> = fs::read_dir(&dir).unwrap().map(|entry| entry.unwrap().file_name()).collect();
        assert_eq!(leftovers.len(), 2, "no .bach-new or .bach-old left: {leftovers:?}");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn discarding_leaves_the_old_files() {
        let dir = temp("discard");
        let target = dir.join("a.txt");
        fs::write(&target, b"old").unwrap();
        let mut transaction = Transaction::default();
        transaction.stage(&target, &mut &b"new"[..], &mut |_| {}).unwrap();
        transaction.discard();
        assert_eq!(fs::read(&target).unwrap(), b"old");
        assert!(!with_suffix(&target, NEW).exists());
        fs::remove_dir_all(&dir).unwrap();
    }
}
