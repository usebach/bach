//! `bach`: builds modern, clean Windows installers from a `bach.toml`.
//!
//! ```text
//! bach init              write a starter bach.toml
//! bach check             read bach.toml and list what would be installed
//! bach build             write the setup file
//! bach preview           open the installer window without installing anything
//! bach inspect FILE      show what a setup file installs
//! ```

mod builder;
mod inspect;
mod output;
#[cfg(windows)]
mod resources;
mod runtime;
mod template;

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
        /// The setup runtime to build on (default: bach-setup.exe next to bach).
        #[arg(long, env = "BACH_RUNTIME")]
        runtime: Option<PathBuf>,
    },
    /// Show what a setup file installs.
    Inspect {
        /// A setup file (or an installed uninstall.exe).
        file: PathBuf,
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
        Command::Preview { config, uninstall, theme, runtime } => {
            builder::preview(&config, uninstall, theme.as_deref(), runtime.as_deref())
        }
        Command::Inspect { file } => inspect::inspect(&file),
    };
    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            output::error(&err);
            ExitCode::FAILURE
        }
    }
}
