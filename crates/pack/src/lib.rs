//! What the Bach builder and the setup runtime agree on.
//!
//! - [`config`]: `bach.toml`, read and checked, with the files it names.
//! - [`manifest`]: the description of an app that travels inside a setup
//!   file (name, version, files, look).
//! - [`archive`]: the compressed stream of files.
//! - [`container`]: how the archive, the logo and the manifest are appended
//!   to the setup runtime's executable, and found again.
//! - [`paths`]: the rules every packaged path and data folder must follow.
//!
//! No UI and no Windows calls live here, so the builder runs on any OS and
//! the format is tested in isolation.

pub mod archive;
pub mod config;
pub mod container;
pub mod manifest;
pub mod paths;

/// The payload format this build writes and reads. A setup runtime refuses
/// a payload with another number instead of misreading it.
pub const FORMAT_VERSION: u32 = 1;

/// This crate's version, recorded in every manifest as the builder version.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");
