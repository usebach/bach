//! `bach.toml`: what to install and how the installer looks.
//!
//! [`load`] reads the file, checks every value and gathers the files to
//! package, so `bach check` and `bach build` fail early, with a sentence
//! naming the setting at fault, instead of producing a broken installer.
//! Paths in the file are relative to the file itself.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use serde::Deserialize;

use crate::archive::{Compression, Source};
use crate::manifest::{self, FileEntry, LogoKind, Manifest};
use crate::paths;

/// The default output, relative to `bach.toml`.
pub const DEFAULT_OUTPUT: &str = "dist/{name}-Setup-{version}.exe";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct File {
    app: AppSection,
    files: FilesSection,
    #[serde(default)]
    installer: InstallerSection,
    #[serde(default)]
    uninstall: UninstallSection,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct AppSection {
    name: String,
    version: String,
    publisher: String,
    executable: String,
    id: Option<String>,
    description: Option<String>,
    website: Option<String>,
    icon: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct FilesSection {
    source: String,
    #[serde(default)]
    exclude: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct InstallerSection {
    output: Option<String>,
    folder: Option<String>,
    desktop_shortcut: Option<bool>,
    theme: Option<String>,
    accent: Option<String>,
    logo: Option<String>,
    compression: Option<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct UninstallSection {
    #[serde(default)]
    data: Vec<String>,
    data_description: Option<String>,
}

/// A checked `bach.toml`, ready to build.
#[derive(Debug)]
pub struct Project {
    /// The folder `bach.toml` is in.
    pub root: PathBuf,
    pub manifest: Manifest,
    /// The files to package, in the manifest's order.
    pub sources: Vec<Source>,
    /// A `.ico` for the setup file's own icon.
    pub icon: Option<PathBuf>,
    /// The logo's bytes, read now so the build can't race an edit.
    pub logo: Option<Vec<u8>>,
    pub output: PathBuf,
    pub compression: Compression,
}

/// Reads, checks and gathers a `bach.toml`.
pub fn load(path: &Path) -> Result<Project> {
    let text = std::fs::read_to_string(path)
        .with_context(|| format!("couldn't read {}", path.display()))?;
    let root = path
        .parent()
        .map(|parent| if parent.as_os_str().is_empty() { Path::new(".") } else { parent })
        .unwrap_or(Path::new("."))
        .to_path_buf();
    parse(&text, &root).with_context(|| format!("{} isn't valid", path.display()))
}

/// [`load`] for text already in hand, with paths relative to `root`.
pub fn parse(text: &str, root: &Path) -> Result<Project> {
    let file: File = toml::from_str(text).map_err(|err| anyhow::anyhow!("{}", err.to_string().trim()))?;
    let app = check_app(&file.app)?;
    let install = check_install(&file.installer, &app)?;
    let uninstall = check_uninstall(&file.uninstall)?;

    let theme = match &file.installer.theme {
        None => manifest::Theme::System,
        Some(value) => manifest::Theme::parse(value)
            .with_context(|| format!("installer.theme is \"{value}\"; use \"system\", \"light\" or \"dark\""))?,
    };
    let accent = match &file.installer.accent {
        None => None,
        Some(value) => Some(parse_hex(value).map(|_| value.trim().to_ascii_lowercase()).with_context(|| {
            format!("installer.accent is \"{value}\"; use a colour like \"#2563eb\"")
        })?),
    };
    let compression = match &file.installer.compression {
        None => Compression::default(),
        Some(value) => Compression::parse(value)
            .with_context(|| format!("installer.compression is \"{value}\"; use \"best\", \"normal\" or \"fast\""))?,
    };

    let icon = match &file.app.icon {
        None => None,
        Some(icon) => {
            let path = existing_file(root, icon, "app.icon")?;
            if !has_extension(&path, "ico") {
                bail!("app.icon must be a .ico file, so Windows can show it; {icon} isn't");
            }
            Some(path)
        }
    };
    let (logo_kind, logo) = match &file.installer.logo {
        None => (None, None),
        Some(logo) => {
            let path = existing_file(root, logo, "installer.logo")?;
            let kind = if has_extension(&path, "svg") {
                LogoKind::Svg
            } else if has_extension(&path, "png") {
                LogoKind::Png
            } else {
                bail!("installer.logo must be an .svg or a .png file; {logo} is neither");
            };
            let bytes = std::fs::read(&path).with_context(|| format!("couldn't read {}", path.display()))?;
            (Some(kind), Some(bytes))
        }
    };

    let output_template = file.installer.output.as_deref().unwrap_or(DEFAULT_OUTPUT);
    let output = root.join(fill_template(output_template, &app));
    if !has_extension(&output, "exe") {
        bail!("installer.output must end in .exe; it's \"{output_template}\"");
    }

    let source_dir = root.join(&file.files.source);
    if !source_dir.is_dir() {
        bail!("files.source is \"{}\", but {} isn't a folder", file.files.source, source_dir.display());
    }
    for pattern in &file.files.exclude {
        if pattern.trim().is_empty() {
            bail!("files.exclude has an empty pattern");
        }
    }
    let sources = gather(&source_dir, &file.files.exclude, &output)?;
    if sources.is_empty() {
        bail!("{} has no files to install", source_dir.display());
    }
    if let Some(clash) = sources.iter().find(|source| source.packaged.eq_ignore_ascii_case(manifest::UNINSTALLER)) {
        bail!(
            "{} would replace the uninstaller Bach adds; rename it or add it to files.exclude",
            clash.path.display()
        );
    }
    let executable_known = sources
        .iter()
        .any(|source| source.packaged.eq_ignore_ascii_case(&app.executable));
    if !executable_known {
        bail!(
            "app.executable is \"{}\", but there's no such file in {} (or it's excluded)",
            app.executable,
            source_dir.display()
        );
    }

    let files = sources
        .iter()
        .map(|source| FileEntry { path: source.packaged.clone(), size: source.size })
        .collect();
    let manifest = Manifest {
        format: crate::FORMAT_VERSION,
        builder: crate::VERSION.into(),
        app,
        install,
        uninstall,
        look: manifest::Look { theme, accent, logo: logo_kind },
        files,
    };
    manifest.validate()?;
    Ok(Project { root: root.to_path_buf(), manifest, sources, icon, logo, output, compression })
}

fn check_app(section: &AppSection) -> Result<manifest::App> {
    let name = required(&section.name, "app.name")?;
    let version = required(&section.version, "app.version")?;
    let publisher = required(&section.publisher, "app.publisher")?;
    let id = match &section.id {
        Some(id) => id.trim().to_string(),
        None => name.clone(),
    };
    if !paths::is_valid_id(&id) {
        let hint = if section.id.is_some() { "app.id" } else { "app.name (or set app.id)" };
        bail!(
            "{hint} is \"{id}\"; it names a folder and a registry key, so use letters, digits, \
             spaces, dots, dashes and underscores"
        );
    }
    let executable = paths::normalize(section.executable.trim()).context("app.executable isn't a usable path")?;
    if !executable.to_ascii_lowercase().ends_with(".exe") {
        bail!("app.executable is \"{executable}\"; it must be an .exe");
    }
    if let Some(website) = &section.website {
        if !(website.starts_with("https://") || website.starts_with("http://")) {
            bail!("app.website is \"{website}\"; it must start with https://");
        }
    }
    Ok(manifest::App {
        id,
        name,
        version,
        publisher,
        description: optional(&section.description),
        website: optional(&section.website),
        executable,
    })
}

fn check_install(section: &InstallerSection, app: &manifest::App) -> Result<manifest::Install> {
    let folder = match &section.folder {
        Some(folder) => folder.trim().to_string(),
        None => app.id.clone(),
    };
    if !paths::is_valid_id(&folder) {
        bail!("installer.folder is \"{folder}\"; it must be a plain folder name, like \"{}\"", app.id);
    }
    Ok(manifest::Install { folder, desktop_shortcut: section.desktop_shortcut.unwrap_or(true) })
}

fn check_uninstall(section: &UninstallSection) -> Result<manifest::Uninstall> {
    for folder in &section.data {
        paths::check_data_folder(folder).context("uninstall.data")?;
    }
    Ok(manifest::Uninstall { data: section.data.clone(), data_description: optional(&section.data_description) })
}

fn required(value: &str, key: &str) -> Result<String> {
    let value = value.trim();
    if value.is_empty() {
        bail!("{key} is empty");
    }
    Ok(value.to_string())
}

fn optional(value: &Option<String>) -> Option<String> {
    value.as_ref().map(|v| v.trim().to_string()).filter(|v| !v.is_empty())
}

fn existing_file(root: &Path, relative: &str, key: &str) -> Result<PathBuf> {
    let path = root.join(relative);
    if !path.is_file() {
        bail!("{key} is \"{relative}\", but {} isn't a file", path.display());
    }
    Ok(path)
}

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension().is_some_and(|ext| ext.eq_ignore_ascii_case(extension))
}

/// `{name}`, `{version}` and `{id}` in the output path.
pub fn fill_template(template: &str, app: &manifest::App) -> String {
    let clean = |value: &str| value.chars().map(|c| if r#"<>:"/\|?*"#.contains(c) { '-' } else { c }).collect::<String>();
    template
        .replace("{name}", &clean(&app.name))
        .replace("{version}", &clean(&app.version))
        .replace("{id}", &clean(&app.id))
}

/// `#rrggbb` (or `rrggbb`) as a number.
pub fn parse_hex(value: &str) -> Option<u32> {
    let hex = value.trim().trim_start_matches('#');
    if hex.len() != 6 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    u32::from_str_radix(hex, 16).ok()
}

/// Every file under `dir`, sorted, minus the excluded ones and the output.
fn gather(dir: &Path, exclude: &[String], output: &Path) -> Result<Vec<Source>> {
    let output = std::fs::canonicalize(output).ok();
    let mut found = BTreeMap::new();
    walk(dir, "", &mut |packaged, path, size| {
        if exclude.iter().any(|pattern| paths::matches(pattern, packaged)) {
            return Ok(());
        }
        if output.is_some() && std::fs::canonicalize(path).ok() == output {
            return Ok(());
        }
        let key = packaged.to_lowercase();
        if found.contains_key(&key) {
            bail!("{} is in the source folder twice, in different letter case", packaged);
        }
        found.insert(key, Source { packaged: packaged.to_string(), path: path.to_path_buf(), size });
        Ok(())
    })?;
    Ok(found.into_values().collect())
}

fn walk(dir: &Path, prefix: &str, visit: &mut dyn FnMut(&str, &Path, u64) -> Result<()>) -> Result<()> {
    let entries = std::fs::read_dir(dir).with_context(|| format!("couldn't list {}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            bail!("{} has a name that isn't valid text", entry.path().display());
        };
        let packaged = if prefix.is_empty() { name.to_string() } else { format!("{prefix}/{name}") };
        let path = entry.path();
        let meta = std::fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            bail!("{} is a symbolic link; Bach packages real files only", path.display());
        }
        if meta.is_dir() {
            walk(&path, &packaged, visit)?;
        } else if meta.is_file() {
            paths::normalize(&packaged).with_context(|| format!("can't package {}", path.display()))?;
            visit(&packaged, &path, meta.len())?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn project_dir(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bach-config-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("build/sub")).unwrap();
        std::fs::write(dir.join("build/hello.exe"), b"MZ").unwrap();
        std::fs::write(dir.join("build/sub/data.txt"), b"data").unwrap();
        std::fs::write(dir.join("build/hello.pdb"), b"debug").unwrap();
        dir
    }

    const MINIMAL: &str = r#"
        [app]
        name = "Hello"
        version = "1.2.3"
        publisher = "Example"
        executable = "hello.exe"

        [files]
        source = "build"
        exclude = ["*.pdb"]
    "#;

    #[test]
    fn a_minimal_config_builds() {
        let dir = project_dir("minimal");
        let project = parse(MINIMAL, &dir).unwrap();
        let paths: Vec<_> = project.manifest.files.iter().map(|f| f.path.as_str()).collect();
        assert_eq!(paths, ["hello.exe", "sub/data.txt"]);
        assert_eq!(project.manifest.app.id, "Hello");
        assert_eq!(project.manifest.install.folder, "Hello");
        assert!(project.manifest.install.desktop_shortcut);
        assert_eq!(project.output, dir.join("dist/Hello-Setup-1.2.3.exe"));
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn mistakes_name_their_setting() {
        let dir = project_dir("mistakes");
        let cases = [
            (MINIMAL.replace("hello.exe\"", "missing.exe\""), "app.executable"),
            (MINIMAL.replace("\"build\"", "\"nowhere\""), "files.source"),
            (format!("{MINIMAL}\n[installer]\ntheme = \"blue\""), "installer.theme"),
            (format!("{MINIMAL}\n[installer]\naccent = \"red\""), "installer.accent"),
            (format!("{MINIMAL}\n[uninstall]\ndata = ['C:\\Windows']"), "uninstall.data"),
            (MINIMAL.replace("name = \"Hello\"", "name = \"He/llo\""), "app.name"),
        ];
        for (text, key) in cases {
            let err = format!("{:#}", parse(&text, &dir).unwrap_err());
            assert!(err.contains(key), "expected {key} in: {err}");
        }
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn unknown_settings_are_typos() {
        let dir = project_dir("typo");
        let text = MINIMAL.replace("[files]", "nmae = \"x\"\n[files]");
        assert!(parse(&text, &dir).is_err());
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn output_names_are_filled_in() {
        let dir = project_dir("template");
        let app = parse(MINIMAL, &dir).unwrap().manifest.app;
        assert_eq!(fill_template("{name}_{version}_{id}.exe", &app), "Hello_1.2.3_Hello.exe");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn hex_colours() {
        assert_eq!(parse_hex("#2563eb"), Some(0x2563eb));
        assert_eq!(parse_hex("FFFFFF"), Some(0xffffff));
        assert_eq!(parse_hex("#fff"), None);
    }
}
