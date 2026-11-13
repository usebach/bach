//! The manifest: everything the setup window needs to know about the app it
//! installs. The builder writes it from `bach.toml`; the setup runtime reads
//! it back out of its own file. It's JSON, so a setup file can be inspected
//! with `bach inspect`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::paths;

/// The uninstaller's file name in every install folder.
pub const UNINSTALLER: &str = "uninstall.exe";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Manifest {
    /// The payload format, [`crate::FORMAT_VERSION`].
    pub format: u32,
    /// The Bach version that built this setup file.
    pub builder: String,
    pub app: App,
    pub install: Install,
    pub uninstall: Uninstall,
    pub look: Look,
    /// Every installed file, relative to the install folder. The uninstaller
    /// removes exactly these, and an update removes the ones a new version
    /// no longer ships.
    pub files: Vec<FileEntry>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct App {
    /// Names the registry entry; stays the same across versions.
    pub id: String,
    pub name: String,
    pub version: String,
    pub publisher: String,
    /// One line, used for the shortcut's tooltip.
    pub description: Option<String>,
    pub website: Option<String>,
    /// The program to start, relative to the install folder.
    pub executable: String,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Install {
    /// The folder's name under `%LOCALAPPDATA%\Programs`, and the name added
    /// to any folder the user picks instead.
    pub folder: String,
    /// Whether the desktop shortcut box starts ticked on a first install.
    pub desktop_shortcut: bool,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Uninstall {
    /// Folders such as `%APPDATA%\Hello`, deleted only when the user ticks
    /// "Also delete my data".
    pub data: Vec<String>,
    /// What lives in them, in a few words.
    pub data_description: Option<String>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct Look {
    pub theme: Theme,
    /// `#rrggbb` for buttons, the progress bar and ticked boxes.
    pub accent: Option<String>,
    /// The logo stored next to the manifest, if any.
    pub logo: Option<LogoKind>,
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    /// Follow Windows' light or dark setting, live.
    #[default]
    System,
    Light,
    Dark,
}

impl Theme {
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "system" => Some(Self::System),
            "light" => Some(Self::Light),
            "dark" => Some(Self::Dark),
            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum LogoKind {
    /// Drawn in one colour that follows the theme, like the Bach mark.
    Svg,
    /// Drawn as is, in full colour.
    Png,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileEntry {
    /// Relative, `/`-separated, checked by [`paths::normalize`].
    pub path: String,
    pub size: u64,
}

impl Manifest {
    /// All installed bytes, for the progress bar and Settings › Apps.
    pub fn total_size(&self) -> u64 {
        self.files.iter().fold(0u64, |sum, file| sum.saturating_add(file.size))
    }

    /// The program inside an install folder.
    pub fn executable_in(&self, dir: &Path) -> PathBuf {
        paths::under(dir, &self.app.executable)
    }

    /// Checks what a setup runtime relies on, so a damaged or hand-edited
    /// payload fails with a sentence instead of misbehaving.
    pub fn validate(&self) -> anyhow::Result<()> {
        use anyhow::{bail, Context};
        if self.format != crate::FORMAT_VERSION {
            bail!(
                "this setup file uses payload format {}, but this runtime reads format {}",
                self.format,
                crate::FORMAT_VERSION
            );
        }
        if !paths::is_valid_id(&self.app.id) {
            bail!("the app id \"{}\" can't name a folder", self.app.id);
        }
        if !paths::is_valid_id(&self.install.folder) {
            bail!("the install folder \"{}\" isn't a plain folder name", self.install.folder);
        }
        if self.app.name.trim().is_empty() || self.app.version.trim().is_empty() {
            bail!("the app has no name or version");
        }
        paths::normalize(&self.app.executable).context("the app's executable")?;
        for file in &self.files {
            let normal = paths::normalize(&file.path)?;
            if normal != file.path {
                bail!("\"{}\" isn't stored in its canonical form", file.path);
            }
            if normal.eq_ignore_ascii_case(UNINSTALLER) {
                bail!("the app ships its own {UNINSTALLER}, which would replace Bach's uninstaller");
            }
        }
        for folder in &self.uninstall.data {
            paths::check_data_folder(folder)?;
        }
        Ok(())
    }
}

#[cfg(test)]
pub(crate) fn sample() -> Manifest {
    Manifest {
        format: crate::FORMAT_VERSION,
        builder: crate::VERSION.into(),
        app: App {
            id: "Hello".into(),
            name: "Hello".into(),
            version: "1.0.0".into(),
            publisher: "Example".into(),
            description: Some("Says hello.".into()),
            website: None,
            executable: "hello.exe".into(),
        },
        install: Install {
            folder: "Hello".into(),
            desktop_shortcut: true,
        },
        uninstall: Uninstall {
            data: vec![r"%APPDATA%\Hello".into()],
            data_description: Some("Settings.".into()),
        },
        look: Look::default(),
        files: vec![
            FileEntry { path: "hello.exe".into(), size: 10 },
            FileEntry { path: "data/readme.txt".into(), size: 5 },
        ],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_json() {
        let manifest = sample();
        let json = serde_json::to_string(&manifest).unwrap();
        let back: Manifest = serde_json::from_str(&json).unwrap();
        assert_eq!(back, manifest);
        assert_eq!(back.total_size(), 15);
        assert!(back.validate().is_ok());
    }

    #[test]
    fn rejects_other_formats_and_bad_paths() {
        let mut manifest = sample();
        manifest.format = 99;
        assert!(manifest.validate().is_err());

        let mut manifest = sample();
        manifest.files.push(FileEntry { path: "../evil.dll".into(), size: 1 });
        assert!(manifest.validate().is_err());
    }
}
