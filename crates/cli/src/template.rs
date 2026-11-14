//! `bach init`: a starter `bach.toml` that explains itself.

use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::detect::Detected;

const TEMPLATE: &str = r##"#:schema https://raw.githubusercontent.com/usebach/bach/main/schema/bach.schema.json
# bach.toml: how Bach builds your installer.
# Build it with `bach build`; see what it installs with `bach check`;
# look at the window with `bach preview`. Paths are relative to this file.
# Every setting: https://github.com/usebach/bach/blob/main/docs/config.md

[app]
name = "@NAME@"
version = "@VERSION@"
publisher = "@PUBLISHER@"
# The program to start, inside the files folder below.
executable = "@EXE@"
# description = "One line about your app."   # also the install page's subtitle
# website = "https://example.com"
# icon = "icon.ico"           # the setup file's icon (and the window's logo)

[files]
# Everything in this folder is installed. Build your app into it first.
source = "@SOURCE@"
@EXCLUDE@

[installer]
# output = "dist/{name}-Setup-{version}.exe"
# theme = "system"            # system, light or dark
# accent = "#2563eb"          # buttons, the progress bar and ticked boxes
# logo = "logo.svg"           # .svg (one colour) or .png, or "none"
# languages = ["en", "de"]    # default: all 13, following Windows' language
# license = "LICENSE.txt"     # shown first, to accept before installing
#
# Each option is true (offered, ticked), false (offered, unticked),
# "always" (done without asking) or "never".
# desktop_shortcut = true
# start_menu_shortcut = "always"
# start_with_windows = "never"
# add_to_path = "never"       # for command-line tools
# path_folder = "bin"         # what goes on PATH (default: the install folder)
# change_folder = true        # may the user pick another folder?
# open_button = true          # offer "Open @NAME@" at the end?
# start_menu_folder = "@PUBLISHER@"  # put the shortcuts in a folder

@REQUIREMENTS@# Updates: the installed app runs `uninstall.exe --update` to fetch them.
# [updates]
# feed = "https://example.com/@SCHEME@/latest.json"
# download_url = "https://example.com/@SCHEME@/{file}"

# Optional parts the user can leave out:
# [[components]]
# id = "samples"
# title = "Sample projects"
# description = "Examples to learn from."
# files = ["samples/**"]
# default = true

# Links like @SCHEME@://open start your app with the link:
# [[protocols]]
# scheme = "@SCHEME@"

# Files your app opens, offered under "Open with":
# [[file_types]]
# extension = ".@SCHEME@"
# description = "@NAME@ document"

# Any text in the window, in one language or per language:
# [text]
# install_title = { en = "Get {name}", de = "Hol dir {name}" }
# footer = "Made with care by {publisher}"

[uninstall]
# Offered as "Also delete my data" when uninstalling. Each folder must start
# with %APPDATA%, %LOCALAPPDATA% or %USERPROFILE%.
# data = ['%APPDATA%\@NAME@']
# data_description = "Your settings and saved files."
"##;

pub fn init(dir: &Path) -> Result<()> {
    let path = dir.join("bach.toml");
    if path.exists() {
        bail!("{} already exists; Bach won't overwrite it", path.display());
    }
    std::fs::create_dir_all(dir).with_context(|| format!("couldn't create {}", dir.display()))?;
    let detected = crate::detect::detect(dir);
    let text = fill(detected.as_ref(), &guess_name(dir));
    std::fs::write(&path, text).with_context(|| format!("couldn't write {}", path.display()))?;
    match &detected {
        Some(found) => println!(
            "Wrote {} for this {} project. Check [app] and [files], then run `bach check`.",
            path.display(),
            found.stack
        ),
        None => println!("Wrote {}. Fill in [app] and [files], then run `bach check`.", path.display()),
    }
    Ok(())
}

/// The template with what the project says about itself, or placeholders
/// from the folder's name.
fn fill(found: Option<&Detected>, folder_name: &str) -> String {
    let empty = Detected::default();
    let found = found.unwrap_or(&empty);
    let name = found.name.clone().unwrap_or_else(|| folder_name.to_string());
    let scheme = executable_name(&name);
    let exclude = if found.exclude.is_empty() {
        "# exclude = [\"*.pdb\", \"*.log\"]".to_string()
    } else {
        let patterns: Vec<String> = found.exclude.iter().map(|pattern| format!("\"{pattern}\"")).collect();
        format!("exclude = [{}]", patterns.join(", "))
    };
    let requirements = found.requirements.clone().map(|block| format!("{block}\n")).unwrap_or_default();
    TEMPLATE
        .replace("@NAME@", &name)
        .replace("@VERSION@", found.version.as_deref().unwrap_or("1.0.0"))
        .replace("@PUBLISHER@", found.publisher.as_deref().unwrap_or(&name))
        .replace("@EXE@", &found.executable.clone().unwrap_or_else(|| format!("{scheme}.exe")))
        .replace("@SOURCE@", found.source.as_deref().unwrap_or("build"))
        .replace("@EXCLUDE@", &exclude)
        .replace("@REQUIREMENTS@", &requirements)
        .replace("@SCHEME@", &scheme)
}

/// "My Cool App" → "my-cool-app": also a usable link scheme and extension.
fn executable_name(name: &str) -> String {
    let slug: String = name.to_lowercase().chars().map(|c| if c.is_ascii_alphanumeric() { c } else { '-' }).collect();
    let slug = slug.split('-').filter(|part| !part.is_empty()).collect::<Vec<_>>().join("-");
    if slug.chars().next().is_some_and(|c| c.is_ascii_lowercase()) {
        slug
    } else {
        format!("app-{slug}")
    }
}

/// The folder's name, tidied into something that works as an app name.
fn guess_name(dir: &Path) -> String {
    let folder = std::fs::canonicalize(dir)
        .ok()
        .and_then(|path| path.file_name().map(|name| name.to_string_lossy().into_owned()))
        .unwrap_or_default();
    let cleaned: String = folder
        .chars()
        .map(|c| if c.is_alphanumeric() || c == ' ' { c } else { ' ' })
        .collect::<String>()
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            chars.next().map(|first| first.to_uppercase().chain(chars).collect::<String>()).unwrap_or_default()
        })
        .collect::<Vec<_>>()
        .join(" ");
    if cleaned.is_empty() {
        "My App".into()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_template_is_a_valid_config() {
        let dir = std::env::temp_dir().join(format!("bach-init-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("build")).unwrap();
        init(&dir).unwrap();
        let name = guess_name(&dir);
        let exe = executable_name(&name);
        std::fs::write(dir.join("build").join(format!("{exe}.exe")), b"MZ").unwrap();
        let text = std::fs::read_to_string(dir.join("bach.toml")).unwrap();
        assert!(text.starts_with("#:schema "), "editors find the schema");
        let project = bach_pack::config::load(&dir.join("bach.toml")).unwrap();
        assert_eq!(project.manifest.app.name, name);
        assert!(init(&dir).is_err(), "init must not overwrite");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    /// A detected Rust project gets a config that builds as soon as the
    /// release build is there.
    #[test]
    fn detected_projects_fill_the_template() {
        let dir = std::env::temp_dir().join(format!("bach-init-rust-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(dir.join("target/release/deps")).unwrap();
        std::fs::write(dir.join("Cargo.toml"), "[package]\nname = \"clip-board\"\nversion = \"0.4.0\"\n").unwrap();
        std::fs::write(dir.join("target/release/clip-board.exe"), b"MZ").unwrap();
        std::fs::write(dir.join("target/release/deps/junk.rlib"), b"x").unwrap();
        init(&dir).unwrap();
        let project = bach_pack::config::load(&dir.join("bach.toml")).unwrap();
        assert_eq!(project.manifest.app.name, "Clip Board");
        assert_eq!(project.manifest.app.version, "0.4.0");
        assert_eq!(project.manifest.files.len(), 1, "build leftovers are excluded");
        std::fs::remove_dir_all(&dir).unwrap();
    }

    #[test]
    fn names_are_tidied() {
        let dir = std::env::temp_dir().join(format!("my_cool-app-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert!(guess_name(&dir).starts_with("My Cool App"));
        std::fs::remove_dir_all(&dir).unwrap();
    }
}
