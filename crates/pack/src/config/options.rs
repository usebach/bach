//! `[installer]` choices, `[[components]]`, `[[protocols]]` and
//! `[[file_types]]`.

use std::collections::HashSet;

use anyhow::{bail, Result};
use serde::Deserialize;

use super::text::localized;
use super::InstallerSection;
use crate::manifest::features::{is_valid_component_id, is_valid_extension, is_valid_scheme};
use crate::manifest::{
    self, Choice, Component, FileEntry, FileType, Protocol, OPTION_AUTOSTART, OPTION_DESKTOP, OPTION_PATH,
    OPTION_START_MENU,
};
use crate::paths;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ComponentSection {
    id: String,
    title: toml::Value,
    description: Option<toml::Value>,
    files: Vec<String>,
    #[serde(default)]
    default: bool,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ProtocolSection {
    scheme: String,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FileTypeSection {
    extension: String,
    description: Option<String>,
}

pub(super) fn install(section: &InstallerSection, app: &manifest::App) -> Result<manifest::Install> {
    let folder = match &section.folder {
        Some(folder) => folder.trim().to_string(),
        None => app.id.clone(),
    };
    if !paths::is_valid_id(&folder) {
        bail!("installer.folder is \"{folder}\"; it must be a plain folder name, like \"{}\"", app.id);
    }
    let path_folder = match section.path_folder.as_deref().map(str::trim) {
        None | Some("") | Some(".") => None,
        Some(folder) => Some(paths::normalize(folder).map_err(|err| anyhow::anyhow!("installer.path_folder: {err}"))?),
    };
    Ok(manifest::Install {
        folder,
        desktop_shortcut: choice(section.desktop_shortcut.as_ref(), "installer.desktop_shortcut", Choice::On)?,
        start_menu_shortcut: choice(section.start_menu_shortcut.as_ref(), "installer.start_menu_shortcut", Choice::Always)?,
        start_with_windows: choice(section.start_with_windows.as_ref(), "installer.start_with_windows", Choice::Never)?,
        add_to_path: choice(section.add_to_path.as_ref(), "installer.add_to_path", Choice::Never)?,
        path_folder,
        change_folder: section.change_folder.unwrap_or(true),
        open_button: section.open_button.unwrap_or(true),
    })
}

/// `true` (offered, ticked), `false` (offered, unticked), `"always"` or
/// `"never"`.
fn choice(value: Option<&toml::Value>, key: &str, default: Choice) -> Result<Choice> {
    let parsed = match value {
        None => return Ok(default),
        Some(toml::Value::Boolean(true)) => Some(Choice::On),
        Some(toml::Value::Boolean(false)) => Some(Choice::Off),
        Some(toml::Value::String(word)) => match word.trim().to_ascii_lowercase().as_str() {
            "always" => Some(Choice::Always),
            "never" => Some(Choice::Never),
            _ => None,
        },
        Some(_) => None,
    };
    parsed.ok_or_else(|| {
        anyhow::anyhow!(
            "{key} is {}; use true (offered, ticked), false (offered, unticked), \"always\" or \"never\"",
            value.map(|value| value.to_string()).unwrap_or_default()
        )
    })
}

/// Checks the components and marks each file with the first component whose
/// patterns match it.
pub(super) fn components(
    sections: &[ComponentSection],
    files: &mut [FileEntry],
    app: &manifest::App,
    languages: &[String],
) -> Result<Vec<Component>> {
    let builtin = [OPTION_DESKTOP, OPTION_START_MENU, OPTION_AUTOSTART, OPTION_PATH];
    let mut seen = HashSet::new();
    let mut components = Vec::new();
    for (index, section) in sections.iter().enumerate() {
        let at = format!("components[{index}]");
        let id = section.id.trim();
        if !is_valid_component_id(id) {
            bail!("{at}.id is \"{id}\"; use lowercase letters, digits, - and _ (like \"samples\")");
        }
        if builtin.contains(&id) {
            bail!("{at}.id is \"{id}\", which Bach uses for its own option; pick another");
        }
        if !seen.insert(id.to_string()) {
            bail!("{at}.id \"{id}\" is used by another component too");
        }
        if section.files.is_empty() || section.files.iter().any(|pattern| pattern.trim().is_empty()) {
            bail!("{at}.files needs at least one pattern, like \"samples/**\"");
        }
        let mut matched = 0usize;
        for file in files.iter_mut().filter(|file| file.component.is_none()) {
            if section.files.iter().any(|pattern| paths::matches(pattern, &file.path)) {
                if file.path.eq_ignore_ascii_case(&app.executable) {
                    bail!("{at}.files includes app.executable ({}), which must always be installed", app.executable);
                }
                file.component = Some(id.to_string());
                matched += 1;
            }
        }
        if matched == 0 {
            bail!("{at}.files matches no files (that another component hasn't claimed first)");
        }
        components.push(Component {
            id: id.to_string(),
            title: localized(&section.title, &format!("{at}.title"), languages)?,
            description: section
                .description
                .as_ref()
                .map(|value| localized(value, &format!("{at}.description"), languages))
                .transpose()?,
            default: section.default,
        });
    }
    Ok(components)
}

pub(super) fn protocols(sections: &[ProtocolSection]) -> Result<Vec<Protocol>> {
    let mut seen = HashSet::new();
    sections
        .iter()
        .enumerate()
        .map(|(index, section)| {
            let scheme = section.scheme.trim().trim_end_matches("://").trim_end_matches(':').to_ascii_lowercase();
            if !is_valid_scheme(&scheme) {
                bail!(
                    "protocols[{index}].scheme is \"{}\"; use 2 to 32 lowercase letters, digits, +, - or ., \
                     starting with a letter, and not a scheme like http that belongs to Windows or browsers",
                    section.scheme
                );
            }
            if !seen.insert(scheme.clone()) {
                bail!("protocols[{index}].scheme \"{scheme}\" is listed twice");
            }
            Ok(Protocol { scheme, description: super::optional(&section.description) })
        })
        .collect()
}

pub(super) fn file_types(sections: &[FileTypeSection]) -> Result<Vec<FileType>> {
    let mut seen = HashSet::new();
    sections
        .iter()
        .enumerate()
        .map(|(index, section)| {
            let trimmed = section.extension.trim().to_ascii_lowercase();
            let extension = if trimmed.starts_with('.') { trimmed } else { format!(".{trimmed}") };
            if !is_valid_extension(&extension) {
                bail!(
                    "file_types[{index}].extension is \"{}\"; use one short extension like \".hello\", \
                     and not one that runs code (.exe, .bat, …)",
                    section.extension
                );
            }
            if !seen.insert(extension.clone()) {
                bail!("file_types[{index}].extension \"{extension}\" is listed twice");
            }
            Ok(FileType { extension, description: super::optional(&section.description) })
        })
        .collect()
}
