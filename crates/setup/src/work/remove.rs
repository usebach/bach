//! Removing an install.
//!
//! The program goes first: if the app is still open that fails before
//! anything else has changed. Then every file the manifest lists, the
//! folders they leave empty, the shortcuts, start with Windows, PATH, link
//! schemes and file types, the Apps entry, the user's data only when asked,
//! and the install folder if nothing else is left in it. Files the app
//! created itself in the install folder stay, so a user's own files there
//! are never touched.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bach_pack::i18n::Key;
use bach_pack::manifest::{Choice, Manifest, UNINSTALLER};
use bach_pack::paths;

use super::{describe, failed, plain, remove_empty_parents, remove_file_if_present, remove_waiting, same_file, Report};
use crate::package::Package;
use crate::system::{self, env_path, host, integration};
use crate::texts::Texts;

pub struct RemovePlan {
    pub package: Arc<Package>,
    pub texts: Arc<Texts>,
    pub dir: PathBuf,
    pub delete_data: bool,
}

pub fn remove(plan: &RemovePlan, report: Report) -> Result<(), String> {
    let manifest = &plan.package.manifest;
    let texts = &plan.texts;
    let dir = &plan.dir;
    check_install_dir(dir, manifest, texts)?;
    let _lock = host::SetupLock::acquire(&manifest.app.id).ok_or_else(|| texts.get(Key::error_busy))?;
    let uninstaller = dir.join(UNINSTALLER);
    let current = std::env::current_exe().ok();
    let running_in_place = current.as_deref().is_some_and(|current| same_file(current, &uninstaller));

    report(0.1, &texts.get(Key::step_removing));
    let program = manifest.executable_in(dir);
    remove_waiting(&program, texts, Key::error_still_open)?;
    for file in &manifest.files {
        let path = paths::under(dir, &file.path);
        remove_file_if_present(&path).map_err(|err| failed(texts, Key::error_delete, &path, &err))?;
    }
    remove_empty_parents(dir, manifest.files.iter().map(|file| file.path.as_str()));

    report(0.4, &texts.get(Key::step_removing_shortcuts));
    let extra = system::extra_shortcuts(manifest).into_iter().map(|(link, _)| link);
    for shortcut in [system::start_menu_shortcut(manifest), system::desktop_shortcut(manifest)]
        .into_iter()
        .flatten()
        .chain(extra)
    {
        let _ = remove_file_if_present(&shortcut);
    }
    if manifest.install.start_menu_folder.is_some() {
        // Only an empty folder goes: another app may share it.
        if let Ok(folder) = system::start_menu_folder(manifest) {
            let _ = fs::remove_dir(folder);
        }
    }
    undo_integration(manifest, dir, &program, texts)?;

    report(0.6, &texts.get(Key::step_unregistering));
    system::unregister(manifest).map_err(|err| format!("{} {}", texts.get(Key::error_unregister), plain(&err)))?;

    if plan.delete_data {
        report(0.75, &texts.get(Key::step_deleting_data));
        delete_data(manifest, texts)?;
    }

    report(0.9, &texts.get(Key::step_cleaning));
    if !running_in_place {
        // The installed uninstaller handed over to this temp copy and is
        // exiting; give it a moment to let go of its file.
        remove_waiting(&uninstaller, texts, Key::error_busy)?;
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
    report(1.0, &texts.get(Key::step_done));
    Ok(())
}

/// Takes back start with Windows, the PATH entry, link schemes and file
/// types; each only if it's this install's own.
fn undo_integration(manifest: &Manifest, dir: &Path, program: &Path, texts: &Texts) -> Result<(), String> {
    let fail = |err: anyhow::Error| format!("{} {}", texts.get(Key::error_system), plain(&err));
    integration::clear_autostart(manifest, program).map_err(fail)?;
    let added_to_path = match &manifest.chosen {
        Some(chosen) => chosen.add_to_path,
        None => manifest.install.add_to_path == Choice::Always,
    };
    if added_to_path {
        env_path::remove(&manifest.path_folder_in(dir)).map_err(fail)?;
    }
    for protocol in &manifest.protocols {
        integration::unregister_protocol(manifest, protocol).map_err(fail)?;
    }
    for file_type in &manifest.file_types {
        integration::unregister_file_type(manifest, file_type).map_err(fail)?;
    }
    Ok(())
}

/// Refuses a folder that isn't plainly this app's install, so a wrong
/// `--install-dir` can never send the uninstaller anywhere else.
fn check_install_dir(dir: &Path, manifest: &Manifest, texts: &Texts) -> Result<(), String> {
    let registered = system::installed(manifest).is_some_and(|installed| same_file(&installed.dir, dir));
    let looks_installed = dir.join(UNINSTALLER).is_file() || manifest.executable_in(dir).is_file();
    if dir.is_absolute() && dir.is_dir() && (registered || looks_installed) {
        Ok(())
    } else {
        Err(texts.with(Key::error_not_found, &[("path", &dir.display().to_string())]))
    }
}

/// The folders the app keeps its data in (checked when the setup file was
/// built and again when it was read, so they're inside the user's profile).
fn delete_data(manifest: &Manifest, texts: &Texts) -> Result<(), String> {
    for template in &manifest.uninstall.data {
        let Some(folder) = paths::expand_data_folder(template, |name| std::env::var(name).ok()) else {
            continue;
        };
        match fs::remove_dir_all(&folder) {
            Ok(()) => {}
            Err(err) if err.kind() == std::io::ErrorKind::NotFound => {}
            Err(err) => {
                return Err(format!(
                    "{} {}",
                    texts.with(Key::error_delete_data, &[("path", &folder.display().to_string())]),
                    describe(&err)
                ))
            }
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
