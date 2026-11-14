//! `[[requirements]]`, `[[shortcuts]]` and `[updates]`.

use anyhow::{bail, Result};
use serde::Deserialize;

use crate::manifest::extras::{is_https, is_valid_shortcut_name};
use crate::manifest::{Check, FileEntry, Requirement, Shortcut, Updates};
use crate::paths;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RequirementSection {
    kind: String,
    version: Option<String>,
    #[serde(default)]
    desktop: bool,
    path: Option<String>,
    key: Option<String>,
    name: Option<String>,
    url: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ShortcutSection {
    name: String,
    target: String,
    arguments: Option<String>,
    description: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UpdatesSection {
    feed: String,
    /// Where the setup file will be downloaded from; `{file}` is its name.
    download_url: Option<String>,
}

pub fn requirements(sections: &[RequirementSection]) -> Result<Vec<Requirement>> {
    sections
        .iter()
        .enumerate()
        .map(|(index, section)| {
            let at = format!("requirements[{index}]");
            let check = match section.kind.trim().to_ascii_lowercase().as_str() {
                "webview2" => Check::Webview2,
                "vcredist" => Check::Vcredist,
                "dotnet" => {
                    let version = section.version.as_deref().map(str::trim).unwrap_or_default();
                    let valid = version.split('.').count() == 2 && version.split('.').all(|part| part.parse::<u32>().is_ok());
                    if !valid {
                        bail!("{at}.version is \"{version}\"; give the .NET version as major.minor, like \"8.0\"");
                    }
                    Check::Dotnet { version: version.to_string(), desktop: section.desktop }
                }
                "file" => {
                    let path = section.path.as_deref().map(str::trim).unwrap_or_default();
                    if path.is_empty() {
                        bail!("{at}.path is missing; give the file to look for, like '%SystemRoot%\\System32\\x.dll'");
                    }
                    Check::File { path: path.to_string() }
                }
                "registry" => {
                    let key = section.key.as_deref().map(str::trim).unwrap_or_default();
                    let upper = key.to_ascii_uppercase();
                    if !(upper.starts_with("HKLM\\") || upper.starts_with("HKCU\\")) {
                        bail!("{at}.key is \"{key}\"; it must start with HKLM\\ or HKCU\\");
                    }
                    Check::Registry { key: key.to_string() }
                }
                other => bail!("{at}.kind is \"{other}\"; use \"webview2\", \"dotnet\", \"vcredist\", \"file\" or \"registry\""),
            };
            let defaults = check.defaults();
            let name = section.name.clone().or_else(|| defaults.map(|(name, _)| name.to_string()));
            let Some(name) = name.filter(|name| !name.trim().is_empty()) else {
                bail!("{at}.name is missing; say what it is, like \"Hello Runtime\"");
            };
            let url = section.url.clone().or_else(|| defaults.map(|(_, url)| url.to_string()));
            let Some(url) = url.filter(|url| is_https(url)) else {
                bail!("{at}.url must be an https link where people can get {name}");
            };
            Ok(Requirement { check, name: name.trim().to_string(), url })
        })
        .collect()
}

pub fn shortcuts(sections: &[ShortcutSection], files: &[FileEntry]) -> Result<Vec<Shortcut>> {
    sections
        .iter()
        .enumerate()
        .map(|(index, section)| {
            let at = format!("shortcuts[{index}]");
            if !is_valid_shortcut_name(&section.name) {
                bail!("{at}.name is \"{}\"; it names a file, so leave out < > : \" / \\ | ? *", section.name);
            }
            let target = paths::normalize(section.target.trim()).map_err(|err| anyhow::anyhow!("{at}.target: {err}"))?;
            if !files.iter().any(|file| file.path.eq_ignore_ascii_case(&target)) {
                bail!("{at}.target is \"{target}\", but no installed file has that path");
            }
            Ok(Shortcut {
                name: section.name.trim().to_string(),
                target,
                arguments: super::optional(&section.arguments),
                description: super::optional(&section.description),
            })
        })
        .collect()
}

/// The feed, and the download URL template for the JSON `bach build` writes.
pub fn updates(section: Option<&UpdatesSection>) -> Result<(Option<Updates>, Option<String>)> {
    let Some(section) = section else {
        return Ok((None, None));
    };
    if !is_https(section.feed.trim()) {
        bail!("updates.feed is \"{}\"; it must be an https link", section.feed);
    }
    let download = section.download_url.as_deref().map(str::trim);
    if let Some(download) = download {
        if !is_https(download) {
            bail!("updates.download_url is \"{download}\"; it must be an https link");
        }
    }
    Ok((Some(Updates { feed: section.feed.trim().to_string() }), download.map(String::from)))
}

pub fn https(value: &Option<String>, key: &str) -> Result<Option<String>> {
    match value.as_deref().map(str::trim) {
        None | Some("") => Ok(None),
        Some(url) if is_https(url) => Ok(Some(url.to_string())),
        Some(url) => bail!("{key} is \"{url}\"; it must be an https link"),
    }
}

/// A PNG's width divided by its height, read from its header.
pub fn png_aspect(png: &[u8]) -> Option<f32> {
    let width = u32::from_be_bytes(png.get(16..20)?.try_into().ok()?);
    let height = u32::from_be_bytes(png.get(20..24)?.try_into().ok()?);
    (width > 0 && height > 0).then(|| width as f32 / height as f32)
}
