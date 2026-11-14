//! Checking a manifest read from a setup file, which is untrusted input.

use std::collections::HashSet;

use anyhow::{bail, Context, Result};

use super::extras::{is_https, is_valid_shortcut_name};
use super::features::{is_valid_component_id, is_valid_extension, is_valid_scheme};
use super::{Manifest, OPTION_AUTOSTART, OPTION_DESKTOP, OPTION_PATH, OPTION_START_MENU, UNINSTALLER};
use crate::{i18n, paths};

/// A license longer than this is damage, not a license.
pub const MAX_LICENSE: usize = 1 << 20;

pub fn manifest(manifest: &Manifest) -> Result<()> {
    if manifest.format != crate::FORMAT_VERSION {
        bail!(
            "this setup file uses payload format {}, but this runtime reads format {}",
            manifest.format,
            crate::FORMAT_VERSION
        );
    }
    if !paths::is_valid_id(&manifest.app.id) {
        bail!("the app id \"{}\" can't name a folder", manifest.app.id);
    }
    if !paths::is_valid_id(&manifest.install.folder) {
        bail!("the install folder \"{}\" isn't a plain folder name", manifest.install.folder);
    }
    if manifest.app.name.trim().is_empty() || manifest.app.version.trim().is_empty() {
        bail!("the app has no name or version");
    }
    paths::normalize(&manifest.app.executable).context("the app's executable")?;
    if let Some(folder) = &manifest.install.path_folder {
        if &paths::normalize(folder)? != folder {
            bail!("the PATH folder \"{folder}\" isn't stored in its canonical form");
        }
    }
    files(manifest)?;
    components(manifest)?;
    for protocol in &manifest.protocols {
        if !is_valid_scheme(&protocol.scheme) {
            bail!("\"{}\" can't be registered as a link scheme", protocol.scheme);
        }
    }
    for file_type in &manifest.file_types {
        if !is_valid_extension(&file_type.extension) {
            bail!("\"{}\" can't be registered as a file type", file_type.extension);
        }
    }
    for folder in &manifest.uninstall.data {
        paths::check_data_folder(folder)?;
    }
    extras(manifest)?;
    locale(manifest)
}

fn extras(manifest: &Manifest) -> Result<()> {
    for requirement in &manifest.requirements {
        if !is_https(&requirement.url) || requirement.name.trim().is_empty() {
            bail!("the requirement \"{}\" has no name or no https link", requirement.name);
        }
    }
    for shortcut in &manifest.shortcuts {
        if !is_valid_shortcut_name(&shortcut.name) {
            bail!("\"{}\" can't name a shortcut", shortcut.name);
        }
        if paths::normalize(&shortcut.target)? != shortcut.target {
            bail!("the shortcut \"{}\" points at \"{}\", which isn't a canonical path", shortcut.name, shortcut.target);
        }
    }
    if let Some(folder) = &manifest.install.start_menu_folder {
        if !is_valid_shortcut_name(folder) {
            bail!("\"{folder}\" can't name a Start menu folder");
        }
    }
    if let Some(updates) = &manifest.updates {
        if !is_https(&updates.feed) {
            bail!("the update feed \"{}\" isn't an https link", updates.feed);
        }
    }
    if let Some(url) = &manifest.uninstall.feedback_url {
        if !is_https(url) {
            bail!("the feedback link \"{url}\" isn't an https link");
        }
    }
    if let Some(hash) = &manifest.archive_sha256 {
        if hash.len() != 64 || !hash.chars().all(|c| c.is_ascii_hexdigit()) {
            bail!("the setup file's checksum is damaged");
        }
    }
    Ok(())
}

fn files(manifest: &Manifest) -> Result<()> {
    let mut seen = HashSet::new();
    for file in &manifest.files {
        let normal = paths::normalize(&file.path)?;
        if normal != file.path {
            bail!("\"{}\" isn't stored in its canonical form", file.path);
        }
        if normal.eq_ignore_ascii_case(UNINSTALLER) {
            bail!("the app ships its own {UNINSTALLER}, which would replace Bach's uninstaller");
        }
        if !seen.insert(normal.to_lowercase()) {
            bail!("\"{}\" is listed twice", file.path);
        }
        if let Some(component) = &file.component {
            if !manifest.components.iter().any(|known| &known.id == component) {
                bail!("\"{}\" belongs to a component, \"{component}\", that doesn't exist", file.path);
            }
        }
    }
    Ok(())
}

fn components(manifest: &Manifest) -> Result<()> {
    let builtin = [OPTION_DESKTOP, OPTION_START_MENU, OPTION_AUTOSTART, OPTION_PATH];
    let mut seen = HashSet::new();
    for component in &manifest.components {
        if !is_valid_component_id(&component.id) || builtin.contains(&component.id.as_str()) {
            bail!("\"{}\" can't name a component", component.id);
        }
        if !seen.insert(component.id.as_str()) {
            bail!("the component \"{}\" is listed twice", component.id);
        }
        if component.title.is_empty() {
            bail!("the component \"{}\" has no title", component.id);
        }
    }
    Ok(())
}

fn locale(manifest: &Manifest) -> Result<()> {
    for code in &manifest.locale.languages {
        if i18n::language(code).is_none() {
            bail!("the setup file asks for a language Bach doesn't speak, \"{code}\"");
        }
    }
    for key in manifest.locale.text.keys() {
        if i18n::Key::parse(key).is_none() {
            bail!("the setup file replaces a text that doesn't exist, \"{key}\"");
        }
    }
    if let Some(license) = &manifest.locale.license {
        if license.values().any(|text| text.len() > MAX_LICENSE) {
            bail!("the license is longer than {} KB", MAX_LICENSE / 1024);
        }
    }
    Ok(())
}
