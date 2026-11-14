//! The update feed: the small JSON an installed app reads to learn about a
//! newer version (`uninstall.exe --update`).
//!
//! `bach build` writes `latest.json` next to the setup file when the config
//! has `[updates]` with a `download_url`. Signing changes the file and so
//! its SHA-256: after signing, write the feed again with `bach feed`.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};
use bach_pack::config::Project;
use bach_pack::container;
use bach_pack::manifest::FeedEntry;
use sha2::{Digest, Sha256};

use crate::output::field;

pub const FILE_NAME: &str = "latest.json";

/// After a build: the feed, when the config says where the file will live.
pub fn after_build(project: &Project, setup: &Path) -> Result<()> {
    match &project.update_download_url {
        Some(template) => {
            let written = write(setup, &download_url(template, setup), None, None)?;
            field("Update feed", written.display());
        }
        None => crate::output::warnings(&["[updates] has no download_url, so no latest.json was written; \
             after uploading the setup file, run `bach feed SETUP --url URL`"
            .to_string()]),
    }
    Ok(())
}

/// `bach feed`: the feed for a (possibly signed) setup file.
pub fn command(setup: &Path, url: Option<&str>, notes: Option<&str>, out: Option<&Path>) -> Result<()> {
    let url = match url {
        Some(url) => url.to_string(),
        None => bail!("pass --url with the https link the setup file will be downloaded from"),
    };
    let written = write(setup, &url, notes, out)?;
    println!("Wrote {}", written.display());
    Ok(())
}

/// `https://example.com/{file}` with the setup file's name, spaces encoded.
fn download_url(template: &str, setup: &Path) -> String {
    let name = setup.file_name().map(|name| name.to_string_lossy().into_owned()).unwrap_or_default();
    template.replace("{file}", &name.replace(' ', "%20"))
}

fn write(setup: &Path, url: &str, notes: Option<&str>, out: Option<&Path>) -> Result<PathBuf> {
    if !bach_pack::manifest::extras::is_https(url) {
        bail!("the download link \"{url}\" must be an https link");
    }
    let mut file = std::fs::File::open(setup).with_context(|| format!("couldn't open {}", setup.display()))?;
    let Some(layout) = container::locate(&mut file)? else {
        bail!("{} isn't a Bach setup file", setup.display());
    };
    let manifest = container::read_manifest(&mut file, &layout)?;
    if !layout.has_archive() {
        bail!("{} is an uninstaller; point bach feed at the setup file", setup.display());
    }
    let bytes = std::fs::read(setup).with_context(|| format!("couldn't read {}", setup.display()))?;
    let entry = FeedEntry {
        version: manifest.app.version.clone(),
        url: url.to_string(),
        sha256: format!("{:x}", Sha256::digest(&bytes)),
        notes: notes.map(String::from),
        size: Some(bytes.len() as u64),
    };
    let target = match out {
        Some(out) => out.to_path_buf(),
        None => setup.parent().unwrap_or(Path::new(".")).join(FILE_NAME),
    };
    let json = serde_json::to_string_pretty(&entry)?;
    std::fs::write(&target, format!("{json}\n")).with_context(|| format!("couldn't write {}", target.display()))?;
    Ok(target)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn download_urls_name_the_file() {
        let url = download_url("https://example.com/downloads/{file}", Path::new("dist/My App-Setup-1.0.exe"));
        assert_eq!(url, "https://example.com/downloads/My%20App-Setup-1.0.exe");
    }
}
