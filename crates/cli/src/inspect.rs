//! `bach inspect`: what a setup file (or an uninstaller) carries.

use std::path::Path;

use anyhow::{bail, Context, Result};
use bach_pack::container;

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
    field("Publisher", &app.publisher);
    field("App id", &app.id);
    field("Program", &app.executable);
    field("Files", format!("{} ({})", manifest.files.len(), output::size(manifest.total_size())));
    if layout.has_archive() {
        field("Packed", output::size(layout.archive.len));
    }
    field("Built by", format!("Bach {}", manifest.builder));
    if !manifest.uninstall.data.is_empty() {
        field("Data", manifest.uninstall.data.join(", "));
    }
    Ok(())
}
