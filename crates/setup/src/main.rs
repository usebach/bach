//! `bach-setup`: the installer and uninstaller window every Bach setup file
//! runs. On its own it installs nothing; `bach build` appends an app's
//! files and manifest to a copy of it, and that copy is the setup file.
//! See `launch.rs` for how one executable plays every part.

// Release builds are a GUI app: no console window behind it.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

#[cfg(windows)]
mod actions;
#[cfg(windows)]
mod app;
#[cfg(windows)]
mod assets;
#[cfg(windows)]
mod keyboard;
#[cfg(windows)]
mod launch;
#[cfg(windows)]
mod log;
#[cfg(windows)]
mod package;
#[cfg(windows)]
mod pages;
#[cfg(windows)]
mod silent;
#[cfg(windows)]
mod start;
#[cfg(windows)]
mod system;
#[cfg(windows)]
mod texts;
#[cfg(windows)]
mod theme;
#[cfg(windows)]
mod ui;
#[cfg(windows)]
mod updates;
#[cfg(windows)]
mod view;
#[cfg(windows)]
mod work;

#[cfg(windows)]
fn main() {
    app::main();
}

#[cfg(not(windows))]
fn main() {
    eprintln!("bach-setup runs on Windows. Use `bach build` on any system to build installers for it.");
    std::process::exit(1);
}
