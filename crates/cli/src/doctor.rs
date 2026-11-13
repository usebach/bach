//! `bach doctor`: whether everything a build needs is in place, in a few
//! lines a person or an agent can act on.

use std::path::Path;

use anyhow::Result;

use crate::output::field;
use crate::runtime::{self, Found};

pub fn doctor(config: &Path) -> Result<()> {
    println!(
        "Bach {} on {} ({})",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS,
        std::env::consts::ARCH
    );
    let mut problems = 0;
    match runtime::locate(None) {
        Ok(Found::Here(path)) => match runtime::read(&path) {
            Ok(_) => field("Runtime", path.display()),
            Err(err) => {
                problems += 1;
                field("Runtime", format!("{} is unusable: {err:#}", path.display()));
            }
        },
        Ok(Found::Downloadable(url)) => field("Runtime", format!("not here yet; the first build downloads it from {url}")),
        Err(err) => {
            problems += 1;
            field("Runtime", format!("{err:#}"));
        }
    }

    if config.is_file() {
        match bach_pack::config::load(config) {
            Ok(project) => {
                let manifest = &project.manifest;
                let warnings = match project.warnings.len() {
                    0 => String::new(),
                    1 => " (1 warning: run bach check)".into(),
                    count => format!(" ({count} warnings: run bach check)"),
                };
                field("Config", format!("{} is valid: {} {}{warnings}", config.display(), manifest.app.name, manifest.app.version));
            }
            Err(err) => {
                problems += 1;
                field("Config", format!("{err:#}"));
            }
        }
    } else {
        field("Config", format!("no {} here; `bach init` writes one", config.display()));
    }

    field("Preview", if cfg!(windows) { "available" } else { "needs Windows (bach check works here)" });
    field("Signing", if has_signtool() { "signtool found" } else { "signtool not found (optional, see docs/signing.md)" });
    println!();
    if problems == 0 {
        println!("Ready to build.");
    } else {
        println!("Fix the {} above, then run bach doctor again.", if problems == 1 { "problem" } else { "problems" });
    }
    Ok(())
}

fn has_signtool() -> bool {
    std::process::Command::new("signtool")
        .arg("/?")
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok()
}
