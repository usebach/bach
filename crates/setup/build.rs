//! Puts an icon and version details into the setup runtime.
//!
//! GPUI loads the window icon from the icon resource with id 1, and
//! `bach build` replaces that icon and the version block with the app's own
//! (crates/cli/src/resources.rs). Both are written in the neutral language
//! (`LANGUAGE 0, 0`), so the replacement lands on the same entries.
//! Other platforms embed nothing.

#[cfg(windows)]
use std::path::PathBuf;

fn main() {
    println!("cargo:rerun-if-changed=icon/bach.ico");
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    if target_os != "windows" {
        return;
    }
    #[cfg(windows)]
    embed();
}

#[cfg(windows)]
fn embed() {
    let manifest_dir = PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").expect("cargo sets CARGO_MANIFEST_DIR"));
    let out_dir = PathBuf::from(std::env::var("OUT_DIR").expect("cargo sets OUT_DIR"));
    let icon = manifest_dir.join("icon").join("bach.ico");
    let version = std::env::var("CARGO_PKG_VERSION").expect("cargo sets CARGO_PKG_VERSION");

    let script = resource_script(&icon.to_string_lossy(), &version);
    let rc = out_dir.join("bach-setup.rc");
    std::fs::write(&rc, script).expect("the resource script should be writable in OUT_DIR");
    embed_resource::compile(&rc, embed_resource::NONE)
        .manifest_required()
        .expect("the setup resources should compile with the Windows SDK's rc.exe");
}

/// The icon (id 1, which GPUI loads) and a VERSIONINFO block.
#[cfg(windows)]
fn resource_script(icon: &str, version: &str) -> String {
    // rc string literals use C escapes, so backslashes double.
    let icon = icon.replace('\\', "\\\\");
    let numbers: Vec<u16> = version
        .split(['.', '-', '+'])
        .take(3)
        .map(|part| part.parse().unwrap_or(0))
        .collect();
    let number = |index: usize| numbers.get(index).copied().unwrap_or(0);
    let binary = format!("{},{},{},0", number(0), number(1), number(2));
    format!(
        r#"LANGUAGE 0, 0

1 ICON "{icon}"

1 VERSIONINFO
FILEVERSION {binary}
PRODUCTVERSION {binary}
FILEOS 0x40004
FILETYPE 0x1
BEGIN
  BLOCK "StringFileInfo"
  BEGIN
    BLOCK "040904B0"
    BEGIN
      VALUE "CompanyName", "Bach"
      VALUE "FileDescription", "Bach Setup"
      VALUE "FileVersion", "{version}"
      VALUE "InternalName", "bach-setup"
      VALUE "LegalCopyright", "GPL-3.0-only"
      VALUE "OriginalFilename", "bach-setup.exe"
      VALUE "ProductName", "Bach"
      VALUE "ProductVersion", "{version}"
    END
  END
  BLOCK "VarFileInfo"
  BEGIN
    VALUE "Translation", 0x409, 1200
  END
END
"#
    )
}
