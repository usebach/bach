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
//! - `--silent`: the same work without a window, for IT tools and scripts.
//!   `--dir`, `--with`, `--without` and `--delete-data` make the choices;
//!   the exit code says how it went.
//! - `--preview`: the windows, with the work only pretended.
//! - `--theme light|dark|system` and `--lang <code>` override the payload's
//!   theme and the user's language.

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

#[derive(Debug, Default)]
pub struct Args {
    pub uninstall: bool,
    pub preview: bool,
    pub silent: bool,
    pub install_dir: Option<PathBuf>,
    /// `--dir`: where a silent install goes.
    pub dir: Option<PathBuf>,
    pub with: Vec<String>,
    pub without: Vec<String>,
    pub delete_data: bool,
    /// Lets a silent run install an older version over a newer one.
    pub allow_downgrade: bool,
    /// `--log FILE`: where this run's log goes.
    pub log: Option<PathBuf>,
    /// `--update`: look for a newer version and install it.
    pub update: bool,
    /// `--check-update`: only say whether there is one (exit code 10).
    pub check_update: bool,
    pub theme: Option<Theme>,
    pub lang: Option<String>,
}

impl Args {
    pub fn parse() -> Self {
        Self::from(std::env::args().skip(1).collect())
    }

    fn from(args: Vec<String>) -> Self {
        let value_of = |flag: &str| {
            args.iter()
                .position(|arg| arg.eq_ignore_ascii_case(flag))
                .and_then(|index| args.get(index + 1))
                .cloned()
        };
        let has = |flags: &[&str]| args.iter().any(|arg| flags.iter().any(|flag| arg.eq_ignore_ascii_case(flag)));
        let list = |flag: &str| {
            value_of(flag)
                .map(|value| value.split(',').map(|id| id.trim().to_ascii_lowercase()).filter(|id| !id.is_empty()).collect())
                .unwrap_or_default()
        };
        Self {
            uninstall: has(&["--uninstall"]),
            preview: has(&["--preview"]),
            // `/S` and `/quiet` are what IT tools try first.
            silent: has(&["--silent", "/s", "/silent", "/quiet", "/q"]),
            install_dir: value_of("--install-dir").map(PathBuf::from),
            dir: value_of("--dir").map(PathBuf::from),
            with: list("--with"),
            without: list("--without"),
            delete_data: has(&["--delete-data"]),
            allow_downgrade: has(&["--allow-downgrade"]),
            log: value_of("--log").map(PathBuf::from),
            update: has(&["--update"]),
            check_update: has(&["--check-update"]),
            theme: value_of("--theme").and_then(|value| Theme::parse(&value)),
            lang: value_of("--lang"),
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
    // A silent removal removes in place (its own file goes after it
    // exits), so whoever started it can wait for its exit code.
    if !args.silent && hand_over_to_temp_copy(&dir, package, args) {
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
fn hand_over_to_temp_copy(dir: &Path, package: &Package, args: &Args) -> bool {
    let copy = std::env::temp_dir().join(format!(
        "{}-uninstall-{}.exe",
        package.manifest.app.id.replace(' ', "-"),
        std::process::id()
    ));
    if std::fs::copy(&package.exe, &copy).is_err() {
        return false;
    }
    let mut command = std::process::Command::new(&copy);
    command.arg("--uninstall").arg("--install-dir").arg(dir);
    if let Some(lang) = &args.lang {
        command.args(["--lang", lang]);
    }
    if command.spawn().is_ok() {
        true
    } else {
        let _ = std::fs::remove_file(&copy);
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn args(line: &str) -> Args {
        Args::from(line.split_whitespace().map(String::from).collect())
    }

    #[test]
    fn flags_are_read() {
        let parsed = args("--silent --dir D:\\Apps\\Hello --with samples,PATH --without desktop-shortcut --lang de");
        assert!(parsed.silent && !parsed.preview);
        assert_eq!(parsed.dir, Some(PathBuf::from(r"D:\Apps\Hello")));
        assert_eq!(parsed.with, ["samples", "path"]);
        assert_eq!(parsed.without, ["desktop-shortcut"]);
        assert_eq!(parsed.lang.as_deref(), Some("de"));
        assert!(args("/S").silent);
        assert!(args("--preview --theme dark").theme == Some(Theme::Dark));
    }
}
