//! Installing and updating.
//!
//! 1. Unpack every file into the install folder. Each goes through a
//!    `.bach-new` file and is renamed into place, so an interrupted copy
//!    never leaves a half-written program behind.
//! 2. On an update, remove the files the old version had and this one
//!    doesn't. The old file list comes from the old uninstaller.
//! 3. Write `uninstall.exe`: this runtime, with the manifest and no files.
//! 4. Add the shortcuts and the entry under Settings › Apps.

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bach_pack::archive;
use bach_pack::container;
use bach_pack::manifest::{Manifest, UNINSTALLER};
use bach_pack::paths;

use super::{describe, is_in_use, plain, remove_empty_parents, remove_file_if_present, still_open, Report};
use crate::package::Package;
use crate::system;

/// Unpacking is most of the work; the steps after it share the rest.
const UNPACK_SHARE: f32 = 0.82;

pub struct InstallPlan {
    pub package: Arc<Package>,
    pub dir: PathBuf,
    pub desktop_shortcut: bool,
}

pub fn install(plan: &InstallPlan, report: Report) -> Result<(), String> {
    let package = &plan.package;
    let manifest = &package.manifest;
    let name = manifest.app.name.as_str();
    let dir = &plan.dir;
    let Some(layout) = package.layout.filter(|layout| layout.has_archive()) else {
        return Err("This setup file carries no files to install.".into());
    };

    report(0.02, "Getting ready");
    fs::create_dir_all(dir).map_err(|err| format!("Couldn't create {}. {}", dir.display(), describe(&err)))?;
    let program = manifest.executable_in(dir);
    ensure_closed(&program, name)?;
    let previous = previous_files(dir);

    let copying = format!("Copying {name}");
    let total = manifest.total_size().max(1);
    let mut done = 0u64;
    let mut source = File::open(&package.exe).map_err(|err| format!("Couldn't read this setup file. {}", describe(&err)))?;
    source
        .seek(SeekFrom::Start(layout.archive.offset))
        .map_err(|err| format!("Couldn't read this setup file. {}", describe(&err)))?;
    let reader = BufReader::new(source.take(layout.archive.len));
    archive::read(reader, &mut |packaged, _size, body| {
        let target = paths::under(dir, packaged);
        place(&target, body, &mut |chunk| {
            done = done.saturating_add(chunk);
            report(0.04 + UNPACK_SHARE * (done as f32 / total as f32).min(1.0), &copying);
        })
        .map_err(|message| anyhow::anyhow!(message))
    })
    .map_err(|err| {
        let message = plain(&err);
        if message.contains("still open") {
            message
        } else {
            format!("Couldn't unpack {name}. {message}")
        }
    })?;

    report(0.88, "Tidying up");
    if let Some(previous) = previous {
        remove_stale(dir, &previous, manifest);
    }

    report(0.9, "Adding the uninstaller");
    write_uninstaller(package, dir)?;

    report(0.93, "Adding shortcuts");
    let description = manifest.app.description.clone().unwrap_or_else(|| name.to_string());
    let start_menu = system::start_menu_shortcut(manifest).map_err(|err| format!("Couldn't find the Start menu. {}", plain(&err)))?;
    system::create_shortcut(&start_menu, &program, &description)
        .map_err(|err| format!("Couldn't add {name} to the Start menu. {}", plain(&err)))?;
    if let Ok(desktop) = system::desktop_shortcut(manifest) {
        if plan.desktop_shortcut {
            system::create_shortcut(&desktop, &program, &description)
                .map_err(|err| format!("Couldn't add the desktop shortcut. {}", plain(&err)))?;
        } else {
            // Unticked on an update: the old one goes too.
            let _ = remove_file_if_present(&desktop);
        }
    }

    report(0.97, &format!("Adding {name} to your apps"));
    system::register(manifest, dir).map_err(|err| format!("Couldn't list {name} under Settings › Apps. {}", plain(&err)))?;
    system::notify_shell();
    report(1.0, "Done");
    Ok(())
}

/// Writes one file through `<target>.bach-new` and renames it into place.
fn place(target: &Path, body: &mut dyn Read, progress: &mut dyn FnMut(u64)) -> Result<(), String> {
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent).map_err(|err| format!("Couldn't create {}. {}", parent.display(), describe(&err)))?;
    }
    let partial = with_suffix(target, ".bach-new");
    let written = (|| -> std::io::Result<()> {
        let mut out = File::create(&partial)?;
        archive::copy_counting(body, &mut out, progress)?;
        out.sync_all()
    })();
    if let Err(err) = written {
        let _ = fs::remove_file(&partial);
        return Err(format!("Couldn't write {}. {}", target.display(), describe(&err)));
    }
    fs::rename(&partial, target).map_err(|err| {
        let _ = fs::remove_file(&partial);
        if is_in_use(&err) {
            format!("{} is in use, so it can't be replaced. Close the app, then try again.", target.display())
        } else {
            format!("Couldn't put {} in place. {}", target.display(), describe(&err))
        }
    })
}

fn with_suffix(path: &Path, suffix: &str) -> PathBuf {
    let mut name = path.as_os_str().to_owned();
    name.push(suffix);
    PathBuf::from(name)
}

/// Fails with a readable sentence when the app is running from `program`.
fn ensure_closed(program: &Path, name: &str) -> Result<(), String> {
    if !program.exists() {
        return Ok(());
    }
    match fs::OpenOptions::new().write(true).open(program) {
        Err(err) if is_in_use(&err) => Err(still_open(name)),
        _ => Ok(()),
    }
}

/// The files the install already in `dir` put there, read from its
/// uninstaller. None for a fresh folder (or an uninstaller Bach can't read,
/// in which case nothing is cleaned up rather than guessed).
fn previous_files(dir: &Path) -> Option<Manifest> {
    let mut file = File::open(dir.join(UNINSTALLER)).ok()?;
    let layout = container::locate(&mut file).ok()??;
    container::read_manifest(&mut file, &layout).ok()
}

/// Removes what the old version installed and the new one doesn't ship.
fn remove_stale(dir: &Path, previous: &Manifest, current: &Manifest) {
    let keep: HashSet<String> = current.files.iter().map(|file| file.path.to_lowercase()).collect();
    let stale: Vec<&str> = previous
        .files
        .iter()
        .map(|file| file.path.as_str())
        .filter(|path| !keep.contains(&path.to_lowercase()))
        .collect();
    for path in &stale {
        // A stale file that won't go is left behind, not a failed update.
        let _ = remove_file_if_present(&paths::under(dir, path));
    }
    remove_empty_parents(dir, stale.into_iter());
}

/// `uninstall.exe`: the runtime's own bytes with the manifest and the logo,
/// and no files.
fn write_uninstaller(package: &Package, dir: &Path) -> Result<(), String> {
    let fail = |err: &dyn std::fmt::Display| format!("Couldn't add the uninstaller. {}", super::sentence(err.to_string()));
    let layout = package.layout.ok_or_else(|| fail(&"this setup file has no payload"))?;
    let mut source = File::open(&package.exe).map_err(|err| fail(&describe(&err)))?;
    let mut runtime = vec![0u8; layout.runtime_len as usize];
    source.read_exact(&mut runtime).map_err(|err| fail(&describe(&err)))?;
    let logo = container::read_logo(&mut source, &layout).map_err(|err| fail(&err))?;

    let target = dir.join(UNINSTALLER);
    let partial = with_suffix(&target, ".bach-new");
    let written = File::create(&partial)
        .map_err(anyhow::Error::from)
        .and_then(|mut out| container::write_uninstaller(&mut out, &runtime, &logo, &package.manifest).map(|_| out))
        .and_then(|out| out.sync_all().map_err(anyhow::Error::from));
    if let Err(err) = written {
        let _ = fs::remove_file(&partial);
        return Err(fail(&err));
    }
    fs::rename(&partial, &target).map_err(|err| {
        let _ = fs::remove_file(&partial);
        if is_in_use(&err) {
            format!("The {} uninstaller is open. Close it, then try again.", package.manifest.app.name)
        } else {
            fail(&describe(&err))
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn files_are_placed_whole() {
        let dir = std::env::temp_dir().join(format!("bach-setup-place-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        let target = dir.join("sub").join("file.bin");
        let mut total = 0;
        place(&target, &mut &b"payload"[..], &mut |n| total += n).unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"payload");
        assert_eq!(total, 7);
        assert!(!with_suffix(&target, ".bach-new").exists());
        // Replacing an existing file works too.
        place(&target, &mut &b"v2"[..], &mut |_| {}).unwrap();
        assert_eq!(fs::read(&target).unwrap(), b"v2");
        fs::remove_dir_all(&dir).unwrap();
    }
}
