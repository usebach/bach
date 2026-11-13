//! Facts about this computer and this session that setup checks before it
//! changes anything: the user's languages, free space, and whether another
//! setup for the same app is already at work.

use std::path::Path;

use windows::core::{HSTRING, PWSTR};
use windows::Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE};
use windows::Win32::Globalization::{GetUserPreferredUILanguages, MUI_LANGUAGE_NAME};
use windows::Win32::Storage::FileSystem::GetDiskFreeSpaceExW;
use windows::Win32::System::Console::{AttachConsole, ATTACH_PARENT_PROCESS};
use windows::Win32::System::Threading::CreateMutexW;

/// The user's Windows display languages, most preferred first, like
/// `["de-DE", "en-US"]`. Empty if Windows won't say.
pub fn ui_languages() -> Vec<String> {
    let mut count = 0u32;
    let mut len = 0u32;
    // Safety: the first call only reports the buffer length it needs; the
    // second fills a buffer of exactly that many UTF-16 units.
    unsafe {
        if GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut count, None, &mut len).is_err() || len == 0 {
            return Vec::new();
        }
        let mut buffer = vec![0u16; len as usize];
        if GetUserPreferredUILanguages(MUI_LANGUAGE_NAME, &mut count, Some(PWSTR(buffer.as_mut_ptr())), &mut len).is_err() {
            return Vec::new();
        }
        // A list of NUL-terminated names, ending in an empty one.
        buffer
            .split(|unit| *unit == 0)
            .filter(|name| !name.is_empty())
            .map(String::from_utf16_lossy)
            .collect()
    }
}

/// Bytes free for this user on the drive that holds `path` (or would: the
/// nearest folder above it that exists). None if Windows won't say.
pub fn free_space(path: &Path) -> Option<u64> {
    let existing = path.ancestors().find(|folder| folder.is_dir())?;
    let mut free = 0u64;
    // Safety: `free` outlives the call; the other outputs are optional.
    unsafe { GetDiskFreeSpaceExW(&HSTRING::from(existing.as_os_str()), Some(&mut free), None, None).ok()? };
    Some(free)
}

/// Held while a setup for one app installs or removes it, so two setup
/// windows (or a window and a silent run) never work on the same install
/// at once. Released when dropped.
pub struct SetupLock(HANDLE);

// Safety: a mutex handle may be closed from any thread.
unsafe impl Send for SetupLock {}

impl SetupLock {
    /// None when another setup for `app_id` holds the lock.
    pub fn acquire(app_id: &str) -> Option<Self> {
        let name = HSTRING::from(format!("Local\\BachSetup-{app_id}"));
        // Safety: a named mutex owned by this handle until Drop closes it.
        unsafe {
            let handle = CreateMutexW(None, true, &name).ok()?;
            if GetLastError() == ERROR_ALREADY_EXISTS {
                let _ = CloseHandle(handle);
                return None;
            }
            Some(Self(handle))
        }
    }
}

impl Drop for SetupLock {
    fn drop(&mut self) {
        // Safety: our own handle, closed once.
        unsafe {
            let _ = CloseHandle(self.0);
        }
    }
}

/// Lets a silent run started from a terminal print there, although setup
/// is a windowed program. Does nothing when there's no terminal.
pub fn attach_parent_console() {
    // Safety: no pointers; failing just means there's no console to use.
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn free_space_looks_above_missing_folders() {
        let missing = std::env::temp_dir().join("bach-free-space").join("not").join("there");
        assert!(free_space(&missing).is_some_and(|free| free > 0));
    }

    #[test]
    fn one_lock_per_app() {
        let id = format!("test-{}", std::process::id());
        let first = SetupLock::acquire(&id).expect("the first lock");
        assert!(SetupLock::acquire(&id).is_none(), "a second setup must wait");
        drop(first);
        assert!(SetupLock::acquire(&id).is_some(), "free again once released");
    }

    #[test]
    fn languages_look_like_tags() {
        for language in ui_languages() {
            assert!(language.len() >= 2 && !language.contains('\0'), "{language:?}");
        }
    }
}
