//! Branding a setup file: its icon (`app.icon`) and the version details
//! Explorer shows under Properties › Details, written with Windows' own
//! resource updater before the payload is appended.
//!
//! The runtime ships its resources in one language (whatever its build
//! used); the updater replaces them in that same language, so Windows never
//! sees two competing icons.

use std::ffi::c_void;
use std::path::Path;

use anyhow::{bail, Context, Result};
use bach_pack::manifest::App;
use windows::core::{BOOL, HSTRING, PCWSTR};
use windows::Win32::Foundation::{FreeLibrary, HMODULE};
use windows::Win32::System::LibraryLoader::{
    BeginUpdateResourceW, EndUpdateResourceW, EnumResourceLanguagesW, EnumResourceNamesW, LoadLibraryExW, UpdateResourceW,
    LOAD_LIBRARY_AS_DATAFILE,
};
use windows::Win32::UI::WindowsAndMessaging::{RT_GROUP_ICON, RT_ICON, RT_VERSION};

/// The runtime's icon group and version block both have id 1.
const ID_ONE: u16 = 1;
/// `MAKEINTRESOURCE`: a numeric resource id in a name's place.
fn int_resource(id: u16) -> PCWSTR {
    PCWSTR(std::ptr::without_provenance(usize::from(id)))
}

/// An icon with more images than this is damage, not an icon.
const MAX_IMAGES: u16 = 32;

pub fn apply(exe: &Path, icon: Option<&[u8]>, app: &App, file_name: &str) -> Result<()> {
    let icon = icon.map(parse_ico).transpose().context("app.icon isn't a usable .ico file")?;
    let icon_language = language_of(exe, RT_GROUP_ICON).unwrap_or(0);
    // Read before the update starts: the file is opened for writing then.
    let old_images = icon_ids(exe);
    let version_language = language_of(exe, RT_VERSION).unwrap_or(0);
    let version = version_info(app, file_name);

    let path = HSTRING::from(exe.as_os_str());
    // Safety: the handle is used only between Begin and End below, and
    // every buffer outlives the UpdateResourceW call that reads it.
    unsafe {
        let update = BeginUpdateResourceW(&path, false).context("Windows couldn't open the setup file's resources")?;
        let result = (|| -> Result<()> {
            if let Some(icon) = &icon {
                for (index, image) in icon.images.iter().enumerate() {
                    let id = int_resource(index as u16 + 1);
                    UpdateResourceW(update, RT_ICON, id, icon_language, Some(image.as_ptr() as *const c_void), image.len() as u32)?;
                }
                // Only images the runtime really has: "deleting" one that
                // isn't there makes EndUpdateResource fail with
                // ERROR_INTERNAL_ERROR (0x8007054F), losing the whole update.
                for stale in old_images.iter().filter(|id| usize::from(**id) > icon.images.len()) {
                    UpdateResourceW(update, RT_ICON, int_resource(*stale), icon_language, None, 0)?;
                }
                UpdateResourceW(
                    update,
                    RT_GROUP_ICON,
                    int_resource(ID_ONE),
                    icon_language,
                    Some(icon.group.as_ptr() as *const c_void),
                    icon.group.len() as u32,
                )?;
            }
            UpdateResourceW(update, RT_VERSION, int_resource(ID_ONE), version_language, Some(version.as_ptr() as *const c_void), version.len() as u32)?;
            Ok(())
        })();
        match result {
            Ok(()) => EndUpdateResourceW(update, false).context("Windows couldn't save the setup file's icon and details"),
            Err(err) => {
                let _ = EndUpdateResourceW(update, true);
                Err(err.context("Windows couldn't update the setup file's icon and details"))
            }
        }
    }
}

/// The language an existing resource of this type (id 1) uses.
fn language_of(exe: &Path, kind: PCWSTR) -> Option<u16> {
    unsafe extern "system" fn first(_: HMODULE, _: PCWSTR, _: PCWSTR, language: u16, found: isize) -> BOOL {
        // Safety: `found` is the &mut Option<u16> passed below, alive for the call.
        unsafe { *(found as *mut Option<u16>) = Some(language) };
        BOOL(0)
    }
    let path = HSTRING::from(exe.as_os_str());
    let mut found: Option<u16> = None;
    // Safety: the module is loaded as data only (no code runs) and freed
    // before returning; the callback writes into `found`, which outlives it.
    unsafe {
        let module = LoadLibraryExW(&path, None, LOAD_LIBRARY_AS_DATAFILE).ok()?;
        // Stopping early (returning FALSE) reports an error; the language is
        // already recorded by then.
        let _ = EnumResourceLanguagesW(Some(module), kind, int_resource(ID_ONE), Some(first), &mut found as *mut _ as isize);
        let _ = FreeLibrary(module);
    }
    found
}

/// The ids of the icon images (RT_ICON) in an executable.
fn icon_ids(exe: &Path) -> Vec<u16> {
    unsafe extern "system" fn collect(_: HMODULE, _: PCWSTR, name: PCWSTR, found: isize) -> BOOL {
        // Numeric ids arrive as pointers no larger than 0xFFFF (IS_INTRESOURCE).
        let value = name.0 as usize;
        if value <= 0xFFFF {
            // Safety: `found` is the &mut Vec<u16> passed below, alive for the call.
            unsafe { (*(found as *mut Vec<u16>)).push(value as u16) };
        }
        BOOL(1)
    }
    let path = HSTRING::from(exe.as_os_str());
    let mut found: Vec<u16> = Vec::new();
    // Safety: loaded as data only and freed before returning; the callback
    // writes into `found`, which outlives the call.
    unsafe {
        let Ok(module) = LoadLibraryExW(&path, None, LOAD_LIBRARY_AS_DATAFILE) else {
            return found;
        };
        let _ = EnumResourceNamesW(Some(module), RT_ICON, Some(collect), &mut found as *mut _ as isize);
        let _ = FreeLibrary(module);
    }
    found
}

/// An `.ico` file split into the resources Windows wants: each image on its
/// own, and a group directory that points at them by id.
struct Ico {
    images: Vec<Vec<u8>>,
    group: Vec<u8>,
}

fn parse_ico(bytes: &[u8]) -> Result<Ico> {
    let read16 = |at: usize| bytes.get(at..at + 2).map(|b| u16::from_le_bytes([b[0], b[1]]));
    let read32 = |at: usize| bytes.get(at..at + 4).map(|b| u32::from_le_bytes([b[0], b[1], b[2], b[3]]));
    if read16(0) != Some(0) || read16(2) != Some(1) {
        bail!("it doesn't start like an icon file");
    }
    let count = read16(4).unwrap_or(0);
    if count == 0 || count > MAX_IMAGES {
        bail!("it holds {count} images; use 1 to {MAX_IMAGES}");
    }
    let mut group = Vec::with_capacity(6 + 14 * count as usize);
    group.extend_from_slice(&[0, 0, 1, 0]);
    group.extend_from_slice(&count.to_le_bytes());
    let mut images = Vec::new();
    for index in 0..count as usize {
        let entry = 6 + index * 16;
        let head = bytes.get(entry..entry + 8).context("its directory is cut short")?;
        let size = read32(entry + 8).context("its directory is cut short")? as usize;
        let offset = read32(entry + 12).context("its directory is cut short")? as usize;
        let image = bytes
            .get(offset..offset.saturating_add(size))
            .filter(|image| !image.is_empty())
            .context("an image runs past the end of the file")?;
        // GRPICONDIRENTRY: the same first 12 bytes, then the image's id.
        group.extend_from_slice(head);
        group.extend_from_slice(&(size as u32).to_le_bytes());
        group.extend_from_slice(&((index + 1) as u16).to_le_bytes());
        images.push(image.to_vec());
    }
    Ok(Ico { images, group })
}

/// A VS_VERSIONINFO block naming the app.
fn version_info(app: &App, file_name: &str) -> Vec<u8> {
    let numbers = version_numbers(&app.version);
    let ms = (u32::from(numbers[0]) << 16) | u32::from(numbers[1]);
    let ls = (u32::from(numbers[2]) << 16) | u32::from(numbers[3]);
    let mut fixed = Vec::with_capacity(52);
    for value in [0xFEEF_04BDu32, 0x0001_0000, ms, ls, ms, ls, 0x3F, 0, 0x0004_0004, 1, 0, 0, 0] {
        fixed.extend_from_slice(&value.to_le_bytes());
    }

    let description = format!("{} Setup", app.name);
    let strings = [
        ("CompanyName", app.publisher.as_str()),
        ("FileDescription", description.as_str()),
        ("FileVersion", app.version.as_str()),
        ("InternalName", "setup"),
        ("OriginalFilename", file_name),
        ("ProductName", app.name.as_str()),
        ("ProductVersion", app.version.as_str()),
    ];
    let table = node("040904B0", Value::None, strings.iter().map(|(key, value)| node(key, Value::Text(value), Vec::new())).collect());
    let string_info = node("StringFileInfo", Value::None, vec![table]);
    let translation = [0x09u8, 0x04, 0xB0, 0x04];
    let var_info = node("VarFileInfo", Value::None, vec![node("Translation", Value::Binary(&translation), Vec::new())]);
    node("VS_VERSION_INFO", Value::Binary(&fixed), vec![string_info, var_info])
}

/// "1.2.3-beta" → [1, 2, 3, 0].
fn version_numbers(version: &str) -> [u16; 4] {
    let mut numbers = [0u16; 4];
    let parts = version.split(['.', '-', '+']).take(4);
    for (slot, part) in numbers.iter_mut().zip(parts) {
        *slot = part.trim().parse().unwrap_or(0);
    }
    numbers
}

enum Value<'a> {
    None,
    Binary(&'a [u8]),
    Text(&'a str),
}

/// One version-resource node: length, value length, type, a UTF-16 key, the
/// value and the children, each padded to 4 bytes.
fn node(key: &str, value: Value, children: Vec<Vec<u8>>) -> Vec<u8> {
    let mut out = vec![0u8; 6];
    push_utf16(&mut out, key);
    pad(&mut out);
    let (value_len, kind) = match value {
        Value::None => (0u16, 1u16),
        Value::Binary(bytes) => {
            out.extend_from_slice(bytes);
            (bytes.len() as u16, 0)
        }
        Value::Text(text) => {
            let units = push_utf16(&mut out, text);
            // Text lengths count UTF-16 units, including the terminator.
            (units as u16, 1)
        }
    };
    for child in children {
        pad(&mut out);
        out.extend_from_slice(&child);
    }
    let len = out.len() as u16;
    out[0..2].copy_from_slice(&len.to_le_bytes());
    out[2..4].copy_from_slice(&value_len.to_le_bytes());
    out[4..6].copy_from_slice(&kind.to_le_bytes());
    out
}

/// Appends `text` as null-terminated UTF-16; returns the units written.
fn push_utf16(out: &mut Vec<u8>, text: &str) -> usize {
    let mut units = 0;
    for unit in text.encode_utf16().chain(std::iter::once(0)) {
        out.extend_from_slice(&unit.to_le_bytes());
        units += 1;
    }
    units
}

fn pad(out: &mut Vec<u8>) {
    while !out.len().is_multiple_of(4) {
        out.push(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app() -> App {
        App {
            id: "Hello".into(),
            name: "Hello".into(),
            version: "1.2.3".into(),
            publisher: "Example".into(),
            description: None,
            website: None,
            executable: "hello.exe".into(),
        }
    }

    #[test]
    fn versions_become_four_numbers() {
        assert_eq!(version_numbers("1.2.3"), [1, 2, 3, 0]);
        assert_eq!(version_numbers("2.0.0-beta.1"), [2, 0, 0, 0]);
        assert_eq!(version_numbers("10"), [10, 0, 0, 0]);
    }

    #[test]
    fn version_blocks_are_well_formed() {
        let block = version_info(&app(), "Hello-Setup-1.2.3.exe");
        let len = u16::from_le_bytes([block[0], block[1]]) as usize;
        assert_eq!(len, block.len());
        assert_eq!(u16::from_le_bytes([block[2], block[3]]), 52);
        // The fixed block's signature sits right after the padded key.
        let key_end = 6 + "VS_VERSION_INFO\0".len() * 2;
        let fixed = key_end + (4 - key_end % 4) % 4;
        assert_eq!(&block[fixed..fixed + 4], &0xFEEF_04BDu32.to_le_bytes());
        // FileVersionMS = 1.2, LS = 3.0.
        assert_eq!(&block[fixed + 8..fixed + 12], &((1u32 << 16) | 2).to_le_bytes());
    }

    #[test]
    fn icons_split_into_images_and_a_group() {
        // A one-image .ico: header, one directory entry, then 4 image bytes.
        let mut ico = vec![0, 0, 1, 0, 1, 0];
        ico.extend_from_slice(&[16, 16, 0, 0, 1, 0, 32, 0]);
        ico.extend_from_slice(&4u32.to_le_bytes());
        ico.extend_from_slice(&22u32.to_le_bytes());
        ico.extend_from_slice(&[1, 2, 3, 4]);
        let parsed = parse_ico(&ico).unwrap();
        assert_eq!(parsed.images, vec![vec![1, 2, 3, 4]]);
        assert_eq!(parsed.group.len(), 6 + 14);
        assert_eq!(&parsed.group[18..20], &1u16.to_le_bytes());
        assert!(parse_ico(b"not an icon").is_err());
    }
}

#[cfg(test)]
mod apply_tests {
    use super::*;

    /// An icon with `count` tiny PNG images.
    fn ico(count: u16) -> Vec<u8> {
        let png = b"\x89PNG\r\n\x1a\nfake image".to_vec();
        let mut out = vec![0, 0, 1, 0];
        out.extend_from_slice(&count.to_le_bytes());
        let mut offset = 6 + 16 * usize::from(count);
        for index in 0..count {
            out.extend_from_slice(&[16 + index as u8, 16 + index as u8, 0, 0, 1, 0, 32, 0]);
            out.extend_from_slice(&(png.len() as u32).to_le_bytes());
            out.extend_from_slice(&(offset as u32).to_le_bytes());
            offset += png.len();
        }
        for _ in 0..count {
            out.extend_from_slice(&png);
        }
        out
    }

    /// Replaces the runtime's icon with fewer, as many, and more images
    /// than it has. The runtime is the bach-setup.exe this workspace built.
    #[test]
    fn icons_of_any_size_replace_the_runtimes() {
        let built = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().join("bach-setup.exe");
        if !built.is_file() {
            eprintln!("skipped: build bach-setup first");
            return;
        }
        let app = App {
            id: "Hello".into(),
            name: "Hello".into(),
            version: "1.2.3".into(),
            publisher: "Example".into(),
            description: None,
            website: None,
            executable: "hello.exe".into(),
        };
        let before = icon_ids(&built).len();
        assert!(before > 0, "the runtime has an icon");
        for count in [1u16, before as u16, before as u16 + 3] {
            let copy = std::env::temp_dir().join(format!("bach-icon-{count}-{}.exe", std::process::id()));
            std::fs::copy(&built, &copy).unwrap();
            apply(&copy, Some(&ico(count)), &app, "Hello-Setup.exe").unwrap();
            assert_eq!(icon_ids(&copy).len(), usize::from(count), "{count} images");
            std::fs::remove_file(copy).unwrap();
        }
    }
}
