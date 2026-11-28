//! `bach update`: brings `bach` itself up to date from GitHub.
//!
//! ```text
//! bach update            install the newest release
//! bach update 0.8.0      install exactly this version (older ones too)
//! bach update --check    only say whether there is a newer release
//! bach update --force    install the version again even if it's the one you have
//! ```
//!
//! Bach replaces itself only when it was installed the way the release
//! download and `install.ps1` / `install.sh` do it: loose files in a folder.
//! A copy that npm, Scoop, winget or Cargo manages is theirs to update, and
//! `bach update` says which command does it. See [`notice`] for the line that
//! tells people a release is out.

mod notice;
mod release;
mod swap;

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

use crate::output;
use crate::version::Version;
pub use notice::{refresh, show as notify};
use release::Release;

/// How this copy of `bach` got where it is.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    /// Loose files: `install.ps1`, `install.sh`, or an unpacked release.
    Standalone,
    Npm,
    Scoop,
    Winget,
    Cargo,
    /// Built from a checkout of Bach's source.
    Checkout,
}

impl Method {
    pub fn detect(exe: &Path) -> Method {
        let path = format!("/{}", exe.to_string_lossy().replace('\\', "/").to_lowercase());
        if path.contains("/node_modules/") {
            Method::Npm
        } else if path.contains("/scoop/apps/") {
            Method::Scoop
        } else if path.contains("/microsoft/winget/") {
            Method::Winget
        } else if path.contains("/.cargo/bin/") || cargo_bin().is_some_and(|bin| exe.starts_with(bin)) {
            Method::Cargo
        } else if path.contains("/target/debug/")
            || path.contains("/target/release/")
            || (path.contains("/target/") && path.contains("/release/"))
        {
            Method::Checkout
        } else {
            Method::Standalone
        }
    }

    /// The name `--json` uses.
    pub fn name(self) -> &'static str {
        match self {
            Method::Standalone => "standalone",
            Method::Npm => "npm",
            Method::Scoop => "scoop",
            Method::Winget => "winget",
            Method::Cargo => "cargo",
            Method::Checkout => "checkout",
        }
    }

    /// What updates this copy.
    pub fn command(self) -> String {
        match self {
            Method::Standalone => "bach update".into(),
            Method::Npm => "npm update --global @usebach/cli".into(),
            Method::Scoop => "scoop update bach".into(),
            Method::Winget => "winget upgrade bach".into(),
            Method::Cargo => "cargo install --git https://github.com/usebach/bach-installer bach-cli --locked --force".into(),
            Method::Checkout => "git pull, then cargo build --release".into(),
        }
    }

    fn manager(self) -> &'static str {
        match self {
            Method::Standalone => "",
            Method::Npm => "npm",
            Method::Scoop => "Scoop",
            Method::Winget => "winget",
            Method::Cargo => "Cargo",
            Method::Checkout => "a checkout of Bach's source",
        }
    }
}

/// Where `cargo install` puts programs: `$CARGO_HOME/bin`.
fn cargo_bin() -> Option<PathBuf> {
    std::env::var_os("CARGO_HOME").filter(|home| !home.is_empty()).map(|home| PathBuf::from(home).join("bin"))
}

/// The version of this `bach`.
pub fn current() -> Version {
    Version::parse(env!("CARGO_PKG_VERSION")).expect("Cargo.toml has a valid version")
}

/// Removes what an earlier update left behind: Windows doesn't let a running
/// program delete itself, so the old copy waited for this run. Costs two file
/// checks; every run does it.
pub fn tidy() {
    let Ok(exe) = std::env::current_exe() else { return };
    let setup = exe.with_file_name(crate::runtime::FILE_NAME);
    for file in [&exe, &setup] {
        let _ = std::fs::remove_file(swap::backup_of(file));
    }
}

pub struct Options<'a> {
    /// An exact version to install; the newest when it's left out.
    pub version: Option<&'a str>,
    pub check: bool,
    pub force: bool,
}

pub fn command(options: Options) -> Result<()> {
    let current = current();
    let exe = std::env::current_exe().context("couldn't find where bach is")?;
    let method = Method::detect(&exe);
    let wanted =
        options.version.map(|text| Version::parse(text).with_context(|| format!("\"{text}\" isn't a version like 0.8.0"))).transpose()?;
    // For tests, as for the install scripts: a folder of release files.
    let from = std::env::var_os("BACH_UPDATE_FROM").map(PathBuf::from);

    // npm hands out releases on its own schedule, so for a copy npm manages
    // the newest version is the one npm has, not the one GitHub shows first.
    let (target, release) = if method == Method::Npm && wanted.is_none() && from.is_none() {
        (release::npm_latest(None)?, None)
    } else {
        let release = match (&from, &wanted) {
            (Some(dir), Some(version)) => Release::from_folder(dir, version)?,
            (Some(_), None) => bail!("BACH_UPDATE_FROM needs a version: bach update VERSION"),
            (None, Some(version)) => Release::tagged(version)?,
            (None, None) => Release::latest(None)?,
        };
        (release.version.clone(), Some(release))
    };
    let newer = target > current;
    output::result("current", current.to_string());
    output::result("latest", target.to_string());
    output::result("update_available", newer);
    output::result("install", method.name());
    if wanted.is_none() {
        notice::remember(&target);
    }

    if options.check {
        if newer {
            say!("A new version of Bach is available: {current} to {target}.");
            say!("Update with: {}", method.command());
        } else {
            say!("Bach {current} is up to date.");
        }
        return Ok(());
    }

    // Nothing to do: the newest is the one installed (or an older, pre-release
    // build is ahead of it), and nobody asked for a particular version.
    if !options.force && (target == current || (wanted.is_none() && !newer)) {
        output::result("updated", false);
        say!("Bach {current} is {}.", if target == current { "already installed" } else { "up to date" });
        return Ok(());
    }

    if method != Method::Standalone {
        bail!("this bach is managed by {}, so {} updates it: {}", method.manager(), method.manager(), method.command());
    }

    // Only a standalone copy gets here, and it always asked GitHub (above).
    let release = release.context("there's no release to install from")?;
    say!("Updating Bach {current} to {target}");
    swap::apply(&release, &exe, from.as_deref())?;
    output::result("updated", true);
    say!("Bach {target} is installed. What's new: {}", release.page());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn method(path: &str) -> Method {
        Method::detect(Path::new(path))
    }

    #[test]
    fn the_way_bach_was_installed_is_read_from_where_it_is() {
        assert_eq!(method(r"C:\Users\me\AppData\Local\Programs\bach-cli\bach.exe"), Method::Standalone);
        assert_eq!(method("/home/me/.local/bin/bach"), Method::Standalone);
        assert_eq!(method(r"C:\Users\me\AppData\Roaming\npm\node_modules\@usebach\cli-win32-x64\bin\bach.exe"), Method::Npm);
        assert_eq!(method(r"C:\Users\me\scoop\apps\bach\current\bach.exe"), Method::Scoop);
        assert_eq!(method(r"C:\Users\me\AppData\Local\Microsoft\WinGet\Packages\Usebach.Bach_x\bach.exe"), Method::Winget);
        assert_eq!(method(r"C:\Users\me\.cargo\bin\bach.exe"), Method::Cargo);
        assert_eq!(method(r"C:\code\bach\target\release\bach.exe"), Method::Checkout);
        assert_eq!(method("/code/bach/target/x86_64-unknown-linux-gnu/release/bach"), Method::Checkout);
        // "target" alone in a name doesn't make a checkout.
        assert_eq!(method(r"C:\tools\targets\bach.exe"), Method::Standalone);
    }

    #[test]
    fn every_manager_has_its_own_command() {
        assert_eq!(Method::Standalone.command(), "bach update");
        for method in [Method::Npm, Method::Scoop, Method::Winget, Method::Cargo] {
            assert!(!method.command().is_empty() && !method.manager().is_empty());
            assert_ne!(method.command(), "bach update");
        }
    }

    #[test]
    fn this_build_has_a_version() {
        assert!(current() > Version::parse("0.0.0").unwrap());
    }
}
