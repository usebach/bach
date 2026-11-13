//! What this launch should be, from the command line and the payload.
//!
//! - A setup file (payload with files): the install window. Run again over
//!   an install, it updates it in place.
//! - `uninstall.exe` in an install folder (payload without files, or
//!   `--uninstall`): the remove window. A running program can't delete its
//!   own file, so the installed uninstaller first hands over to a copy of
//!   itself in the temp folder and exits; that copy (started with
//!   `--install-dir`) shows the window, removes everything and deletes
//!   itself once it closes.
//! - `--preview`: the same windows, with the work only pretended.
//! - `--theme light|dark|system`: overrides the payload's theme.

use std::path::{Path, PathBuf};

use bach_pack::manifest::{Theme, UNINSTALLER};

use crate::package::Package;
use crate::system;

#[derive(Clone, Debug)]
pub enum Mode {
    Install,
    /// Remove the install in `dir`.
    Remove { dir: PathBuf },
}

pub struct Args {
    pub uninstall: bool,
    pub preview: bool,
    pub install_dir: Option<PathBuf>,
    pub theme: Option<Theme>,
}

impl Args {
    pub fn parse() -> Self {
        let args: Vec<String> = std::env::args().skip(1).collect();
        let value_of = |flag: &str| {
            args.iter()
                .position(|arg| arg == flag)
                .and_then(|index| args.get(index + 1))
                .cloned()
        };
        Self {
            uninstall: args.iter().any(|arg| arg == "--uninstall"),
            preview: args.iter().any(|arg| arg == "--preview"),
            install_dir: value_of("--install-dir").map(PathBuf::from),
            theme: value_of("--theme").and_then(|value| Theme::parse(&value)),
        }
    }
}

/// The mode for this package. For the installed uninstaller this hands
/// over to a temp copy and exits the process.
pub fn mode(args: &Args, package: &Package) -> Mode {
    let removing = args.uninstall || !package.has_files();
    if !removing {
        return Mode::Install;
    }
    if args.preview {
        let dir = system::installed(&package.manifest)
            .map(|installed| installed.dir)
            .unwrap_or_else(|| system::default_install_dir(&package.manifest.install.folder));
        return Mode::Remove { dir };
    }
    if let Some(dir) = &args.install_dir {
        return Mode::Remove { dir: dir.clone() };
    }
    let dir = install_dir_of_this_uninstaller(package)
        .or_else(|| system::installed(&package.manifest).map(|installed| installed.dir))
        .unwrap_or_default();
    if hand_over_to_temp_copy(&dir, package) {
        std::process::exit(0);
    }
    // The hand-over failed: remove from here, and this file goes once the
    // process has exited.
    Mode::Remove { dir }
}

/// The folder this `uninstall.exe` sits in, when it's the installed one.
fn install_dir_of_this_uninstaller(package: &Package) -> Option<PathBuf> {
    let name = package.exe.file_name()?.to_string_lossy().to_lowercase();
    (name == UNINSTALLER).then(|| package.exe.parent().map(Path::to_path_buf)).flatten()
}

/// Starts a copy of this executable from the temp folder in remove mode.
/// True once that copy is running.
fn hand_over_to_temp_copy(dir: &Path, package: &Package) -> bool {
    let copy = std::env::temp_dir().join(format!("{}-uninstall-{}.exe", package.manifest.app.id.replace(' ', "-"), std::process::id()));
    if std::fs::copy(&package.exe, &copy).is_err() {
        return false;
    }
    std::process::Command::new(&copy)
        .arg("--uninstall")
        .arg("--install-dir")
        .arg(dir)
        .spawn()
        .is_ok()
}
