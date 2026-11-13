//! Removing an install.
//!
//! The program goes first: if the app is still open that fails before
//! anything else has changed. Then every file the manifest lists, the
//! folders they leave empty, the shortcuts, the Apps entry, the user's data
//! only when asked, and the install folder if nothing else is left in it.
//! Files the app created itself in the install folder stay, so a user's own
//! files there are never touched.

use std::fs;
use std::path::Path;
use std::sync::Arc;

use bach_pack::manifest::{Manifest, UNINSTALLER};
use bach_pack::paths;

use super::{describe, plain, remove_empty_parents, remove_file_if_present, remove_waiting, same_file, Report};
use crate::package::Package;
use crate::system;

pub struct RemovePlan {
    pub package: Arc<Package>,
    pub dir: std::path::PathBuf,
    pub delete_data: bool,
}

pub fn remove(plan: &RemovePlan, report: Report) -> Result<(), String> {
    let manifest = &plan.package.manifest;
    let name = manifest.app.name.as_str();
    let dir = &plan.dir;
    check_install_dir(dir, manifest)?;
    let uninstaller = dir.join(UNINSTALLER);
    let current = std::env::current_exe().ok();
    let running_in_place = current.as_deref().is_some_and(|current| same_file(current, &uninstaller));

    report(0.1, &format!("Removing {name}"));
    remove_waiting(&manifest.executable_in(dir), name)?;
    for file in &manifest.files {
        let path = paths::under(dir, &file.path);
        remove_file_if_present(&path).map_err(|err| format!("Couldn't delete {}. {}", path.display(), describe(&err)))?;
    }
    remove_empty_parents(dir, manifest.files.iter().map(|file| file.path.as_str()));

    report(0.45, "Removing shortcuts");
    for shortcut in [system::start_menu_shortcut(manifest), system::desktop_shortcut(manifest)]
        .into_iter()
        .flatten()
    {
        let _ = remove_file_if_present(&shortcut);
    }

    report(0.6, &format!("Removing {name} from your apps"));
    system::unregister(manifest).map_err(|err| format!("Couldn't take {name} off Settings › Apps. {}", plain(&err)))?;

    if plan.delete_data {
        report(0.75, "Deleting your data");
        delete_data(manifest)?;
    }

    report(0.9, "Cleaning up");
    if !running_in_place {
        // The installed uninstaller handed over to this temp copy and is
        // exiting; give it a moment to let go of its file.
        remove_waiting(&uninstaller, &format!("The {name} uninstaller"))?;
    }
    // Only an empty folder goes: the user may have kept their own files there.
    let _ = fs::remove_dir(dir);
    system::notify_shell();

    // This process's own file can't go while it runs: a hidden helper waits
    // for it to exit and deletes it then.
    if let Some(current) = current {
        if running_in_place || is_temporary_copy(&current) {
            let folder = running_in_place.then_some(dir.as_path());
            let _ = delete_after_exit(&current, folder);
        }
    }
    report(1.0, "Done");
    Ok(())
}

/// Refuses a folder that isn't plainly this app's install, so a wrong
/// `--install-dir` can never send the uninstaller anywhere else.
fn check_install_dir(dir: &Path, manifest: &Manifest) -> Result<(), String> {
    let registered = system::installed(manifest).is_some_and(|installed| same_file(&installed.dir, dir));
    let looks_installed = dir.join(UNINSTALLER).is_file() || manifest.executable_in(dir).is_file();
    if dir.is_absolute() && dir.is_dir() && (registered || looks_installed) {
        Ok(())
    } else {
        Err(format!("Couldn't find {} in {}. It may already be removed.", manifest.app.name, dir.display()))
    }
}

/// The folders the app keeps its data in (checked when the setup file was
/// built and again when it was read, so they're inside the user's profile).
fn delete_data(manifest: &Manifest) -> Result<(), String> {
    for template in &manifest.uninstall.data {
        let Some(folder) = paths::expand_data_folder(template, |name| std::env::var(name).ok()) else {
            continue;
        };
        match fs::remove_dir_all(&folder) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => return Err(format!("Couldn't delete your data in {}. {}", folder.display(), describe(&err))),
        }
    }
    Ok(())
}

/// The uninstaller's own copy in the temp folder (see launch.rs).
fn is_temporary_copy(path: &Path) -> bool {
    let temp = std::env::temp_dir();
    path.parent().is_some_and(|parent| same_file(parent, &temp))
}

/// Starts a hidden `cmd` that retries deleting `file` (and then removing
/// `folder`, if empty) for up to two minutes, until this process has exited
/// and let go of it.
fn delete_after_exit(file: &Path, folder: Option<&Path>) -> std::io::Result<()> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;

    let file = file.display();
    let cleanup = match folder {
        Some(folder) => format!("rmdir \"{}\" >nul 2>&1 & ", folder.display()),
        None => String::new(),
    };
    // cmd reads everything after `if (…)` on the same line as part of the
    // if, so the wait sits in the if and the way out in the else.
    let script = format!(
        "/d /q /c \"for /l %i in (1,1,120) do @(del /f /q \"{file}\" >nul 2>&1 & \
         if exist \"{file}\" (ping -n 2 127.0.0.1 >nul) else ({cleanup}exit))\""
    );
    std::process::Command::new("cmd.exe")
        .raw_arg(script)
        .creation_flags(CREATE_NO_WINDOW)
        .spawn()
        .map(|_| ())
}
