//! `[[requirements]]`, `[[shortcuts]]` and `[updates]`.

use anyhow::{bail, Result};
use serde::Deserialize;

use crate::manifest::extras::{is_https, is_valid_shortcut_name};
use crate::manifest::{Check, Choice, FileEntry, Requirement, Shortcut, Updates};
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
    /// The update feed's https link. Not needed with `github`.
    feed: Option<String>,
    /// `owner/repo`: updates come from that repository's GitHub releases.
    github: Option<String>,
    /// Where the setup file will be downloaded from; `{file}` is its name
    /// and `{version}` the app's version.
    download_url: Option<String>,
    /// Keep the app up to date in the background: `true` (offered, ticked,
    /// the default), `false` (offered, unticked), `"always"` or `"never"`.
    auto: Option<toml::Value>,
}

/// What `[updates]` becomes: the manifest part, and the build-time part.
pub struct UpdateSettings {
    pub updates: Updates,
    /// The download link template for the feed `bach build` writes.
    pub download_url: Option<String>,
    /// The GitHub repository, when releases are the home of the updates.
    pub github: Option<String>,
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

/// The feed and the download link template for the JSON `bach build` writes.
///
/// With `github = "owner/repo"` both have a default that fits GitHub
/// releases made by the Bach action: the feed is the latest release's
/// `latest.json`, and each version's setup file is downloaded from that
/// version's own tag (`v{version}`), so a link never changes what it serves.
pub fn updates(section: Option<&UpdatesSection>) -> Result<Option<UpdateSettings>> {
    let Some(section) = section else {
        return Ok(None);
    };
    let github = match section.github.as_deref().map(str::trim) {
        None | Some("") => None,
        Some(repo) if is_github_repo(repo) => Some(repo.to_string()),
        Some(repo) => bail!("updates.github is \"{repo}\"; use the repository as owner/name, like \"usebach/bach\""),
    };
    let feed = match (section.feed.as_deref().map(str::trim), &github) {
        (Some(feed), _) if !feed.is_empty() => {
            if !is_https(feed) {
                bail!("updates.feed is \"{feed}\"; it must be an https link");
            }
            feed.to_string()
        }
        (_, Some(repo)) => format!("https://github.com/{repo}/releases/latest/download/latest.json"),
        _ => bail!("[updates] needs github = \"owner/name\" or feed = \"https://…/latest.json\""),
    };
    let download = match (section.download_url.as_deref().map(str::trim), &github) {
        (Some(download), _) if !download.is_empty() => {
            if !is_https(download) {
                bail!("updates.download_url is \"{download}\"; it must be an https link");
            }
            Some(download.to_string())
        }
        (_, Some(repo)) => Some(format!("https://github.com/{repo}/releases/download/v{{version}}/{{file}}")),
        _ => None,
    };
    let auto = super::options::choice(section.auto.as_ref(), "updates.auto", Choice::On)?;
    Ok(Some(UpdateSettings { updates: Updates { feed, auto }, download_url: download, github }))
}

/// `owner/name`, as GitHub allows them.
fn is_github_repo(text: &str) -> bool {
    let valid = |part: &str| {
        !part.is_empty()
            && part.len() <= 100
            && part.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_' | '.'))
            && part != "."
            && part != ".."
    };
    text.split_once('/').is_some_and(|(owner, name)| valid(owner) && valid(name))
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
