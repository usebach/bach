//! `bach version`: shows the version a project carries, or raises it
//! everywhere at once.
//!
//! ```text
//! bach version              list every file's version; exit 1 if they differ
//! bach version patch        1.4.2 → 1.4.3 in bach.toml, package.json, the
//!                           lock files, Cargo.toml, tauri.conf.json, …
//! bach version 2.0.0-rc.1   an exact version (never lower than the current)
//! bach version current      keep it (the very first release)
//! ```
//!
//! The new version is the only thing written to stdout, so a workflow can
//! capture it; everything else goes to stderr.

use std::cmp::Ordering;
use std::fmt;
use std::path::Path;

use anyhow::{bail, Result};

use crate::version_files::{self, Found};

/// `1.2.3` or `1.2.3-rc.1`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    core: [u64; 3],
    pre: Option<String>,
}

impl Version {
    /// A leading `v` is dropped; build metadata (`+x`) isn't supported.
    pub fn parse(text: &str) -> Option<Version> {
        let text = text.trim().trim_start_matches(['v', 'V']);
        let (core, pre) = match text.split_once('-') {
            Some((core, pre)) => (core, Some(pre)),
            None => (text, None),
        };
        let mut numbers = core.split('.').map(|part| {
            (!part.is_empty() && part.chars().all(|c| c.is_ascii_digit()) && (part == "0" || !part.starts_with('0')))
                .then(|| part.parse::<u64>().ok())
                .flatten()
        });
        let version = Version { core: [numbers.next()??, numbers.next()??, numbers.next()??], pre: None };
        if numbers.next().is_some() {
            return None;
        }
        let pre = match pre {
            None => None,
            Some(pre) if !pre.is_empty() && pre.chars().all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '-') => Some(pre.to_string()),
            Some(_) => return None,
        };
        Some(Version { pre, ..version })
    }

    /// What `target` makes of this version: patch, minor, major, current, or
    /// an exact version that isn't lower.
    pub fn next(&self, target: &str) -> Result<Version> {
        let [major, minor, patch] = self.core;
        Ok(match target {
            "current" => self.clone(),
            // 1.2.3-rc.1 → 1.2.3: releasing a pre-release's own version.
            "patch" if self.pre.is_some() => Version { core: self.core, pre: None },
            "patch" => Version { core: [major, minor, patch.saturating_add(1)], pre: None },
            "minor" => Version { core: [major, minor.saturating_add(1), 0], pre: None },
            "major" => Version { core: [major.saturating_add(1), 0, 0], pre: None },
            exact => {
                let Some(next) = Version::parse(exact) else {
                    bail!("\"{exact}\" isn't a version. Use patch, minor, major, current, or something like 1.0.0 or 1.0.0-rc.1.");
                };
                if next.cmp(self) == Ordering::Less {
                    bail!("{next} is lower than the current version, {self}.");
                }
                next
            }
        })
    }
}

impl Ord for Version {
    /// A pre-release sorts below its release, as in semver.
    fn cmp(&self, other: &Self) -> Ordering {
        self.core.cmp(&other.core).then_with(|| match (&self.pre, &other.pre) {
            (None, None) => Ordering::Equal,
            (None, Some(_)) => Ordering::Greater,
            (Some(_), None) => Ordering::Less,
            (Some(a), Some(b)) => a.cmp(b),
        })
    }
}

impl PartialOrd for Version {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.core[0], self.core[1], self.core[2])?;
        match &self.pre {
            Some(pre) => write!(f, "-{pre}"),
            None => Ok(()),
        }
    }
}

pub struct Options<'a> {
    pub config: &'a Path,
    pub target: Option<&'a str>,
    pub dry_run: bool,
}

pub fn command(options: Options) -> Result<()> {
    let dir = match options.config.parent() {
        Some(dir) if !dir.as_os_str().is_empty() => dir,
        _ => Path::new("."),
    };
    let found = version_files::read_all(dir)?;
    if found.is_empty() {
        bail!(
            "found no version in {} (looked for bach.toml, package.json, Cargo.toml, tauri.conf.json and .csproj files)",
            dir.display()
        );
    }
    match options.target {
        None => show(&found),
        Some(target) => raise(dir, &found, target, options.dry_run),
    }
}

/// Lists each file's version; differing versions are an error.
fn show(found: &[Found]) -> Result<()> {
    // A lock file carries the version twice; once is enough to read.
    let mut shown: Vec<&Found> = Vec::new();
    for item in found {
        if !shown.contains(&item) {
            shown.push(item);
            println!("  {:<28} {}", item.file, item.version);
        }
    }
    let first = &found[0].version;
    if let Some(other) = found.iter().find(|item| &item.version != first) {
        bail!("{} says {}, but {} says {}. Run `bach version current` to bring them together.", other.file, other.version, found[0].file, first);
    }
    eprintln!("Everything agrees on {first}.");
    Ok(())
}

/// The version bach.toml has is the app's; without one, the first file's.
fn raise(dir: &Path, found: &[Found], target: &str, dry_run: bool) -> Result<()> {
    let base_text = &found.iter().find(|item| item.file == "bach.toml").unwrap_or(&found[0]).version;
    let Some(base) = Version::parse(base_text) else {
        bail!("the current version, \"{base_text}\", isn't one of the form 1.2.3. Fix it first, or give an exact version.");
    };
    let new = base.next(target)?.to_string();
    let changes = version_files::write_all(dir, base_text, &new, dry_run)?;
    for change in &changes {
        eprintln!("{}{}: {} → {new}", if dry_run { "would change " } else { "" }, change.file, change.from);
    }
    if changes.is_empty() {
        eprintln!("Every file already says {new}.");
    }
    println!("{new}");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::parse(text).unwrap()
    }

    #[test]
    fn versions_are_read_strictly() {
        assert_eq!(v("v1.2.3").to_string(), "1.2.3");
        assert_eq!(v("1.2.3-rc.1").to_string(), "1.2.3-rc.1");
        for bad in ["1.2", "1.2.3.4", "01.2.3", "a.b.c", "1.2.3-", "1.2.3+build", "", "latest"] {
            assert!(Version::parse(bad).is_none(), "{bad}");
        }
    }

    #[test]
    fn the_bumps() {
        let now = v("1.4.2");
        assert_eq!(now.next("patch").unwrap().to_string(), "1.4.3");
        assert_eq!(now.next("minor").unwrap().to_string(), "1.5.0");
        assert_eq!(now.next("major").unwrap().to_string(), "2.0.0");
        assert_eq!(now.next("current").unwrap().to_string(), "1.4.2");
        assert_eq!(now.next("v1.4.2").unwrap().to_string(), "1.4.2", "the version as it is");
        assert_eq!(now.next("2.0.0-rc.1").unwrap().to_string(), "2.0.0-rc.1");
        assert!(now.next("1.4.1").unwrap_err().to_string().contains("lower"));
        assert!(now.next("soon").unwrap_err().to_string().contains("isn't a version"));
        assert_eq!(v("1.5.0-rc.1").next("patch").unwrap().to_string(), "1.5.0", "a pre-release's patch is its release");
        assert!(v("1.5.0").next("1.5.0-rc.1").is_err(), "a pre-release is lower than its release");
    }

    #[test]
    fn the_command_brings_files_together() {
        let dir = std::env::temp_dir().join(format!("bach-version-cmd-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("bach.toml"), "[app]\nversion = \"1.0.0\"\n").unwrap();
        std::fs::write(dir.join("package.json"), "{\"version\":\"0.5.0\"}").unwrap();
        let config = dir.join("bach.toml");
        assert!(command(Options { config: &config, target: None, dry_run: false }).is_err(), "differing versions are an error");
        command(Options { config: &config, target: Some("minor"), dry_run: false }).unwrap();
        assert!(command(Options { config: &config, target: None, dry_run: false }).is_ok());
        assert_eq!(std::fs::read_to_string(dir.join("package.json")).unwrap(), "{\"version\":\"1.1.0\"}", "bach.toml is the base");
        let empty = dir.join("nothing").join("bach.toml");
        std::fs::create_dir_all(empty.parent().unwrap()).unwrap();
        assert!(command(Options { config: &empty, target: Some("patch"), dry_run: false }).unwrap_err().to_string().contains("found no version"));
        std::fs::remove_dir_all(dir).unwrap();
    }
}
