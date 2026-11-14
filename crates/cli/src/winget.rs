//! `bach winget`: the three manifests the Windows Package Manager needs to
//! list a setup file, ready for a pull request to microsoft/winget-pkgs.
//!
//! Bach's setup files are per-user `exe` installers with `--silent`, and
//! the Settings › Apps entry is named by the app id, which winget reads as
//! the product code to tell installed versions apart.

use std::path::Path;

use anyhow::{bail, Context, Result};
use bach_pack::container;
use bach_pack::manifest::Manifest;
use sha2::{Digest, Sha256};

const MANIFEST_VERSION: &str = "1.6.0";

pub struct Options<'a> {
    pub setup: &'a Path,
    pub url: &'a str,
    pub license: Option<&'a str>,
    pub out: Option<&'a Path>,
}

pub fn command(options: Options) -> Result<()> {
    if !bach_pack::manifest::extras::is_https(options.url) {
        bail!("--url must be the https link the setup file is downloaded from");
    }
    let mut file = std::fs::File::open(options.setup).with_context(|| format!("couldn't open {}", options.setup.display()))?;
    let Some(layout) = container::locate(&mut file)? else {
        bail!("{} isn't a Bach setup file", options.setup.display());
    };
    if !layout.has_archive() {
        bail!("{} is an uninstaller; point bach winget at the setup file", options.setup.display());
    }
    let manifest = container::read_manifest(&mut file, &layout)?;
    let bytes = std::fs::read(options.setup)?;
    let sha256 = format!("{:X}", Sha256::digest(&bytes));
    let id = package_id(&manifest);
    let version = &manifest.app.version;
    let dir = match options.out {
        Some(out) => out.to_path_buf(),
        None => options.setup.parent().unwrap_or(Path::new(".")).join("winget").join(&id).join(version),
    };
    std::fs::create_dir_all(&dir).with_context(|| format!("couldn't create {}", dir.display()))?;
    let files = manifests(&manifest, &id, options.url, &sha256, options.license);
    for (name, text) in &files {
        std::fs::write(dir.join(name), text).with_context(|| format!("couldn't write {name}"))?;
    }
    println!("Wrote the winget manifests for {id} {version} to {}", dir.display());
    if options.license.is_none() {
        crate::output::warnings(&["the manifests say License: Proprietary; pass --license with yours (like MIT)".into()]);
    }
    println!("Check them with `winget validate {}`, then open a pull request to microsoft/winget-pkgs.", dir.display());
    Ok(())
}

/// `Publisher.AppId` with only the characters winget allows.
fn package_id(manifest: &Manifest) -> String {
    let clean = |text: &str| text.chars().filter(|c| c.is_ascii_alphanumeric()).collect::<String>();
    let publisher = clean(&manifest.app.publisher);
    let app = clean(&manifest.app.id);
    format!("{}.{}", if publisher.is_empty() { "Publisher".into() } else { publisher }, app)
}

fn yaml(text: &str) -> String {
    format!("\"{}\"", text.replace('\\', "\\\\").replace('"', "\\\""))
}

fn manifests(manifest: &Manifest, id: &str, url: &str, sha256: &str, license: Option<&str>) -> Vec<(String, String)> {
    let app = &manifest.app;
    let version = &app.version;
    let header = |kind: &str| {
        format!(
            "# Created with Bach (bach winget).\n# yaml-language-server: $schema=https://aka.ms/winget-manifest.{kind}.{MANIFEST_VERSION}.schema.json\n\nPackageIdentifier: {id}\nPackageVersion: {}\n",
            yaml(version)
        )
    };
    let version_file = format!("{}DefaultLocale: en-US\nManifestType: version\nManifestVersion: {MANIFEST_VERSION}\n", header("version"));
    let installer = format!(
        "{}InstallerType: exe\nScope: user\nInstallModes:\n  - interactive\n  - silent\n  - silentWithProgress\nInstallerSwitches:\n  Silent: --silent\n  SilentWithProgress: --silent\n  InstallLocation: --dir \"<INSTALLPATH>\"\n  Log: --log \"<LOGPATH>\"\nUpgradeBehavior: install\nProductCode: {}\nInstallers:\n  - Architecture: x64\n    InstallerUrl: {url}\n    InstallerSha256: {sha256}\nManifestType: installer\nManifestVersion: {MANIFEST_VERSION}\n",
        header("installer"),
        yaml(&app.id)
    );
    let description = app.description.clone().unwrap_or_else(|| app.name.clone());
    let homepage = app.website.as_ref().map(|site| format!("PackageUrl: {site}\n")).unwrap_or_default();
    let locale = format!(
        "{}PackageLocale: en-US\nPublisher: {}\nPackageName: {}\n{homepage}License: {}\nShortDescription: {}\nManifestType: defaultLocale\nManifestVersion: {MANIFEST_VERSION}\n",
        header("defaultLocale"),
        yaml(&app.publisher),
        yaml(&app.name),
        yaml(license.unwrap_or("Proprietary")),
        yaml(&description)
    );
    vec![
        (format!("{id}.yaml"), version_file),
        (format!("{id}.installer.yaml"), installer),
        (format!("{id}.locale.en-US.yaml"), locale),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> Manifest {
        serde_json::from_str(
            r#"{"format":1,"builder":"t","app":{"id":"Hello App","name":"Hello","version":"1.2.0","publisher":"Hello Inc.",
            "description":"Says \"hi\".","website":"https://hello.example","executable":"hello.exe"},
            "install":{"folder":"Hello","desktop_shortcut":true},"uninstall":{"data":[],"data_description":null},
            "look":{"theme":"system","accent":null,"logo":null},"files":[{"path":"hello.exe","size":1}]}"#,
        )
        .unwrap()
    }

    #[test]
    fn manifests_name_the_package_and_installer() {
        let manifest = manifest();
        let id = package_id(&manifest);
        assert_eq!(id, "HelloInc.HelloApp");
        let files = manifests(&manifest, &id, "https://hello.example/Setup.exe", "AB", Some("MIT"));
        assert_eq!(files.len(), 3);
        let installer = &files[1].1;
        assert!(installer.contains("Silent: --silent") && installer.contains("Scope: user"));
        assert!(installer.contains("ProductCode: \"Hello App\""));
        assert!(files[2].1.contains("ShortDescription: \"Says \\\"hi\\\".\""));
    }
}
