//! The files that carry an app's version, read and rewritten in place.
//!
//! `bach version` keeps all of them in step, so a release never needs the
//! version typed twice: bach.toml, package.json and its lock file, Cargo.toml
//! and Cargo.lock, Tauri's config and a .NET project. Every edit replaces
//! only the version text, so line endings, key order and formatting stay as
//! they were (a lock file that is rewritten by hand-rolled JSON would show up
//! as a diff of thousands of lines, and `npm ci` is picky about it).

use std::path::{Path, PathBuf};

use anyhow::{Context, Result};

/// One file's version.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Found {
    /// Relative to the project folder, with `/`.
    pub file: String,
    pub version: String,
}

/// A file whose version was (or would be) rewritten.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    pub file: String,
    pub from: String,
}

/// Where a version lives inside a file.
#[derive(Clone, Copy)]
enum Kind {
    /// `[table]` with `version = "x"` in a TOML file; the tables are tried
    /// in order and each one that has a literal version is rewritten.
    Toml(&'static [&'static str]),
    /// A string at these keys in a JSON file.
    Json(&'static [&'static [&'static str]]),
    /// `<Version>x</Version>` in a .NET project.
    Xml,
    /// Cargo.lock: the packages built from this checkout.
    CargoLock,
}

/// The files looked at, in the order they are reported. Folders are relative
/// to the project folder (the one holding bach.toml).
const FILES: &[(&str, Kind)] = &[
    ("bach.toml", Kind::Toml(&["app"])),
    ("package.json", Kind::Json(&[&["version"]])),
    ("package-lock.json", Kind::Json(&[&["version"], &["packages", "", "version"]])),
    ("npm-shrinkwrap.json", Kind::Json(&[&["version"], &["packages", "", "version"]])),
    ("Cargo.toml", Kind::Toml(&["package", "workspace.package"])),
    ("Cargo.lock", Kind::CargoLock),
    ("src-tauri/Cargo.toml", Kind::Toml(&["package", "workspace.package"])),
    ("src-tauri/Cargo.lock", Kind::CargoLock),
    ("src-tauri/tauri.conf.json", Kind::Json(&[&["version"], &["package", "version"]])),
];

/// Every version the project's files say, in a stable order.
pub fn read_all(dir: &Path) -> Result<Vec<Found>> {
    let mut found = Vec::new();
    for (file, kind) in files(dir)? {
        let text = read(dir, &file)?;
        for version in versions_in(&kind, &text) {
            found.push(Found { file: file.clone(), version });
        }
    }
    Ok(found)
}

/// Writes `new` everywhere a version is found and returns what changed.
/// `base` is the version being replaced, which is how Cargo.lock's own
/// packages are told from its dependencies.
pub fn write_all(dir: &Path, base: &str, new: &str, dry_run: bool) -> Result<Vec<Change>> {
    let mut changes = Vec::new();
    // Cargo.toml goes first: the versions it had are the ones Cargo.lock holds.
    let mut cargo_olds = vec![base.to_string()];
    for (file, kind) in files(dir)? {
        let text = read(dir, &file)?;
        let (edited, olds) = match &kind {
            Kind::CargoLock => rewrite_cargo_lock(&text, &cargo_olds, new),
            kind => rewrite(kind, &text, new),
        };
        if matches!(kind, Kind::Toml(_)) && file.ends_with("Cargo.toml") {
            cargo_olds.extend(olds.iter().cloned());
        }
        let Some(first) = olds.iter().find(|old| old.as_str() != new) else {
            continue;
        };
        changes.push(Change { file: file.clone(), from: first.clone() });
        if !dry_run {
            std::fs::write(dir.join(&file), edited).with_context(|| format!("couldn't write {file}"))?;
        }
    }
    Ok(changes)
}

/// The existing files from [`FILES`] plus the folder's .NET projects.
fn files(dir: &Path) -> Result<Vec<(String, Kind)>> {
    let mut list: Vec<(String, Kind)> = Vec::new();
    for (file, kind) in FILES {
        if dir.join(file).is_file() {
            list.push((file.to_string(), *kind));
        }
    }
    let mut projects: Vec<String> = std::fs::read_dir(dir)
        .with_context(|| format!("couldn't read {}", dir.display()))?
        .flatten()
        .filter(|entry| entry.path().extension().is_some_and(|ext| ext.eq_ignore_ascii_case("csproj")))
        .map(|entry| entry.file_name().to_string_lossy().into_owned())
        .collect();
    projects.sort();
    list.extend(projects.into_iter().map(|name| (name, Kind::Xml)));
    Ok(list)
}

fn read(dir: &Path, file: &str) -> Result<String> {
    let path: PathBuf = dir.join(file);
    std::fs::read_to_string(&path).with_context(|| format!("couldn't read {}", path.display()))
}

fn versions_in(kind: &Kind, text: &str) -> Vec<String> {
    match kind {
        Kind::CargoLock => Vec::new(),
        kind => rewrite(kind, text, "").1,
    }
}

/// The text with the new version, and the versions it replaced.
fn rewrite(kind: &Kind, text: &str, new: &str) -> (String, Vec<String>) {
    let mut text = text.to_string();
    let mut olds = Vec::new();
    match kind {
        Kind::Toml(tables) => {
            for table in *tables {
                if let Some((range, old)) = toml_version(&text, table) {
                    text.replace_range(range, new);
                    olds.push(old);
                }
            }
        }
        Kind::Json(paths) => {
            for path in *paths {
                if let Some((range, old)) = json_string(&text, path) {
                    text.replace_range(range, new);
                    olds.push(old);
                }
            }
        }
        Kind::Xml => {
            if let Some((range, old)) = xml_version(&text) {
                text.replace_range(range, new);
                olds.push(old);
            }
        }
        Kind::CargoLock => {}
    }
    (text, olds)
}

/// The byte range and value of `version = "x"` directly in `[table]`.
fn toml_version(text: &str, table: &str) -> Option<(std::ops::Range<usize>, String)> {
    let mut in_table = false;
    let mut offset = 0;
    for line in text.split_inclusive('\n') {
        let trimmed = line.trim();
        if trimmed.starts_with('[') {
            // `[ app ]` and `[app] # comment` are the same table.
            let header = trimmed.split('#').next().unwrap_or("").trim();
            in_table = header.strip_prefix('[').and_then(|rest| rest.strip_suffix(']')).is_some_and(|name| name.trim() == table);
        } else if in_table {
            if let Some(found) = toml_version_line(line, offset) {
                return Some(found);
            }
        }
        offset += line.len();
    }
    None
}

/// `version = "1.2.3"` (or with single quotes) → where the 1.2.3 is.
fn toml_version_line(line: &str, offset: usize) -> Option<(std::ops::Range<usize>, String)> {
    let rest = line.trim_start().strip_prefix("version")?.trim_start().strip_prefix('=')?.trim_start();
    let quote = rest.chars().next().filter(|c| *c == '"' || *c == '\'')?;
    let value = &rest[1..];
    let end = value.find(quote)?;
    let start = offset + (line.len() - value.len());
    Some((start..start + end, value[..end].to_string()))
}

/// The first `<Version>x</Version>` of a .NET project file.
fn xml_version(text: &str) -> Option<(std::ops::Range<usize>, String)> {
    let start = text.find("<Version>")? + "<Version>".len();
    let end = start + text[start..].find("</Version>")?;
    let value = text[start..end].trim();
    // A property like $(MajorVersion).0 isn't a version to rewrite.
    if value.is_empty() || value.contains('$') {
        return None;
    }
    Some((start..end, text[start..end].to_string()))
}

/// Cargo.lock's own packages (the ones without a `source`, which are the
/// workspace's crates) at any of `olds`.
fn rewrite_cargo_lock(text: &str, olds: &[String], new: &str) -> (String, Vec<String>) {
    let lines: Vec<&str> = text.split_inclusive('\n').collect();
    let mut out = String::with_capacity(text.len());
    let mut changed = Vec::new();
    let mut i = 0;
    while i < lines.len() {
        if lines[i].trim() != "[[package]]" {
            out.push_str(lines[i]);
            i += 1;
            continue;
        }
        let end = (i + 1..lines.len()).find(|&j| lines[j].trim_start().starts_with('[')).unwrap_or(lines.len());
        let block = &lines[i..end];
        let local = !block.iter().any(|line| line.trim_start().starts_with("source"));
        for line in block {
            match toml_version_line(line, 0) {
                Some((range, old)) if local && olds.contains(&old) && old != new => {
                    out.push_str(&line[..range.start]);
                    out.push_str(new);
                    out.push_str(&line[range.end..]);
                    changed.push(old);
                }
                _ => out.push_str(line),
            }
        }
        i = end;
    }
    (out, changed)
}

/// The byte range and contents of the string at `path` in a JSON document,
/// found without parsing the rest (the text around it is never touched).
fn json_string(text: &str, path: &[&str]) -> Option<(std::ops::Range<usize>, String)> {
    let mut scan = Scan { bytes: text.as_bytes(), at: if text.starts_with('\u{feff}') { 3 } else { 0 } };
    let range = scan.find(path)?;
    Some((range.clone(), text[range].to_string()))
}

struct Scan<'a> {
    bytes: &'a [u8],
    at: usize,
}

impl Scan<'_> {
    fn space(&mut self) {
        while self.bytes.get(self.at).is_some_and(u8::is_ascii_whitespace) {
            self.at += 1;
        }
    }

    fn eat(&mut self, byte: u8) -> Option<()> {
        self.space();
        (self.bytes.get(self.at) == Some(&byte)).then(|| self.at += 1)
    }

    /// A string's contents, leaving the scanner after the closing quote.
    fn string(&mut self) -> Option<std::ops::Range<usize>> {
        self.eat(b'"')?;
        let start = self.at;
        while let Some(&byte) = self.bytes.get(self.at) {
            match byte {
                b'\\' => self.at += 2,
                b'"' => {
                    self.at += 1;
                    return Some(start..self.at - 1);
                }
                _ => self.at += 1,
            }
        }
        None
    }

    /// Passes over any value.
    fn skip(&mut self) -> Option<()> {
        self.space();
        match *self.bytes.get(self.at)? {
            b'"' => self.string().map(|_| ()),
            open @ (b'{' | b'[') => {
                let close = if open == b'{' { b'}' } else { b']' };
                self.at += 1;
                self.space();
                if self.bytes.get(self.at) == Some(&close) {
                    self.at += 1;
                    return Some(());
                }
                loop {
                    if open == b'{' {
                        self.string()?;
                        self.eat(b':')?;
                    }
                    self.skip()?;
                    self.space();
                    match self.bytes.get(self.at)? {
                        b',' => self.at += 1,
                        byte if *byte == close => {
                            self.at += 1;
                            return Some(());
                        }
                        _ => return None,
                    }
                }
            }
            _ => {
                while self.bytes.get(self.at).is_some_and(|b| !matches!(b, b',' | b'}' | b']') && !b.is_ascii_whitespace()) {
                    self.at += 1;
                }
                Some(())
            }
        }
    }

    /// Walks an object for `path`; the last key must hold a string.
    fn find(&mut self, path: &[&str]) -> Option<std::ops::Range<usize>> {
        self.eat(b'{')?;
        self.space();
        if self.bytes.get(self.at) == Some(&b'}') {
            return None;
        }
        loop {
            let key = self.string()?;
            self.eat(b':')?;
            if &self.bytes[key] == path[0].as_bytes() {
                return if path.len() == 1 {
                    self.string()
                } else {
                    self.find(&path[1..])
                };
            }
            self.skip()?;
            self.space();
            match self.bytes.get(self.at)? {
                b',' => self.at += 1,
                _ => return None,
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str, files: &[(&str, &str)]) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("bach-version-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        for (path, text) in files {
            let path = dir.join(path);
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(path, text).unwrap();
        }
        dir
    }

    #[test]
    fn only_the_top_level_version_of_a_json_file_changes() {
        let text = "{\n  \"name\": \"a\",\n  \"build\": { \"version\": \"9.9.9\", \"list\": [1, {\"version\": \"8\"}] },\n  \"version\": \"1.0.0\"\n}\n";
        let (range, old) = json_string(text, &["version"]).unwrap();
        assert_eq!(old, "1.0.0");
        assert_eq!(&text[range], "1.0.0");
        assert!(json_string("{\"name\": \"a\"}", &["version"]).is_none());
        assert!(json_string("{}", &["version"]).is_none());
        assert!(json_string("not json", &["version"]).is_none());
    }

    #[test]
    fn a_lock_files_root_package_is_found_by_its_empty_key() {
        let lock = r#"{"name":"a","version":"1.0.0","lockfileVersion":3,"packages":{"":{"name":"a","version":"1.0.0","dependencies":{"x":"^1"}},"node_modules/x":{"version":"1.4.0"}}}"#;
        let (_, old) = json_string(lock, &["packages", "", "version"]).unwrap();
        assert_eq!(old, "1.0.0");
        let (edited, _) = rewrite(&Kind::Json(&[&["version"], &["packages", "", "version"]]), lock, "2.0.0");
        assert_eq!(edited.matches("2.0.0").count(), 2);
        assert!(edited.contains("\"node_modules/x\":{\"version\":\"1.4.0\"}"), "dependencies stay as locked");
    }

    #[test]
    fn toml_versions_keep_their_quotes_comments_and_line_endings() {
        let text = "# bach\r\n[app]\r\nname = \"x\"\r\nversion = \"1.0.0\" # keep\r\n\r\n[files]\r\nsource = \"build\"\r\nversion = \"nope\"\r\n";
        let (edited, olds) = rewrite(&Kind::Toml(&["app"]), text, "1.1.0");
        assert_eq!(olds, ["1.0.0"]);
        assert_eq!(edited, text.replace("1.0.0", "1.1.0"));
        let single = "[package]\nversion = '0.1.0'\n";
        assert_eq!(rewrite(&Kind::Toml(&["package"]), single, "0.2.0").0, "[package]\nversion = '0.2.0'\n");
        let inherited = "[package]\nversion.workspace = true\n";
        assert!(rewrite(&Kind::Toml(&["package"]), inherited, "1.0.0").1.is_empty(), "an inherited version isn't edited here");
    }

    #[test]
    fn a_project_is_brought_to_one_version() {
        let dir = folder(
            "sync",
            &[
                ("bach.toml", "[app]\nname = \"A\"\nversion = \"1.0.0\"\n"),
                ("package.json", "{\n  \"name\": \"a\",\n  \"version\": \"1.0.0\"\n}\n"),
                ("package-lock.json", "{\n  \"name\": \"a\",\n  \"version\": \"0.9.0\",\n  \"lockfileVersion\": 3,\n  \"packages\": {\n    \"\": {\n      \"name\": \"a\",\n      \"version\": \"0.9.0\"\n    }\n  }\n}\n"),
                ("src-tauri/tauri.conf.json", "{\"productName\":\"A\",\"version\":\"1.0.0\"}"),
                ("A.csproj", "<Project><PropertyGroup><Version>1.0.0</Version></PropertyGroup></Project>"),
            ],
        );
        assert_eq!(read_all(&dir).unwrap().iter().filter(|f| f.version == "1.0.0").count(), 4);
        let dry = write_all(&dir, "1.0.0", "1.2.0", true).unwrap();
        assert_eq!(dry.len(), 5);
        assert!(std::fs::read_to_string(dir.join("package.json")).unwrap().contains("1.0.0"), "a dry run writes nothing");
        write_all(&dir, "1.0.0", "1.2.0", false).unwrap();
        let after = read_all(&dir).unwrap();
        assert!(after.iter().all(|f| f.version == "1.2.0"), "{after:?}");
        assert_eq!(after.len(), 6, "the lock file has two versions");
        assert!(write_all(&dir, "1.2.0", "1.2.0", false).unwrap().is_empty(), "nothing to do twice");
        std::fs::remove_dir_all(dir).unwrap();
    }

    #[test]
    fn cargo_lock_only_changes_this_workspaces_crates() {
        let lock = "version = 4\n\n[[package]]\nname = \"app\"\nversion = \"0.3.0\"\ndependencies = [\"dep\"]\n\n[[package]]\nname = \"dep\"\nversion = \"0.3.0\"\nsource = \"registry+https://github.com/rust-lang/crates.io-index\"\nchecksum = \"abc\"\n\n[[package]]\nname = \"member\"\nversion = \"0.3.0\"\n";
        let dir = folder(
            "cargo",
            &[("Cargo.toml", "[workspace.package]\nversion = \"0.3.0\"\n\n[package]\nname = \"app\"\nversion.workspace = true\n"), ("Cargo.lock", lock)],
        );
        let changes = write_all(&dir, "0.3.0", "0.4.0", false).unwrap();
        assert_eq!(changes.iter().map(|c| c.file.as_str()).collect::<Vec<_>>(), ["Cargo.toml", "Cargo.lock"]);
        let after = std::fs::read_to_string(dir.join("Cargo.lock")).unwrap();
        assert_eq!(after.matches("0.4.0").count(), 2, "app and member");
        assert!(after.contains("name = \"dep\"\nversion = \"0.3.0\"\nsource"), "a dependency keeps its version");
        std::fs::remove_dir_all(dir).unwrap();
    }
}
