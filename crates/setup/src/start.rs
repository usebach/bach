//! Where an install starts from: a fresh one, or an update over an install
//! already on this computer, with the choices it made last time. Shared by
//! the window and a silent run, so both offer the same thing.

use std::path::PathBuf;

use bach_pack::manifest::{Choices, Manifest};

use crate::system::{self, Installed};
use crate::work;

pub struct Start {
    /// The install Settings › Apps knows about.
    pub existing: Option<Installed>,
    /// That install's manifest, read from its uninstaller.
    pub previous: Option<Manifest>,
    pub dir: PathBuf,
    pub choices: Choices,
    /// The installed version is newer than this setup file's.
    pub downgrade: bool,
}

/// The requirements this computer doesn't meet yet, by index.
pub fn missing_requirements(manifest: &Manifest) -> Vec<usize> {
    manifest
        .requirements
        .iter()
        .enumerate()
        .filter(|(_, requirement)| !crate::system::apps::is_present(&requirement.check))
        .map(|(index, _)| index)
        .collect()
}

pub fn install(manifest: &Manifest) -> Start {
    let existing = system::installed(manifest);
    let dir = existing
        .as_ref()
        .map(|installed| installed.dir.clone())
        .unwrap_or_else(|| system::default_install_dir(&manifest.install.folder));
    let previous = existing.as_ref().and_then(|_| work::previous_install(&dir, manifest));
    let choices = match &previous {
        Some(previous) => {
            let mut choices = Choices::for_update(manifest, previous);
            if previous.chosen.is_none() {
                // Bach 0.1 didn't record choices; its desktop shortcut tells.
                choices.desktop_shortcut = system::desktop_shortcut(manifest).is_ok_and(|path| path.exists());
            }
            choices
        }
        None => Choices::defaults(manifest),
    };
    let downgrade = existing.as_ref().is_some_and(|installed| {
        bach_pack::version::compare(&installed.version, &manifest.app.version) == Some(std::cmp::Ordering::Greater)
    });
    Start { existing, previous, dir, choices, downgrade }
}
