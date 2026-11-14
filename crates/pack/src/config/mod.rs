//! `bach.toml`: what to install and how the installer looks and talks.
//!
//! [`load`] reads the file, checks every value and gathers the files to
//! package, so `bach check` and `bach build` fail early, with a sentence
//! naming the setting at fault, instead of producing a broken installer.
//! Paths in the file are relative to the file itself.

mod extras;
mod gather;
mod icon;
mod options;
mod text;

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
    #[serde(default)]
    components: Vec<options::ComponentSection>,
    #[serde(default)]
    protocols: Vec<options::ProtocolSection>,
    #[serde(default)]
    file_types: Vec<options::FileTypeSection>,
    #[serde(default)]
    text: BTreeMap<String, toml::Value>,
    #[serde(default)]
    requirements: Vec<extras::RequirementSection>,
    #[serde(default)]
    shortcuts: Vec<extras::ShortcutSection>,
    updates: Option<extras::UpdatesSection>,
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
    theme: Option<String>,
    accent: Option<String>,
    logo: Option<String>,
    compression: Option<String>,
    languages: Option<Vec<String>>,
    license: Option<toml::Value>,
    desktop_shortcut: Option<toml::Value>,
    start_menu_shortcut: Option<toml::Value>,
    start_with_windows: Option<toml::Value>,
    add_to_path: Option<toml::Value>,
    path_folder: Option<String>,
    change_folder: Option<bool>,
    open_button: Option<bool>,
    start_menu_folder: Option<String>,
    website_button: Option<bool>,
    logo_height: Option<u16>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct UninstallSection {
    #[serde(default)]
    data: Vec<String>,
    data_description: Option<toml::Value>,
    feedback_url: Option<String>,
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
    /// Things that build fine but probably aren't meant, one sentence each.
    pub warnings: Vec<String>,
    /// `updates.download_url`: where the setup file will live, for the
    /// feed JSON `bach build` writes next to it.
    pub update_download_url: Option<String>,
}

/// Reads, checks and gathers a `bach.toml`.
pub fn load(path: &Path) -> Result<Project> {
    let text = std::fs::read_to_string(path).with_context(|| format!("couldn't read {}", path.display()))?;
    let root = path
        .parent()
        .map(|parent| if parent.as_os_str().is_empty() { Path::new(".") } else { parent })
        .unwrap_or(Path::new("."))
        .to_path_buf();
    parse(&text, &root).with_context(|| format!("{} isn't valid", path.display()))
}

/// [`load`] for text already in hand, with paths relative to `root`.
pub fn parse(text: &str, root: &Path) -> Result<Project> {
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    let file: File = toml::from_str(text).map_err(|err| anyhow::anyhow!("{}", err.to_string().trim()))?;
    let mut warnings = Vec::new();
    let app = check_app(&file.app)?;
    let install = options::install(&file.installer, &app)?;
    let languages = text::languages(file.installer.languages.as_deref())?;
    let mut locale = text::locale(&file.text, &languages, &mut warnings)?;
    locale.license = text::license(file.installer.license.as_ref(), root, &languages)?;
    if let Some(value) = &file.uninstall.data_description {
        let description = text::localized(value, "uninstall.data_description", &languages)?;
        locale.text.insert(crate::i18n::Key::delete_data_description.name().into(), description);
    }
    for folder in &file.uninstall.data {
        paths::check_data_folder(folder).context("uninstall.data")?;
    }

    let theme = match &file.installer.theme {
        None => manifest::Theme::System,
        Some(value) => manifest::Theme::parse(value)
            .with_context(|| format!("installer.theme is \"{value}\"; use \"system\", \"light\" or \"dark\""))?,
    };
    let accent = match &file.installer.accent {
        None => None,
        Some(value) => Some(
            parse_hex(value)
                .map(|_| value.trim().to_ascii_lowercase())
                .with_context(|| format!("installer.accent is \"{value}\"; use a colour like \"#2563eb\""))?,
        ),
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
    let (logo_kind, logo, hide_logo) = logo(file.installer.logo.as_deref(), icon.as_deref(), root, &mut warnings)?;
    let logo_aspect = match (logo_kind, &logo) {
        (Some(LogoKind::Png), Some(png)) => extras::png_aspect(png).filter(|aspect| (aspect - 1.0).abs() > 0.02),
        _ => None,
    };
    if let Some(height) = file.installer.logo_height {
        if !(32..=96).contains(&height) {
            bail!("installer.logo_height is {height}; use 32 to 96 (points)");
        }
    }
    if file.app.website.is_none() && file.installer.website_button == Some(true) {
        bail!("installer.website_button is true, but app.website isn't set");
    }
    let (updates, update_download_url) = extras::updates(file.updates.as_ref())?;
    let feedback_url = extras::https(&file.uninstall.feedback_url, "uninstall.feedback_url")?;

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
    let sources = gather::gather(&source_dir, &file.files.exclude, &output)?;
    if sources.is_empty() {
        bail!("{} has no files to install", source_dir.display());
    }
    if let Some(clash) = sources.iter().find(|source| source.packaged.eq_ignore_ascii_case(manifest::UNINSTALLER)) {
        bail!(
            "{} would replace the uninstaller Bach adds; rename it or add it to files.exclude",
            clash.path.display()
        );
    }
    if !sources.iter().any(|source| source.packaged.eq_ignore_ascii_case(&app.executable)) {
        bail!(
            "app.executable is \"{}\", but there's no such file in {} (or it's excluded)",
            app.executable,
            source_dir.display()
        );
    }

    let mut files: Vec<FileEntry> = sources.iter().map(|source| FileEntry::new(&source.packaged, source.size)).collect();
    let components = options::components(&file.components, &mut files, &app, &languages)?;
    let manifest = Manifest {
        format: crate::FORMAT_VERSION,
        builder: crate::VERSION.into(),
        app,
        install,
        uninstall: manifest::Uninstall { data: file.uninstall.data.clone(), data_description: None, feedback_url },
        look: manifest::Look { theme, accent, logo: logo_kind, hide_logo, logo_height: file.installer.logo_height, logo_aspect },
        files,
        components,
        protocols: options::protocols(&file.protocols)?,
        file_types: options::file_types(&file.file_types)?,
        locale,
        chosen: None,
        requirements: extras::requirements(&file.requirements)?,
        shortcuts: Vec::new(),
        updates,
        archive_sha256: None,
    };
    if manifest.install.add_to_path != manifest::Choice::Never {
        if let Some(folder) = &manifest.install.path_folder {
            let inside = manifest.files.iter().any(|entry| entry.path.to_lowercase().starts_with(&format!("{}/", folder.to_lowercase())));
            if !inside {
                warnings.push(format!("installer.path_folder is \"{folder}\", but no installed file is in it"));
            }
        }
    }
    let mut manifest = manifest;
    manifest.shortcuts = extras::shortcuts(&file.shortcuts, &manifest.files)?;
    manifest.validate()?;
    Ok(Project {
        root: root.to_path_buf(),
        manifest,
        sources,
        icon,
        logo,
        output,
        compression,
        warnings,
        update_download_url,
    })
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

/// The logo: a file, `"none"`, or the app's icon when it has a PNG image.
fn logo(
    logo: Option<&str>,
    icon: Option<&Path>,
    root: &Path,
    warnings: &mut Vec<String>,
) -> Result<(Option<LogoKind>, Option<Vec<u8>>, bool)> {
    match logo.map(str::trim) {
        Some(none) if none.eq_ignore_ascii_case("none") => Ok((None, None, true)),
        Some(logo) => {
            let path = existing_file(root, logo, "installer.logo")?;
            let kind = if has_extension(&path, "svg") {
                LogoKind::Svg
            } else if has_extension(&path, "png") {
                LogoKind::Png
            } else {
                bail!("installer.logo must be an .svg or a .png file (or \"none\"); {logo} is neither");
            };
            let bytes = std::fs::read(&path).with_context(|| format!("couldn't read {}", path.display()))?;
            if kind == LogoKind::Png && !bytes.starts_with(icon::PNG_SIGNATURE) {
                bail!("installer.logo is \"{logo}\", but it isn't a PNG image inside");
            }
            Ok((Some(kind), Some(bytes), false))
        }
        None => {
            let Some(icon) = icon else {
                return Ok((None, None, false));
            };
            let bytes = std::fs::read(icon).with_context(|| format!("couldn't read {}", icon.display()))?;
            match icon::largest_png(&bytes) {
                Some(png) => Ok((Some(LogoKind::Png), Some(png), false)),
                None => {
                    warnings.push(
                        "app.icon has no PNG-compressed image, so the window shows the Bach mark; \
                         set installer.logo to show your own"
                            .into(),
                    );
                    Ok((None, None, false))
                }
            }
        }
    }
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

#[cfg(test)]
mod tests;
