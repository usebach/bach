//! `bach`: builds modern, clean Windows installers from a `bach.toml`.
//!
//! ```text
//! bach init              write a starter bach.toml
//! bach check             read bach.toml and list what would be installed
//! bach build             write the setup file
//! bach preview           open the installer window without installing anything
//! bach inspect FILE      show what a setup file installs
//! bach doctor            check that everything a build needs is in place
//! bach feed FILE --url   write the update feed for a (signed) setup file
//! bach winget FILE --url write winget manifests for a setup file
//! ```

mod builder;
mod detect;
mod doctor;
mod feed;
mod inspect;
mod output;
#[cfg(windows)]
mod resources;
mod runtime;
mod template;
mod winget;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "bach", version, about = "Build modern, clean Windows installers from one bach.toml.")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Write a starter bach.toml in this folder (or the one given).
    Init {
        /// Where to write it.
        #[arg(default_value = ".")]
        dir: PathBuf,
    },
    /// Read a bach.toml and list what the installer would install.
    Check {
        /// The config file.
        #[arg(default_value = "bach.toml")]
        config: PathBuf,
    },
    /// Build the setup file.
    Build {
        /// The config file.
        #[arg(default_value = "bach.toml")]
        config: PathBuf,
        /// Write the setup file here instead of installer.output.
        #[arg(long, short)]
        out: Option<PathBuf>,
        /// The setup runtime to build on (default: bach-setup.exe next to bach).
        #[arg(long, env = "BACH_RUNTIME")]
        runtime: Option<PathBuf>,
        /// Compress quickly instead of as small as possible.
        #[arg(long)]
        fast: bool,
    },
    /// Open the installer window as it will look, without installing anything.
    Preview {
        /// The config file.
        #[arg(default_value = "bach.toml")]
        config: PathBuf,
        /// Show the uninstaller instead.
        #[arg(long)]
        uninstall: bool,
        /// Override the theme: system, light or dark.
        #[arg(long)]
        theme: Option<String>,
        /// Show it in this language (en, de, fr, …) instead of Windows' own.
        #[arg(long)]
        lang: Option<String>,
        /// The setup runtime to build on (default: bach-setup.exe next to bach).
        #[arg(long, env = "BACH_RUNTIME")]
        runtime: Option<PathBuf>,
    },
    /// Show what a setup file installs.
    Inspect {
        /// A setup file (or an installed uninstall.exe).
        file: PathBuf,
    },
    /// Write the update feed (latest.json) for a setup file, after signing it.
    Feed {
        /// The setup file.
        file: PathBuf,
        /// The https link it will be downloaded from.
        #[arg(long)]
        url: Option<String>,
        /// A line about what's new.
        #[arg(long)]
        notes: Option<String>,
        /// Write the feed here instead of latest.json next to the setup file.
        #[arg(long, short)]
        out: Option<PathBuf>,
    },
    /// Write the manifests that list a setup file in winget.
    Winget {
        /// The setup file.
        file: PathBuf,
        /// The https link it's downloaded from.
        #[arg(long)]
        url: String,
        /// The app's license, like MIT (default: Proprietary).
        #[arg(long)]
        license: Option<String>,
        /// Write the manifests into this folder.
        #[arg(long, short)]
        out: Option<PathBuf>,
    },
    /// Check that everything a build needs is in place.
    Doctor {
        /// The config file.
        #[arg(default_value = "bach.toml")]
        config: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();
    let result = match cli.command {
        Command::Init { dir } => template::init(&dir),
        Command::Check { config } => builder::check(&config),
        Command::Build { config, out, runtime, fast } => {
            builder::build(&config, out.as_deref(), runtime.as_deref(), fast).map(|_| ())
        }
        Command::Preview { config, uninstall, theme, lang, runtime } => builder::preview(
            &config,
            builder::PreviewOptions {
                uninstall,
                theme: theme.as_deref(),
                lang: lang.as_deref(),
                runtime: runtime.as_deref(),
            },
        ),
        Command::Inspect { file } => inspect::inspect(&file),
        Command::Doctor { config } => doctor::doctor(&config),
        Command::Winget { file, url, license, out } => winget::command(winget::Options {
            setup: &file,
            url: &url,
            license: license.as_deref(),
            out: out.as_deref(),
        }),
        Command::Feed { file, url, notes, out } => feed::command(&file, url.as_deref(), notes.as_deref(), out.as_deref()),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            output::error(&err);
            ExitCode::FAILURE
        }
    }
}
