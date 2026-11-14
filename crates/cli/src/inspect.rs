//! `bach inspect`: what a setup file (or an uninstaller) carries, and the
//! summary `bach check` and `bach build` print for a config.

use std::path::Path;

use anyhow::{bail, Context, Result};
use bach_pack::container;
use bach_pack::i18n::{self, ANY_LANGUAGE};
use bach_pack::manifest::{Choice, Manifest};

use crate::output::{self, field};

pub fn inspect(path: &Path) -> Result<()> {
    let mut file = std::fs::File::open(path).with_context(|| format!("couldn't open {}", path.display()))?;
    let Some(layout) = container::locate(&mut file)? else {
        bail!("{} isn't a Bach setup file", path.display());
    };
    let manifest = container::read_manifest(&mut file, &layout)?;
    let app = &manifest.app;
    let kind = if layout.has_archive() { "Installer" } else { "Uninstaller" };
    println!("{kind} for {} {}", app.name, app.version);
    field("App id", &app.id);
    details(&manifest);
    if layout.has_archive() {
        field("Packed", output::size(layout.archive.len));
    }
    if let Some(chosen) = &manifest.chosen {
        let mut picked = Vec::new();
        for (on, name) in [
            (chosen.desktop_shortcut, "desktop shortcut"),
            (chosen.start_menu_shortcut, "Start menu"),
            (chosen.start_with_windows, "start with Windows"),
            (chosen.add_to_path, "PATH"),
            (chosen.auto_update, "automatic updates"),
        ] {
            if on {
                picked.push(name.to_string());
            }
        }
        picked.extend(chosen.components.iter().cloned());
        field("Chosen", if picked.is_empty() { "nothing optional".into() } else { picked.join(", ") });
    }
    field("Built by", format!("Bach {}", manifest.builder));
    Ok(())
}

/// Publisher, program, files, folder, options, languages and the rest.
pub fn details(manifest: &Manifest) {
    let install = &manifest.install;
    field("Publisher", &manifest.app.publisher);
    field("Program", &manifest.app.executable);
    field("Files", format!("{} ({})", manifest.files.len(), output::size(manifest.total_size())));
    let folder = format!(r"%LOCALAPPDATA%\Programs\{}", install.folder);
    field("Installs to", if install.change_folder { folder } else { format!("{folder} (fixed)") });

    let mut options = Vec::new();
    for (choice, name) in [
        (install.desktop_shortcut, "desktop shortcut"),
        (install.start_menu_shortcut, "Start menu"),
        (install.start_with_windows, "start with Windows"),
        (install.add_to_path, "add to PATH"),
    ] {
        match choice {
            Choice::Never => {}
            Choice::Off => options.push(format!("{name} (offered)")),
            Choice::On => options.push(format!("{name} (offered, ticked)")),
            Choice::Always => options.push(name.to_string()),
        }
    }
    match bach_pack::manifest::choices_auto_update(manifest) {
        Choice::Never => {}
        Choice::Off => options.push("automatic updates (offered)".to_string()),
        Choice::On => options.push("automatic updates (offered, ticked)".to_string()),
        Choice::Always => options.push("automatic updates".to_string()),
    }
    field("Options", options.join(", "));
    for component in &manifest.components {
        let title = i18n::resolve(&component.title, "en", &manifest.locale.languages).unwrap_or(&component.id);
        let count = manifest.files.iter().filter(|file| file.component.as_ref() == Some(&component.id)).count();
        let ticked = if component.default { ", ticked" } else { "" };
        let files = if count == 1 { "file" } else { "files" };
        field("Component", format!("{} \"{title}\" ({count} {files}{ticked})", component.id));
    }
    for protocol in &manifest.protocols {
        field("Opens links", format!("{}://", protocol.scheme));
    }
    if !manifest.file_types.is_empty() {
        let types: Vec<&str> = manifest.file_types.iter().map(|file_type| file_type.extension.as_str()).collect();
        field("Opens files", types.join(", "));
    }
    let languages = if manifest.locale.languages.is_empty() {
        format!("all {} (follows Windows)", i18n::LANGUAGES.len())
    } else {
        manifest.locale.languages.join(", ")
    };
    field("Languages", languages);
    if let Some(license) = &manifest.locale.license {
        let codes: Vec<&str> = license.keys().map(|code| if code == ANY_LANGUAGE { "all languages" } else { code }).collect();
        field("License", format!("shown first ({})", codes.join(", ")));
    }
    if !manifest.locale.text.is_empty() {
        let keys: Vec<&str> = manifest.locale.text.keys().map(String::as_str).collect();
        field("Own texts", keys.join(", "));
    }
    for requirement in &manifest.requirements {
        field("Requires", &requirement.name);
    }
    for shortcut in &manifest.shortcuts {
        field("Shortcut", format!("{} → {}", shortcut.name, shortcut.target));
    }
    if let Some(updates) = &manifest.updates {
        field("Updates", &updates.feed);
    }
    if !manifest.uninstall.data.is_empty() {
        field("Data", manifest.uninstall.data.join(", "));
    }
}
