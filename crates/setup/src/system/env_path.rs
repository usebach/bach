//! The user's `PATH` (`HKCU\Environment`, value `Path`).
//!
//! `PATH` belongs to the user and to every other program they installed,
//! so setup edits it with care: it works on the raw UTF-16 value, adds or
//! removes exactly one entry and leaves every other byte as it was, keeps
//! the value's type (`REG_EXPAND_SZ` entries like `%USERPROFILE%\bin` stay
//! unexpanded), and writes nothing when there's nothing to change. A value
//! of a type it doesn't expect is left alone with an error.

use std::path::Path;

use anyhow::{bail, Context, Result};
use windows::core::w;
use windows::Win32::Foundation::{LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, WM_SETTINGCHANGE};
use windows_registry::{Type, CURRENT_USER};

const ENVIRONMENT: &str = "Environment";
const VALUE: &str = "Path";
/// Windows reads at most this many characters of an environment variable.
const MAX_LEN: usize = 32_767;

/// Adds `folder` to the user's `PATH` unless it's there already.
pub fn add(folder: &Path) -> Result<()> {
    if edit(ENVIRONMENT, VALUE, folder, with_entry)? {
        broadcast();
    }
    Ok(())
}

/// Takes `folder` off the user's `PATH`, if it's there.
pub fn remove(folder: &Path) -> Result<()> {
    if edit(ENVIRONMENT, VALUE, folder, without_entry)? {
        broadcast();
    }
    Ok(())
}

/// Applies `change` to the value; true when it wrote something.
fn edit(
    key_path: &str,
    value_name: &str,
    folder: &Path,
    change: fn(&[u16], &[u16]) -> Option<Vec<u16>>,
) -> Result<bool> {
    let entry: Vec<u16> = folder.as_os_str().encode_wide_vec();
    if entry.contains(&u16::from(b';')) {
        bail!("{} contains a semicolon, which PATH can't hold", folder.display());
    }
    let key = CURRENT_USER.create(key_path).context("couldn't open the user's environment")?;
    let (kind, current) = match key.get_type(value_name) {
        Ok(kind @ (Type::String | Type::ExpandString)) => {
            let raw = key.get_value(value_name).context("couldn't read PATH")?;
            (kind, trim_nuls(&wide(&raw)))
        }
        Ok(_) => bail!("PATH isn't text in the registry, so setup leaves it alone"),
        // No PATH of the user's own yet: start one.
        Err(_) => (Type::ExpandString, Vec::new()),
    };
    let Some(updated) = change(&current, &entry) else {
        return Ok(false);
    };
    if updated.len() >= MAX_LEN {
        bail!("PATH would be longer than Windows allows");
    }
    let mut bytes: Vec<u8> = updated.iter().flat_map(|unit| unit.to_le_bytes()).collect();
    bytes.extend_from_slice(&[0, 0]);
    key.set_bytes(value_name, kind, &bytes).context("couldn't write PATH")?;
    Ok(true)
}

trait EncodeWide {
    fn encode_wide_vec(&self) -> Vec<u16>;
}

impl EncodeWide for std::ffi::OsStr {
    fn encode_wide_vec(&self) -> Vec<u16> {
        use std::os::windows::ffi::OsStrExt;
        self.encode_wide().collect()
    }
}

fn wide(bytes: &[u8]) -> Vec<u16> {
    bytes.as_chunks::<2>().0.iter().map(|pair| u16::from_le_bytes(*pair)).collect()
}

fn trim_nuls(units: &[u16]) -> Vec<u16> {
    let end = units.iter().rposition(|unit| *unit != 0).map_or(0, |at| at + 1);
    units[..end].to_vec()
}

/// Whether two entries name the same folder: case-insensitive, ignoring a
/// trailing backslash and surrounding spaces.
fn same_entry(a: &[u16], b: &[u16]) -> bool {
    let normal = |units: &[u16]| {
        let text = String::from_utf16_lossy(units);
        text.trim().trim_end_matches(['\\', '/']).to_lowercase()
    };
    let a = normal(a);
    !a.is_empty() && a == normal(b)
}

/// `path` with `entry` appended, or None when it's already there.
fn with_entry(path: &[u16], entry: &[u16]) -> Option<Vec<u16>> {
    if path.split(|unit| *unit == u16::from(b';')).any(|part| same_entry(part, entry)) {
        return None;
    }
    let mut out = path.to_vec();
    if !out.is_empty() && out.last() != Some(&u16::from(b';')) {
        out.push(u16::from(b';'));
    }
    out.extend_from_slice(entry);
    Some(out)
}

/// `path` without `entry`, or None when it isn't there. Every other entry,
/// empty ones included, stays exactly as it was.
fn without_entry(path: &[u16], entry: &[u16]) -> Option<Vec<u16>> {
    let parts: Vec<&[u16]> = path.split(|unit| *unit == u16::from(b';')).collect();
    if !parts.iter().any(|part| same_entry(part, entry)) {
        return None;
    }
    let kept: Vec<&[u16]> = parts.into_iter().filter(|part| !same_entry(part, entry)).collect();
    Some(kept.join(&u16::from(b';')))
}

/// Tells running programs (Explorer, so new terminals) that the
/// environment changed. Never waits long on a program that hangs.
fn broadcast() {
    // Safety: a broadcast with a static string and a 2-second cap.
    unsafe {
        SendMessageTimeoutW(
            HWND_BROADCAST,
            WM_SETTINGCHANGE,
            WPARAM(0),
            LPARAM(w!("Environment").as_ptr() as isize),
            SMTO_ABORTIFHUNG,
            2000,
            None,
        );
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn units(text: &str) -> Vec<u16> {
        text.encode_utf16().collect()
    }

    fn text(units: &[u16]) -> String {
        String::from_utf16(units).unwrap()
    }

    #[test]
    fn adding_appends_once() {
        let added = with_entry(&units(r"C:\a;%USERPROFILE%\b"), &units(r"C:\App\bin")).unwrap();
        assert_eq!(text(&added), r"C:\a;%USERPROFILE%\b;C:\App\bin");
        assert!(with_entry(&added, &units(r"c:\app\BIN\")).is_none());
        assert_eq!(text(&with_entry(&[], &units(r"C:\App")).unwrap()), r"C:\App");
        assert_eq!(text(&with_entry(&units(r"C:\a;"), &units(r"C:\App")).unwrap()), r"C:\a;C:\App");
    }

    #[test]
    fn removing_touches_only_that_entry() {
        let path = units(r"C:\a;;C:\App\bin;%X%\c;C:\App\bin\");
        let removed = without_entry(&path, &units(r"C:\App\bin")).unwrap();
        assert_eq!(text(&removed), r"C:\a;;%X%\c");
        assert!(without_entry(&removed, &units(r"C:\App\bin")).is_none());
        assert!(without_entry(&units(""), &units(r"C:\App")).is_none());
    }

    #[test]
    fn unpaired_surrogates_survive() {
        // Not valid UTF-16, but it's the user's: keep it bit for bit.
        let mut path = units(r"C:\a;");
        path.push(0xD800);
        path.extend(units(r"\x"));
        let added = with_entry(&path, &units(r"C:\App")).unwrap();
        assert!(added.starts_with(&path));
        let removed = without_entry(&added, &units(r"C:\App")).unwrap();
        assert_eq!(removed, path);
    }

    /// The registry round trip, on a scratch key instead of the real PATH.
    #[test]
    fn registry_round_trip_keeps_the_type() {
        let key_path = format!(r"Software\BachTest-{}\Environment", std::process::id());
        let key = CURRENT_USER.create(&key_path).unwrap();
        key.set_expand_string(VALUE, r"%USERPROFILE%\bin").unwrap();
        let folder = Path::new(r"C:\Users\x\AppData\Local\Programs\Hello");

        assert!(edit(&key_path, VALUE, folder, with_entry).unwrap());
        assert!(!edit(&key_path, VALUE, folder, with_entry).unwrap(), "added once only");
        assert_eq!(key.get_type(VALUE).unwrap(), Type::ExpandString);
        assert_eq!(key.get_string(VALUE).unwrap(), format!(r"%USERPROFILE%\bin;{}", folder.display()));

        assert!(edit(&key_path, VALUE, folder, without_entry).unwrap());
        assert_eq!(key.get_string(VALUE).unwrap(), r"%USERPROFILE%\bin");
        assert_eq!(key.get_type(VALUE).unwrap(), Type::ExpandString);

        key.set_u32(VALUE, 7).unwrap();
        assert!(edit(&key_path, VALUE, folder, with_entry).is_err(), "a number is left alone");
        CURRENT_USER.remove_tree(format!(r"Software\BachTest-{}", std::process::id())).unwrap();
    }
}
