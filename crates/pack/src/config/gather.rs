//! Collecting the files under `[files] source`.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::archive::Source;
use crate::paths;

/// Every file under `dir`, sorted, minus the excluded ones and the output.
pub fn gather(dir: &Path, exclude: &[String], output: &Path) -> Result<Vec<Source>> {
    let output = std::fs::canonicalize(output).ok();
    let mut found = BTreeMap::new();
    walk(dir, "", &mut |packaged, path, size| {
        if exclude.iter().any(|pattern| paths::matches(pattern, packaged)) {
            return Ok(());
        }
        if output.is_some() && std::fs::canonicalize(path).ok() == output {
            return Ok(());
        }
        let key = packaged.to_lowercase();
        if found.contains_key(&key) {
            bail!("{} is in the source folder twice, in different letter case", packaged);
        }
        found.insert(key, Source { packaged: packaged.to_string(), path: path.to_path_buf(), size });
        Ok(())
    })?;
    Ok(found.into_values().collect())
}

fn walk(dir: &Path, prefix: &str, visit: &mut dyn FnMut(&str, &Path, u64) -> Result<()>) -> Result<()> {
    let entries = std::fs::read_dir(dir).with_context(|| format!("couldn't list {}", dir.display()))?;
    for entry in entries {
        let entry = entry?;
        let name = entry.file_name();
        let Some(name) = name.to_str() else {
            bail!("{} has a name that isn't valid text", entry.path().display());
        };
        let packaged = if prefix.is_empty() { name.to_string() } else { format!("{prefix}/{name}") };
        let path = entry.path();
        let meta = std::fs::symlink_metadata(&path)?;
        if meta.file_type().is_symlink() {
            bail!("{} is a symbolic link; Bach packages real files only", path.display());
        }
        if meta.is_dir() {
            walk(&path, &packaged, visit)?;
        } else if meta.is_file() {
            paths::normalize(&packaged).with_context(|| format!("can't package {}", path.display()))?;
            visit(&packaged, &path, meta.len())?;
        }
    }
    Ok(())
}
