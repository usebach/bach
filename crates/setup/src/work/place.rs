//! Putting files in place all or nothing.
//!
//! Every file is first unpacked next to its target as `<name>.bach-new`.
//! Only when all of them are on disk are they moved into place; a file
//! being replaced steps aside as `<name>.bach-old` first. If a move fails
//! (a file in use), every move so far is undone, so an update either
//! completes or leaves the earlier version exactly as it was. A full disk
//! or a damaged archive fails while unpacking, before anything changed.
//!
//! A power cut or a crash can stop the moves halfway, where no code gets a
//! chance to undo them. So a small journal in the install folder lists what
//! is about to move, and [`recover`] (run at the start of every install,
//! update and removal) rolls a halfway update back from it. New files are
//! flushed to disk before the journal and the old copies go, so a power cut
//! right after an update can't leave empty files behind a finished-looking
//! install.

use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Path, PathBuf};

use bach_pack::archive;

const NEW: &str = ".bach-new";
const OLD: &str = ".bach-old";
/// The journal's file name in the install folder.
const JOURNAL: &str = ".bach-update";

/// Where the journal for an install in `dir` lives.
pub fn journal_path(dir: &Path) -> PathBuf {
    dir.join(JOURNAL)
}

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
        // No flush to disk per file: on an app with thousands of files that
        // alone takes most of the install. The old files stay until every
        // new one is written, and a run cut short by a power loss is undone
        // by running setup again, which clears what it left.
        let written = (|| -> io::Result<()> {
            let mut out = io::BufWriter::with_capacity(256 * 1024, File::create(&partial)?);
            archive::copy_counting(body, &mut out, progress)?;
            out.into_inner().map_err(|err| err.into_error())?;
            Ok(())
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

    /// Moves every unpacked file into place, or none. `journal` is where the
    /// list of moves is kept until they're done and on disk.
    pub fn commit(self, journal: &Path) -> Result<(), CommitError> {
        if let Err(error) = write_journal(journal, &self.staged) {
            self.discard();
            let _ = fs::remove_file(journal);
            return Err(CommitError { target: journal.to_path_buf(), error });
        }
        let mut done: Vec<(PathBuf, Option<PathBuf>)> = Vec::with_capacity(self.staged.len());
        for (index, target) in self.staged.iter().enumerate() {
            match swap_in(target) {
                Ok(backup) => done.push((target.clone(), backup)),
                Err(error) => {
                    undo(&done);
                    for rest in &self.staged[index..] {
                        let _ = fs::remove_file(with_suffix(rest, NEW));
                    }
                    let _ = fs::remove_file(journal);
                    return Err(CommitError { target: target.clone(), error });
                }
            }
        }
        // The new files must be on disk before the old copies and the
        // journal go: after a power cut, the journal is what undoes this.
        flush(&self.staged);
        let _ = fs::remove_file(journal);
        // Old copies a running program still holds stay until next time.
        for (_, backup) in done {
            if let Some(backup) = backup {
                let _ = fs::remove_file(backup);
            }
        }
        Ok(())
    }
}

/// Lists every move, one per line: `R<path>` for a file that replaces
/// another, `N<path>` for one that's new. Written and flushed before the
/// first move, so it's there whenever a move might be half done.
fn write_journal(journal: &Path, staged: &[PathBuf]) -> io::Result<()> {
    let mut text = String::new();
    for target in staged {
        text.push(if fs::symlink_metadata(target).is_ok() { 'R' } else { 'N' });
        text.push_str(&target.to_string_lossy());
        text.push('\n');
    }
    let mut file = File::create(journal)?;
    io::Write::write_all(&mut file, text.as_bytes())?;
    file.sync_all()
}

/// Flushes the files to disk, a few at a time: one by one it takes seconds
/// per thousand files.
fn flush(targets: &[PathBuf]) {
    let threads = std::thread::available_parallelism().map_or(4, |count| count.get()).clamp(1, 8);
    let chunk = targets.len().div_ceil(threads).max(1);
    std::thread::scope(|scope| {
        for part in targets.chunks(chunk) {
            scope.spawn(move || {
                for target in part {
                    if let Ok(file) = fs::OpenOptions::new().write(true).open(target) {
                        let _ = file.sync_all();
                    }
                }
            });
        }
    });
}

/// Rolls back an update that stopped halfway (a power cut, a crash, the
/// process killed), using the journal it left in `dir`. Does nothing when
/// there is none. Safe to run again: a second run finds nothing to undo.
/// Returns how many files it put back or took away.
pub fn recover(dir: &Path) -> usize {
    let journal = journal_path(dir);
    let Ok(text) = fs::read_to_string(&journal) else {
        return 0;
    };
    let mut changed = 0;
    for line in text.lines().rev() {
        let (kind, path) = line.split_at(line.chars().next().map_or(0, char::len_utf8));
        let target = PathBuf::from(path);
        // The journal is a plain file in the install folder: only ever act
        // on paths inside it.
        let inside = target.starts_with(dir)
            && target.components().all(|part| !matches!(part, std::path::Component::ParentDir));
        if path.is_empty() || !inside {
            continue;
        }
        let (old, new) = (with_suffix(&target, OLD), with_suffix(&target, NEW));
        if fs::symlink_metadata(&old).is_ok() {
            // A file that was replaced: the old one is still here.
            let _ = fs::remove_file(&target);
            if fs::rename(&old, &target).is_ok() {
                changed += 1;
            }
            let _ = fs::remove_file(&new);
        } else if fs::symlink_metadata(&new).is_ok() {
            // Not moved yet.
            let _ = fs::remove_file(&new);
        } else if kind == "N" && fs::remove_file(&target).is_ok() {
            // A new file that made it in, and has nothing to go back to.
            changed += 1;
        }
    }
    let _ = fs::remove_file(&journal);
    changed
}

/// Moves `from` over `to`, also when `to` is a program that is running (the
/// uninstaller running an update, say): Windows won't overwrite a running
/// program but lets it be renamed, so it steps aside as `<to>.bach-old`,
/// which goes now or on the next run.
pub fn replace_file(from: &Path, to: &Path) -> io::Result<()> {
    match fs::rename(from, to) {
        Ok(()) => Ok(()),
        Err(first) => {
            if fs::symlink_metadata(to).is_err() {
                return Err(first);
            }
            let backup = with_suffix(to, OLD);
            let _ = fs::remove_file(&backup);
            fs::rename(to, &backup).map_err(|_| first)?;
            if let Err(err) = fs::rename(from, to) {
                let _ = fs::rename(&backup, to);
                return Err(err);
            }
            let _ = fs::remove_file(&backup);
            Ok(())
        }
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
        assert!(transaction.commit(&journal_path(&dir)).is_ok());
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
        let err = transaction.commit(&journal_path(&dir)).expect_err("b.txt is held");
        drop(holder);
        assert_eq!(err.target, blocked);
        assert_eq!(fs::read(&first).unwrap(), b"old a", "a.txt is rolled back");
        assert_eq!(fs::read(&blocked).unwrap(), b"old b");
        let leftovers: Vec<_> = fs::read_dir(&dir).unwrap().map(|entry| entry.unwrap().file_name()).collect();
        assert_eq!(leftovers.len(), 2, "no .bach-new, .bach-old or journal left: {leftovers:?}");
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

    #[test]
    fn a_finished_update_leaves_no_journal() {
        let dir = temp("journal-gone");
        let target = dir.join("a.txt");
        fs::write(&target, b"old").unwrap();
        let mut transaction = Transaction::default();
        transaction.stage(&target, &mut &b"new"[..], &mut |_| {}).unwrap();
        transaction.commit(&journal_path(&dir)).unwrap();
        assert!(!journal_path(&dir).exists());
        assert_eq!(recover(&dir), 0, "nothing to undo");
        assert_eq!(fs::read(&target).unwrap(), b"new");
        fs::remove_dir_all(&dir).unwrap();
    }

    /// Plays the moves of a commit up to a power cut after the first file,
    /// then lets `recover` undo them.
    #[test]
    fn a_halfway_update_is_rolled_back() {
        let dir = temp("recover");
        let (a, b, fresh) = (dir.join("a.txt"), dir.join("b.txt"), dir.join("fresh.txt"));
        fs::write(&a, b"old a").unwrap();
        fs::write(&b, b"old b").unwrap();
        let mut transaction = Transaction::default();
        for (file, body) in [(&a, "new a"), (&b, "new b"), (&fresh, "new fresh")] {
            transaction.stage(file, &mut body.as_bytes(), &mut |_| {}).unwrap();
        }
        write_journal(&journal_path(&dir), &transaction.staged).unwrap();
        swap_in(&a).unwrap();
        swap_in(&fresh).unwrap();
        // ...power cut: b.txt wasn't reached, a.txt and fresh.txt were.
        assert_eq!(fs::read(&a).unwrap(), b"new a");

        assert!(recover(&dir) >= 2);
        assert_eq!(fs::read(&a).unwrap(), b"old a");
        assert_eq!(fs::read(&b).unwrap(), b"old b");
        assert!(!fresh.exists(), "a file the update added goes again");
        let left: Vec<_> = fs::read_dir(&dir).unwrap().map(|entry| entry.unwrap().file_name()).collect();
        assert_eq!(left.len(), 2, "only the two old files remain: {left:?}");
        assert_eq!(recover(&dir), 0, "a second run changes nothing");
        assert_eq!(fs::read(&a).unwrap(), b"old a");
        fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn a_journal_can_not_point_outside_the_folder() {
        let dir = temp("recover-outside");
        let outside = temp("recover-victim");
        let victim = outside.join("keep.txt");
        fs::write(&victim, b"mine").unwrap();
        fs::write(with_suffix(&victim, OLD), b"x").unwrap();
        fs::write(journal_path(&dir), format!("R{}\n", victim.display())).unwrap();
        assert_eq!(recover(&dir), 0);
        assert_eq!(fs::read(&victim).unwrap(), b"mine");
        fs::remove_dir_all(&dir).unwrap();
        fs::remove_dir_all(&outside).unwrap();
    }

    #[test]
    fn a_running_program_can_be_replaced() {
        let dir = temp("replace-running");
        let target = dir.join("uninstall.exe");
        let ping = Path::new(&std::env::var_os("SystemRoot").unwrap()).join("System32").join("ping.exe");
        fs::copy(&ping, &target).unwrap();
        let mut running = std::process::Command::new(&target).args(["-n", "30", "127.0.0.1"]).spawn();
        assert!(running.is_ok(), "the stand-in program starts");
        std::thread::sleep(std::time::Duration::from_millis(300));
        let fresh = dir.join("uninstall.exe.bach-new");
        fs::write(&fresh, b"new").unwrap();
        replace_file(&fresh, &target).unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"new");
        if let Ok(child) = &mut running {
            let _ = child.kill();
            let _ = child.wait();
        }
        let _ = fs::remove_dir_all(&dir);
    }
}
