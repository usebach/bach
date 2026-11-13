//! Finding the setup runtime (`bach-setup.exe`) every installer is built on.

use std::path::{Path, PathBuf};

use anyhow::{bail, Context, Result};

/// The runtime's file name, next to `bach.exe` in a release and in
/// `target\<profile>` after `cargo build`.
pub const FILE_NAME: &str = "bach-setup.exe";

/// The runtime to use: the one given (flag or `BACH_RUNTIME`), else the
/// one next to this `bach` executable.
pub fn find(given: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = given {
        if !path.is_file() {
            bail!("the setup runtime {} doesn't exist", path.display());
        }
        return Ok(path.to_path_buf());
    }
    let here = std::env::current_exe().context("couldn't find where bach is")?;
    let beside = here.parent().map(|dir| dir.join(FILE_NAME));
    match beside {
        Some(path) if path.is_file() => Ok(path),
        _ => bail!(
            "couldn't find {FILE_NAME} next to bach. Keep the two files together, \
             build it with `cargo build -p bach-setup --release`, or pass --runtime"
        ),
    }
}

/// The runtime's bytes, checked to be a bare runtime and not a setup file
/// that already carries a payload.
pub fn read(path: &Path) -> Result<Vec<u8>> {
    let mut file = std::fs::File::open(path).with_context(|| format!("couldn't open {}", path.display()))?;
    if bach_pack::container::locate(&mut file)?.is_some() {
        bail!("{} is already an installer; point --runtime at a plain bach-setup.exe", path.display());
    }
    let bytes = std::fs::read(path).with_context(|| format!("couldn't read {}", path.display()))?;
    if !bytes.starts_with(b"MZ") {
        bail!("{} isn't a Windows executable", path.display());
    }
    Ok(bytes)
}
