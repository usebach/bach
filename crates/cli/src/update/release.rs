//! Asking GitHub which Bach release is the newest, and which download of it
//! fits this computer.
//!
//! Only the tag and the names of the release's files are read from GitHub's
//! answer. The download address is built here from the tag and a file name
//! Bach made up itself, so an answer can't send Bach to another host.

use std::path::Path;

use anyhow::{bail, Context, Result};
use serde_json::Value;

use crate::runtime;
use crate::version::Version;

const API: &str = "https://api.github.com/repos/usebach/bach-installer/releases";
const DOWNLOADS: &str = "https://github.com/usebach/bach-installer/releases/download";
const PAGE: &str = "https://github.com/usebach/bach-installer/releases/tag";
const PAGE_LIST: &str = "https://github.com/usebach/bach-installer/releases";
const NPM_LATEST: &str = "https://registry.npmjs.org/@usebach/cli/latest";

/// One published release.
pub struct Release {
    pub version: Version,
    tag: String,
    assets: Vec<String>,
}

/// The download of a release that fits this computer.
pub struct Archive {
    pub name: String,
    pub url: String,
}

impl Release {
    /// The newest published release that isn't a pre-release or a draft.
    pub fn latest(seconds: Option<u32>) -> Result<Release> {
        parse(&get(&format!("{API}/latest"), seconds)?)
    }

    /// Exactly this version.
    pub fn tagged(version: &Version) -> Result<Release> {
        let url = format!("{API}/tags/v{version}");
        let body = get(&url, None).with_context(|| format!("Bach {version} isn't a published release (see {PAGE_LIST})"))?;
        parse(&body)
    }

    /// For `BACH_UPDATE_FROM`: the files in a folder stand in for a release.
    pub fn from_folder(dir: &Path, version: &Version) -> Result<Release> {
        let assets = std::fs::read_dir(dir)
            .with_context(|| format!("couldn't read BACH_UPDATE_FROM ({})", dir.display()))?
            .filter_map(|entry| entry.ok()?.file_name().into_string().ok())
            .collect();
        Ok(Release { version: version.clone(), tag: format!("v{version}"), assets })
    }

    /// Where the release's notes are.
    pub fn page(&self) -> String {
        format!("{PAGE}/{}", self.tag)
    }

    /// This computer's download. Windows on ARM takes the x64 build when the
    /// release has no ARM one: it runs there too.
    pub fn archive(&self) -> Result<Archive> {
        let candidates = candidates(&self.version, std::env::consts::OS, std::env::consts::ARCH);
        let Some(name) = candidates.iter().find(|name| self.assets.contains(name)) else {
            let wanted = candidates.first().map(String::as_str).unwrap_or("");
            if wanted.is_empty() {
                bail!("there's no prebuilt bach for this system; build it with: cargo install --git https://github.com/usebach/bach-installer bach-cli --locked --force");
            }
            bail!("Bach {} has no download for this system ({wanted}), so nothing was changed", self.version);
        };
        let url = format!("{DOWNLOADS}/{}/{name}", self.tag);
        Ok(Archive { name: name.clone(), url })
    }
}

/// The file names a release can have for this system, best first.
fn candidates(version: &Version, os: &str, arch: &str) -> Vec<String> {
    let arch = if matches!(arch, "aarch64" | "arm64") { "arm64" } else { "x64" };
    match os {
        "windows" => {
            let mut names = vec![format!("bach-{version}-windows-{arch}.zip")];
            if arch == "arm64" {
                names.push(format!("bach-{version}-windows-x64.zip"));
            }
            names
        }
        "macos" | "linux" => vec![format!("bach-{version}-{os}-{arch}.tar.gz")],
        _ => Vec::new(),
    }
}

/// The newest version of `@usebach/cli` on npm: what `npm install` would
/// give a person whose bach came from npm.
pub fn npm_latest(seconds: Option<u32>) -> Result<Version> {
    parse_npm(&get(NPM_LATEST, seconds)?)
}

/// The registry's answer for a package's `latest` tag has the version in `version`.
fn parse_npm(body: &str) -> Result<Version> {
    let json: Value = serde_json::from_str(body).context("npm's answer isn't readable")?;
    let text = json["version"].as_str().context("npm's answer has no version")?;
    Version::parse(text).with_context(|| format!("npm's version \"{text}\" isn't a version"))
}

/// GitHub's answer for one release: its tag and its files.
fn parse(body: &str) -> Result<Release> {
    let json: Value = serde_json::from_str(body).context("GitHub's answer isn't readable")?;
    if json["draft"].as_bool().unwrap_or(false) {
        bail!("that release is still a draft");
    }
    let tag = json["tag_name"].as_str().context("GitHub's answer has no release tag")?.to_string();
    let version = Version::parse(&tag)
        .filter(|version| format!("v{version}") == tag)
        .with_context(|| format!("the release tag \"{tag}\" isn't a version"))?;
    let assets = json["assets"]
        .as_array()
        .map(|assets| assets.iter().filter_map(|asset| asset["name"].as_str().map(str::to_string)).collect())
        .unwrap_or_default();
    Ok(Release { version, tag, assets })
}

/// The body of `url`, fetched like every other Bach download (https only,
/// curl or PowerShell) into the private cache folder and read back.
fn get(url: &str, seconds: Option<u32>) -> Result<String> {
    let root = runtime::cache_root()
        .context("there's no folder of yours to keep downloads in (LOCALAPPDATA, or XDG_CACHE_HOME and HOME, isn't set)")?;
    let dir = root.join("update");
    runtime::private_dir(&dir).with_context(|| format!("couldn't prepare {}", dir.display()))?;
    let file = dir.join(format!("release-{}.json", std::process::id()));
    let result = runtime::fetch_within(url, &file, seconds.or(Some(30)))
        .with_context(|| format!("couldn't reach GitHub ({url}); check your connection"))
        .and_then(|()| std::fs::read_to_string(&file).context("GitHub's answer isn't text"));
    let _ = std::fs::remove_file(&file);
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(text: &str) -> Version {
        Version::parse(text).unwrap()
    }

    #[test]
    fn a_release_is_its_tag_and_its_files() {
        let body = r#"{"tag_name":"v0.8.0","draft":false,"assets":[{"name":"bach-0.8.0-windows-x64.zip","browser_download_url":"https://evil.example/x"}]}"#;
        let release = parse(body).unwrap();
        assert_eq!(release.version, v("0.8.0"));
        let candidate = candidates(&release.version, "windows", "x86_64");
        assert_eq!(candidate, ["bach-0.8.0-windows-x64.zip"]);
        // The address comes from the tag, never from the answer.
        assert_eq!(
            format!("{DOWNLOADS}/{}/{}", release.tag, candidate[0]),
            "https://github.com/usebach/bach-installer/releases/download/v0.8.0/bach-0.8.0-windows-x64.zip"
        );
        assert_eq!(release.page(), "https://github.com/usebach/bach-installer/releases/tag/v0.8.0");
    }

    #[test]
    fn npms_answer_is_its_version() {
        assert_eq!(parse_npm(r#"{"name":"@usebach/cli","version":"0.10.0"}"#).unwrap(), v("0.10.0"));
        for body in ["nope", "{}", r#"{"version":"latest"}"#] {
            assert!(parse_npm(body).is_err(), "{body}");
        }
    }

    #[test]
    fn odd_answers_are_refused() {
        for body in [
            "not json",
            "{}",
            r#"{"tag_name":"latest"}"#,
            r#"{"tag_name":"v1.2"}"#,
            r#"{"tag_name":"v0.8.0","draft":true}"#,
            r#"{"tag_name":"v01.2.3"}"#,
        ] {
            assert!(parse(body).is_err(), "{body}");
        }
    }

    #[test]
    fn every_system_has_its_own_file() {
        let version = v("0.8.0");
        assert_eq!(candidates(&version, "windows", "aarch64"), ["bach-0.8.0-windows-arm64.zip", "bach-0.8.0-windows-x64.zip"]);
        assert_eq!(candidates(&version, "linux", "aarch64"), ["bach-0.8.0-linux-arm64.tar.gz"]);
        assert_eq!(candidates(&version, "macos", "x86_64"), ["bach-0.8.0-macos-x64.tar.gz"]);
        assert!(candidates(&version, "freebsd", "x86_64").is_empty());
    }

    #[test]
    fn a_release_without_this_systems_file_says_so() {
        let release = Release { version: v("0.8.0"), tag: "v0.8.0".into(), assets: vec!["bach-0.8.0-other.zip".into()] };
        let error = release.archive().map(|_| ()).unwrap_err().to_string();
        assert!(error.contains("nothing was changed") || error.contains("no prebuilt"), "{error}");
    }
}
