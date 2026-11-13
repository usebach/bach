//! The rules for every path that crosses from a config into a setup file,
//! and from a setup file onto someone's disk.
//!
//! Packaged paths are relative, use `/`, and can never climb out of the
//! install folder. Data folders (deleted on request when uninstalling) must
//! sit inside the user's own profile, so a typo in a config can never point
//! the uninstaller at `C:\` or a whole drive.

use std::path::{Path, PathBuf};

use anyhow::{bail, Result};

/// Characters Windows refuses in a file name, besides the path separators.
const FORBIDDEN: &[char] = &['<', '>', ':', '"', '|', '?', '*'];

/// Names Windows reserves for devices, with or without an extension.
const RESERVED: &[&str] = &[
    "con", "prn", "aux", "nul", "com1", "com2", "com3", "com4", "com5", "com6", "com7", "com8",
    "com9", "lpt1", "lpt2", "lpt3", "lpt4", "lpt5", "lpt6", "lpt7", "lpt8", "lpt9",
];

/// Checks a packaged path and returns it in its one canonical spelling:
/// relative, `/`-separated, no `.` or `..`, no empty parts.
pub fn normalize(path: &str) -> Result<String> {
    if path.starts_with(['/', '\\']) {
        bail!("\"{path}\" is absolute; packaged paths are relative to the install folder");
    }
    let mut parts = Vec::new();
    for part in path.split(['/', '\\']) {
        check_component(path, part)?;
        parts.push(part);
    }
    if parts.is_empty() {
        bail!("a packaged path is empty");
    }
    Ok(parts.join("/"))
}

fn check_component(path: &str, part: &str) -> Result<()> {
    if part.is_empty() || part == "." {
        bail!("\"{path}\" has an empty part");
    }
    if part == ".." {
        bail!("\"{path}\" climbs out of its folder with \"..\"");
    }
    if let Some(bad) = part.chars().find(|c| FORBIDDEN.contains(c) || c.is_control()) {
        bail!("\"{path}\" contains \"{}\", which Windows doesn't allow in names", bad.escape_default());
    }
    if part.ends_with(['.', ' ']) {
        bail!("\"{path}\" has a part ending in a dot or space, which Windows drops");
    }
    let stem = part.split('.').next().unwrap_or(part).to_ascii_lowercase();
    if RESERVED.contains(&stem.as_str()) {
        bail!("\"{path}\" uses \"{part}\", a name Windows reserves for devices");
    }
    Ok(())
}

/// Where a packaged path lands inside `dir`. Call [`normalize`] first; this
/// only joins.
pub fn under(dir: &Path, packaged: &str) -> PathBuf {
    packaged.split('/').fold(dir.to_path_buf(), |path, part| path.join(part))
}

/// Whether `id` can name a folder and a registry key: letters, digits,
/// spaces, dots, dashes and underscores, not starting or ending in a space
/// or dot.
pub fn is_valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 64
        && id.chars().all(|c| c.is_alphanumeric() || matches!(c, ' ' | '.' | '-' | '_'))
        && !id.starts_with([' ', '.'])
        && !id.ends_with([' ', '.'])
}

/// The folders a data folder may start with, and the environment variable
/// each one reads.
pub const DATA_ROOTS: &[&str] = &["APPDATA", "LOCALAPPDATA", "USERPROFILE"];

/// Checks a data folder such as `%APPDATA%\Hello`: one of [`DATA_ROOTS`],
/// then at least one folder of the app's own.
pub fn check_data_folder(template: &str) -> Result<()> {
    let Some((root, rest)) = split_data_folder(template) else {
        bail!(
            "\"{template}\" must start with %APPDATA%, %LOCALAPPDATA% or %USERPROFILE%, \
             so the uninstaller only ever deletes inside your user's own folders"
        );
    };
    if !DATA_ROOTS.contains(&root.as_str()) {
        bail!("\"{template}\" starts with %{root}%; use %APPDATA%, %LOCALAPPDATA% or %USERPROFILE%");
    }
    if rest.is_empty() {
        bail!("\"{template}\" names the whole %{root}% folder; add your app's own folder after it");
    }
    normalize(&rest).map(|_| ())
}

/// The real folder a checked data folder points at, reading the root from
/// `lookup` (the environment, in the setup runtime).
pub fn expand_data_folder(template: &str, lookup: impl Fn(&str) -> Option<String>) -> Option<PathBuf> {
    check_data_folder(template).ok()?;
    let (root, rest) = split_data_folder(template)?;
    let base = lookup(&root).filter(|value| !value.trim().is_empty())?;
    let rest = normalize(&rest).ok()?;
    Some(under(Path::new(&base), &rest))
}

/// `%APPDATA%\Hello\Cache` → ("APPDATA", "Hello\Cache").
fn split_data_folder(template: &str) -> Option<(String, String)> {
    let inner = template.strip_prefix('%')?;
    let end = inner.find('%')?;
    let root = inner[..end].to_ascii_uppercase();
    let rest = inner[end + 1..].trim_start_matches(['/', '\\']).to_string();
    Some((root, rest))
}

/// Matches a packaged path against an exclude pattern. `*` matches within
/// one name, `**` across folders, `?` one character. A pattern without `/`
/// matches the file name anywhere (`*.pdb`); one with `/` matches from the
/// top of the source folder (`docs/**`). Case-insensitive, like Windows.
pub fn matches(pattern: &str, path: &str) -> bool {
    let pattern = pattern.replace('\\', "/").to_lowercase();
    let path = path.to_lowercase();
    if pattern.contains('/') {
        glob(pattern.as_bytes(), path.as_bytes())
    } else {
        let name = path.rsplit('/').next().unwrap_or(&path);
        glob(pattern.as_bytes(), name.as_bytes())
    }
}

fn glob(pattern: &[u8], text: &[u8]) -> bool {
    match pattern.split_first() {
        None => text.is_empty(),
        Some((b'*', rest)) if rest.first() == Some(&b'*') => {
            let rest = &rest[1..];
            let rest = rest.strip_prefix(b"/").unwrap_or(rest);
            (0..=text.len()).any(|at| glob(rest, &text[at..]))
        }
        Some((b'*', rest)) => {
            let span = text.iter().position(|&c| c == b'/').unwrap_or(text.len());
            (0..=span).any(|at| glob(rest, &text[at..]))
        }
        Some((b'?', rest)) => text.first().is_some_and(|&c| c != b'/') && glob(rest, &text[1..]),
        Some((&c, rest)) => text.first() == Some(&c) && glob(rest, &text[1..]),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_separators() {
        assert_eq!(normalize(r"bin\app.exe").unwrap(), "bin/app.exe");
        assert_eq!(normalize("a/b/c.txt").unwrap(), "a/b/c.txt");
    }

    #[test]
    fn rejects_paths_that_escape_or_break_windows() {
        for bad in ["", "/etc/passwd", r"\x", "a/../b", "./a", "a//b", "C:/x", "a/con.txt", "a/b.", "q?"] {
            assert!(normalize(bad).is_err(), "{bad:?} should be rejected");
        }
    }

    #[test]
    fn ids_name_folders() {
        assert!(is_valid_id("Hello World"));
        assert!(is_valid_id("my-app_2.0"));
        assert!(!is_valid_id(""));
        assert!(!is_valid_id("a/b"));
        assert!(!is_valid_id(" lead"));
        assert!(!is_valid_id("trail."));
    }

    #[test]
    fn data_folders_stay_in_the_profile() {
        assert!(check_data_folder(r"%APPDATA%\Hello").is_ok());
        assert!(check_data_folder("%localappdata%/Hello/Cache").is_ok());
        assert!(check_data_folder(r"%APPDATA%").is_err());
        assert!(check_data_folder(r"%APPDATA%\").is_err());
        assert!(check_data_folder(r"%APPDATA%\..\x").is_err());
        assert!(check_data_folder(r"%WINDIR%\x").is_err());
        assert!(check_data_folder(r"C:\Hello").is_err());
    }

    #[test]
    fn data_folders_expand() {
        let lookup = |name: &str| (name == "APPDATA").then(|| r"C:\Users\a\AppData\Roaming".to_string());
        let path = expand_data_folder(r"%APPDATA%\Hello\Cache", lookup).unwrap();
        assert_eq!(path, Path::new(r"C:\Users\a\AppData\Roaming").join("Hello").join("Cache"));
        assert!(expand_data_folder(r"%LOCALAPPDATA%\Hello", lookup).is_none());
    }

    #[test]
    fn globs() {
        assert!(matches("*.pdb", "bin/app.pdb"));
        assert!(matches("*.PDB", "app.pdb"));
        assert!(!matches("*.pdb", "app.exe"));
        assert!(matches("docs/**", "docs/a/b.md"));
        assert!(!matches("docs/*", "docs/a/b.md"));
        assert!(matches("docs/*", "docs/b.md"));
        assert!(matches("**/cache/*", "a/b/cache/x"));
        assert!(matches("app?.log", "app1.log"));
    }
}
