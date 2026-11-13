//! Installing and updating.
//!
//! 1. Check: no other setup for this app is running, the app isn't open,
//!    and the drive has room.
//! 2. Unpack the chosen files next to their targets, then move them into
//!    place all at once (see `place.rs`), so a failure leaves the earlier
//!    version as it was.
//! 3. On an update, remove the files the old install had and this one
//!    doesn't. The old file list comes from the old uninstaller.
//! 4. Write `uninstall.exe`: this runtime, with the installed manifest.
//! 5. Shortcuts, start with Windows, PATH, link schemes and file types,
//!    each added or taken away to match the choices, and the entry under
//!    Settings › Apps.

use std::collections::HashSet;
use std::fs::{self, File};
use std::io::{BufReader, Read, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use bach_pack::archive;
use bach_pack::container;
use bach_pack::i18n::Key;
use bach_pack::manifest::{Choices, Manifest, UNINSTALLER};
use bach_pack::paths;

use super::place::{self, Transaction};
use super::{describe, failed, is_disk_full, is_in_use, plain, remove_empty_parents, remove_file_if_present, Report};
use crate::package::Package;
use crate::system::{self, env_path, host, integration};
use crate::texts::{self, Texts};

/// Unpacking is most of the work; the steps after it share the rest.
const UNPACK_SHARE: f32 = 0.8;
/// Room left over after the files, for the uninstaller and the file system.
const SPARE_BYTES: u64 = 32 << 20;

pub struct InstallPlan {
    pub package: Arc<Package>,
    pub texts: Arc<Texts>,
    pub dir: PathBuf,
    pub choices: Choices,
}

pub fn install(plan: &InstallPlan, report: Report) -> Result<(), String> {
    let package = &plan.package;
    let texts = &plan.texts;
    let manifest = &package.manifest;
    let choices = plan.choices.clone().settle(manifest);
    let dir = &plan.dir;
    let Some(layout) = package.layout.filter(|layout| layout.has_archive()) else {
        return Err(texts.get(Key::error_damaged));
    };
    let _lock = host::SetupLock::acquire(&manifest.app.id).ok_or_else(|| texts.get(Key::error_busy))?;

    report(0.02, &texts.get(Key::step_preparing));
    create_install_dir(dir, texts)?;
    let program = manifest.executable_in(dir);
    ensure_closed(&program, texts)?;
    let previous = previous_install(dir, manifest);
    let needed = manifest.selected_size(&choices);
    check_free_space(dir, needed, texts)?;

    unpack(plan, &choices, layout.archive, needed, report)?;

    report(0.86, &texts.get(Key::step_tidying));
    let installed = manifest.as_installed(&choices);
    if let Some(previous) = &previous {
        remove_stale(dir, previous, &installed);
    }

    report(0.89, &texts.get(Key::step_uninstaller));
    write_uninstaller(package, &installed, dir, texts)?;

    report(0.92, &texts.get(Key::step_shortcuts));
    shortcuts(manifest, &choices, &program, texts)?;
    integrate(manifest, previous.as_ref(), &choices, dir, &program, texts)?;

    report(0.97, &texts.get(Key::step_registering));
    system::register(manifest, dir, needed)
        .map_err(|err| format!("{} {}", texts.get(Key::error_register), plain(&err)))?;
    system::notify_shell();
    report(1.0, &texts.get(Key::step_done));
    Ok(())
}

fn create_install_dir(dir: &Path, texts: &Texts) -> Result<(), String> {
    fs::create_dir_all(dir).map_err(|err| {
        if err.raw_os_error() == Some(5) {
            texts.with(Key::error_no_access, &[("path", &dir.display().to_string())])
        } else {
            failed(texts, Key::error_create, dir, &err)
        }
    })
}

/// Fails with a readable sentence when the app is running from `program`.
fn ensure_closed(program: &Path, texts: &Texts) -> Result<(), String> {
    if !program.exists() {
        return Ok(());
    }
    match fs::OpenOptions::new().write(true).open(program) {
        Err(err) if is_in_use(&err) => Err(texts.get(Key::error_still_open)),
        _ => Ok(()),
    }
}

fn check_free_space(dir: &Path, needed: u64, texts: &Texts) -> Result<(), String> {
    match host::free_space(dir) {
        Some(free) if free < needed.saturating_add(SPARE_BYTES) => Err(texts.with(
            Key::error_disk_full,
            &[("size", &texts::size(needed.saturating_add(SPARE_BYTES))), ("free", &texts::size(free))],
        )),
        _ => Ok(()),
    }
}

/// Unpacks the chosen files and moves them into place, all or nothing.
fn unpack(
    plan: &InstallPlan,
    choices: &Choices,
    archive_section: container::Section,
    needed: u64,
    report: Report,
) -> Result<(), String> {
    let texts = &plan.texts;
    let manifest = &plan.package.manifest;
    let dir = &plan.dir;
    let read_failed = |err: std::io::Error| format!("{} {}", texts.get(Key::error_read_setup), describe(&err));
    let wanted: HashSet<&str> =
        manifest.files.iter().filter(|file| choices.installs(file)).map(|file| file.path.as_str()).collect();
    for file in &manifest.files {
        place::clear_leftovers(&paths::under(dir, &file.path));
    }

    let copying = texts.get(Key::step_copying);
    let total = needed.max(1);
    let mut done = 0u64;
    let mut source = File::open(&plan.package.exe).map_err(read_failed)?;
    source.seek(SeekFrom::Start(archive_section.offset)).map_err(read_failed)?;
    let reader = BufReader::new(source.take(archive_section.len));
    let mut transaction = Transaction::default();
    let mut write_error: Option<String> = None;
    let unpacked = archive::read(reader, &mut |packaged, _size, body| {
        if !wanted.contains(packaged) {
            return Ok(());
        }
        let target = paths::under(dir, packaged);
        transaction
            .stage(&target, body, &mut |chunk| {
                done = done.saturating_add(chunk);
                report(0.04 + UNPACK_SHARE * (done as f32 / total as f32).min(1.0), &copying);
            })
            .map_err(|err| {
                write_error = Some(if is_disk_full(&err) {
                    texts.with(Key::error_disk_full, &[("size", &texts::size(needed)), ("free", "0 KB")])
                } else {
                    failed(texts, Key::error_write, &target, &err)
                });
                anyhow::anyhow!("{err}")
            })
    });
    if let Err(err) = unpacked {
        transaction.discard();
        remove_empty_parents(dir, wanted.iter().copied());
        return Err(write_error.unwrap_or_else(|| format!("{} {}", texts.get(Key::error_damaged), plain(&err))));
    }
    transaction.commit().map_err(|failure| {
        if is_in_use(&failure.error) {
            texts.with(Key::error_in_use, &[("path", &failure.target.display().to_string())])
        } else {
            failed(texts, Key::error_write, &failure.target, &failure.error)
        }
    })
}

/// The install already in `dir`, read from its uninstaller: only when it's
/// this same app (another app's uninstaller in a shared folder is none of
/// our business), and None when it can't be read, in which case nothing is
/// cleaned up rather than guessed.
pub fn previous_install(dir: &Path, manifest: &Manifest) -> Option<Manifest> {
    let mut file = File::open(dir.join(UNINSTALLER)).ok()?;
    let layout = container::locate(&mut file).ok()??;
    let previous = container::read_manifest(&mut file, &layout).ok()?;
    (previous.app.id == manifest.app.id).then_some(previous)
}

/// Removes what the old install put there and the new one doesn't.
fn remove_stale(dir: &Path, previous: &Manifest, installed: &Manifest) {
    let keep: HashSet<String> = installed.files.iter().map(|file| file.path.to_lowercase()).collect();
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

/// `uninstall.exe`: the runtime's own bytes with the installed manifest and
/// the logo, and no files.
fn write_uninstaller(package: &Package, installed: &Manifest, dir: &Path, texts: &Texts) -> Result<(), String> {
    let fail = |detail: String| format!("{} {}", texts.get(Key::error_uninstaller), super::sentence(detail));
    let layout = package.layout.ok_or_else(|| fail("this setup file has no payload".into()))?;
    let mut source = File::open(&package.exe).map_err(|err| fail(describe(&err)))?;
    let runtime_len = usize::try_from(layout.runtime_len).map_err(|_| texts.get(Key::error_damaged))?;
    let mut runtime = vec![0u8; runtime_len];
    source.read_exact(&mut runtime).map_err(|err| fail(describe(&err)))?;
    let logo = container::read_logo(&mut source, &layout).map_err(|err| fail(plain(&err)))?;

    let target = dir.join(UNINSTALLER);
    let partial = place::with_suffix(&target, ".bach-new");
    let written = File::create(&partial)
        .map_err(anyhow::Error::from)
        .and_then(|mut out| container::write_uninstaller(&mut out, &runtime, &logo, installed).map(|_| out))
        .and_then(|out| out.sync_all().map_err(anyhow::Error::from));
    if let Err(err) = written {
        let _ = fs::remove_file(&partial);
        return Err(fail(plain(&err)));
    }
    fs::rename(&partial, &target).map_err(|err| {
        let _ = fs::remove_file(&partial);
        if is_in_use(&err) {
            texts.with(Key::error_in_use, &[("path", &target.display().to_string())])
        } else {
            fail(describe(&err))
        }
    })
}

/// Adds or removes the Start menu and desktop shortcuts to match the
/// choices (an update may have unticked one).
fn shortcuts(manifest: &Manifest, choices: &Choices, program: &Path, texts: &Texts) -> Result<(), String> {
    let description = manifest.app.description.clone().unwrap_or_else(|| manifest.app.name.clone());
    let wanted = [
        (system::start_menu_shortcut(manifest), choices.start_menu_shortcut),
        (system::desktop_shortcut(manifest), choices.desktop_shortcut),
    ];
    for (link, wanted) in wanted {
        let Ok(link) = link else {
            continue;
        };
        if wanted {
            system::create_shortcut(&link, program, &description).map_err(|err| {
                format!("{} {}", texts.with(Key::error_shortcut, &[("path", &link.display().to_string())]), plain(&err))
            })?;
        } else {
            let _ = remove_file_if_present(&link);
        }
    }
    Ok(())
}

/// Start with Windows, PATH, link schemes and file types: added when
/// chosen, and taken back when an update no longer wants them.
fn integrate(
    manifest: &Manifest,
    previous: Option<&Manifest>,
    choices: &Choices,
    dir: &Path,
    program: &Path,
    texts: &Texts,
) -> Result<(), String> {
    let fail = |err: anyhow::Error| format!("{} {}", texts.get(Key::error_system), plain(&err));
    if choices.start_with_windows {
        integration::set_autostart(manifest, program).map_err(fail)?;
    } else {
        integration::clear_autostart(manifest, program).map_err(fail)?;
    }

    // PATH is only touched when this app put something there or will now.
    let path_folder = manifest.path_folder_in(dir);
    if let Some(previous) = previous.filter(|previous| previous.chosen.as_ref().is_some_and(|before| before.add_to_path)) {
        let old_folder = previous.path_folder_in(dir);
        if old_folder != path_folder || !choices.add_to_path {
            env_path::remove(&old_folder).map_err(fail)?;
        }
    }
    if choices.add_to_path {
        env_path::add(&path_folder).map_err(fail)?;
    }

    if let Some(previous) = previous {
        for old in previous.protocols.iter().filter(|old| !manifest.protocols.contains(old)) {
            integration::unregister_protocol(manifest, old).map_err(fail)?;
        }
        for old in previous.file_types.iter().filter(|old| !manifest.file_types.iter().any(|new| new.extension == old.extension)) {
            integration::unregister_file_type(manifest, old).map_err(fail)?;
        }
    }
    for protocol in &manifest.protocols {
        integration::register_protocol(manifest, protocol, program).map_err(fail)?;
    }
    for file_type in &manifest.file_types {
        integration::register_file_type(manifest, file_type, program).map_err(fail)?;
    }
    Ok(())
}
