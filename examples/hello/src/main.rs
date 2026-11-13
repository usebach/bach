//! The app the example installer installs: one window that says hello.

#![cfg_attr(windows, windows_subsystem = "windows")]

#[cfg(windows)]
fn main() {
    use windows::core::w;
    use windows::Win32::UI::WindowsAndMessaging::{MessageBoxW, MB_ICONINFORMATION, MB_OK};

    // Safety: both strings are static, null-terminated wide literals.
    unsafe {
        MessageBoxW(
            None,
            w!("Hello! Bach installed me, and the uninstaller in Settings › Apps removes me again."),
            w!("Hello"),
            MB_OK | MB_ICONINFORMATION,
        );
    }
}

#[cfg(not(windows))]
fn main() {
    println!("Hello! This example app is meant for Windows.");
}
