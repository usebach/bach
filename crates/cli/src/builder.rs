//! `bach check`, `bach build` and `bach preview`.

use std::fs::{self, File};
use std::io::{BufWriter, Seek, SeekFrom};
use std::path::{Path, PathBuf};
use std::time::Instant;

use anyhow::{bail, Context, Result};
use bach_pack::archive::{self, Compression};
use bach_pack::config::{self, Project};
use bach_pack::container;
use sha2::{Digest, Sha256};

use crate::output::{self, field, Progress};
use crate::runtime;

pub fn check(config_path: &Path) -> Result<()> {
    let project = config::load(config_path)?;
    let manifest = &project.manifest;
    println!("{} {} is ready to build.", manifest.app.name, manifest.app.version);
    summary(&project);
    println!();
    for file in &manifest.files {
        let component = file.component.as_deref().map(|id| format!("  ({id})")).unwrap_or_default();
        println!("  {:>10}  {}{component}", output::size(file.size), file.path);
    }
    output::warnings(&project.warnings);
    Ok(())
}

pub fn build(config_path: &Path, out: Option<&Path>, runtime: Option<&Path>, fast: bool) -> Result<PathBuf> {
    let started = Instant::now();
    let mut project = config::load(config_path)?;
    if fast {
        project.compression = Compression::Fast;
    }
    let output = out.map(Path::to_path_buf).unwrap_or_else(|| project.output.clone());
    let runtime = runtime::find(runtime)?;
    let app = &project.manifest.app;
    println!("Building {} {}", app.name, app.version);
    summary(&project);

    output::warnings(&project.warnings);

    let output = tidy(&output);
    let packed = write_setup(&project, &runtime, &output)?;

    let size = fs::metadata(&output).map(|meta| meta.len()).unwrap_or(0);
    let installed = project.manifest.total_size().max(1);
    println!();
    println!("Wrote {} ({})", output.display(), output::size(size));
    if project.manifest.updates.is_some() {
        crate::feed::after_build(&project, &output)?;
    }
    field(
        "Packed",
        format!(
            "{} of {} ({:.0}%), in {:.1}s",
            output::size(packed),
            output::size(installed),
            packed as f64 / installed as f64 * 100.0,
            started.elapsed().as_secs_f32()
        ),
    );
    Ok(output)
}

pub struct PreviewOptions<'a> {
    pub uninstall: bool,
    pub theme: Option<&'a str>,
    pub lang: Option<&'a str>,
    pub runtime: Option<&'a Path>,
}

pub fn preview(config_path: &Path, options: PreviewOptions) -> Result<()> {
    let PreviewOptions { uninstall, theme, lang, runtime } = options;
    let mut project = config::load(config_path)?;
    project.compression = Compression::Fast;
    if let Some(theme) = theme {
        if bach_pack::manifest::Theme::parse(theme).is_none() {
            bail!("--theme is \"{theme}\"; use system, light or dark");
        }
    }
    if let Some(lang) = lang {
        if bach_pack::i18n::language(lang).is_none() {
            let known: Vec<&str> = bach_pack::i18n::LANGUAGES.iter().map(|language| language.code).collect();
            bail!("--lang is \"{lang}\"; use one of {}", known.join(", "));
        }
    }
    if !cfg!(windows) {
        bail!("bach preview opens a Windows window, so it needs Windows; `bach check` works everywhere");
    }
    output::warnings(&project.warnings);
    let runtime = runtime::find(runtime)?;
    let dir = std::env::temp_dir().join("bach-preview");
    fs::create_dir_all(&dir)?;
    // Earlier previews go, except the ones still open; each preview gets a
    // file of its own, so an open window never blocks the next one.
    if let Ok(entries) = fs::read_dir(&dir) {
        for entry in entries.flatten() {
            let _ = fs::remove_file(entry.path());
        }
    }
    let output = dir.join(format!(
        "{}-Setup-preview-{}.exe",
        project.manifest.app.id.replace(' ', "-"),
        std::process::id()
    ));
    write_setup(&project, &runtime, &output)?;

    let mut command = std::process::Command::new(&output);
    command.arg("--preview");
    if uninstall {
        command.arg("--uninstall");
    }
    if let Some(theme) = theme {
        command.args(["--theme", theme]);
    }
    if let Some(lang) = lang {
        command.args(["--lang", lang]);
    }
    command.spawn().with_context(|| format!("couldn't start {}", output.display()))?;
    println!("Opened the {} preview. Nothing is installed or removed.", if uninstall { "uninstaller" } else { "installer" });
    Ok(())
}

fn summary(project: &Project) {
    crate::inspect::details(&project.manifest);
}

/// `examples/hello/../../dist/x.exe` becomes `dist/x.exe`, without touching
/// the disk (the file may not exist yet).
fn tidy(path: &Path) -> PathBuf {
    use std::path::Component;
    let mut out = PathBuf::new();
    for part in path.components() {
        match part {
            Component::CurDir => {}
            Component::ParentDir if matches!(out.components().next_back(), Some(Component::Normal(_))) => {
                out.pop();
            }
            other => out.push(other.as_os_str()),
        }
    }
    if out.as_os_str().is_empty() {
        PathBuf::from(".")
    } else {
        out
    }
}

/// Writes the setup file through a `.partial` file, so a failed build never
/// leaves a half-written installer where the real one belongs. Returns the
/// archive's packed size.
fn write_setup(project: &Project, runtime_path: &Path, output: &Path) -> Result<u64> {
    let runtime = runtime::read(runtime_path)?;
    if let Some(dir) = output.parent().filter(|dir| !dir.as_os_str().is_empty()) {
        fs::create_dir_all(dir).with_context(|| format!("couldn't create {}", dir.display()))?;
    }
    let mut partial = output.as_os_str().to_owned();
    partial.push(".partial");
    let partial = PathBuf::from(partial);

    let packed = match write_into(project, &runtime, &partial) {
        Ok(packed) => packed,
        Err(err) => {
            let _ = fs::remove_file(&partial);
            return Err(err);
        }
    };
    if output.exists() {
        fs::remove_file(output).with_context(|| format!("couldn't replace {} (is it open?)", output.display()))?;
    }
    fs::rename(&partial, output).with_context(|| format!("couldn't move the setup file to {}", output.display()))?;
    Ok(packed)
}

fn write_into(project: &Project, runtime: &[u8], path: &Path) -> Result<u64> {
    fs::write(path, runtime).with_context(|| format!("couldn't write {}", path.display()))?;
    brand(project, path)?;

    let mut file = File::options().read(true).write(true).open(path)?;
    let archive_offset = file.seek(SeekFrom::End(0))?;
    let total = project.manifest.total_size();
    let mut progress = Progress::new("Packing", total);
    let hashing = Hashing { inner: BufWriter::new(file), hasher: Sha256::new() };
    let hashing = archive::write(hashing, &project.sources, project.compression, &mut |done| progress.update(done))?;
    progress.finish();
    let Hashing { inner, hasher } = hashing;
    let mut file = inner.into_inner().map_err(|err| err.into_error())?;
    let logo = project.logo.as_deref().unwrap_or_default();
    // The runtime checks this before unpacking anything.
    let mut manifest = project.manifest.clone();
    manifest.archive_sha256 = Some(format!("{:x}", hasher.finalize()));
    let layout = container::finish(&mut file, archive_offset, logo, &manifest)?;
    file.sync_all()?;
    Ok(layout.archive.len)
}

/// A writer that hashes what passes through it.
struct Hashing<W> {
    inner: W,
    hasher: Sha256,
}

impl<W: std::io::Write> std::io::Write for Hashing<W> {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        let written = self.inner.write(bytes)?;
        self.hasher.update(&bytes[..written]);
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.inner.flush()
    }
}

/// Gives the setup file the app's icon and version details, so Explorer,
/// the taskbar and the file's Properties name the app, not Bach.
#[cfg(windows)]
fn brand(project: &Project, path: &Path) -> Result<()> {
    let icon = match &project.icon {
        Some(icon) => Some(fs::read(icon).with_context(|| format!("couldn't read {}", icon.display()))?),
        None => None,
    };
    let file_name = path
        .file_name()
        .map(|name| name.to_string_lossy().trim_end_matches(".partial").to_string())
        .unwrap_or_default();
    crate::resources::apply(path, icon.as_deref(), &project.manifest.app, &file_name)
}

#[cfg(not(windows))]
fn brand(project: &Project, _path: &Path) -> Result<()> {
    eprintln!(
        "  note: the setup file keeps Bach's icon and version details{}; \
         build on Windows to brand it",
        if project.icon.is_some() { " (app.icon is skipped)" } else { "" }
    );
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn output_paths_are_tidied() {
        let tidied = tidy(Path::new("examples/hello/../../dist/x.exe"));
        assert_eq!(tidied, Path::new("dist").join("x.exe"));
        assert_eq!(tidy(Path::new("../out/x.exe")), Path::new("..").join("out").join("x.exe"));
    }
}
