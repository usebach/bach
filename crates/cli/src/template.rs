//! `bach init`: a starter `bach.toml` that explains itself.

use std::path::Path;

use anyhow::{bail, Context, Result};

const TEMPLATE: &str = r##"# bach.toml: how Bach builds your installer.
# Build it with `bach build`; see what it installs with `bach check`;
# look at the window with `bach preview`. Paths are relative to this file.

[app]
name = "@NAME@"
version = "1.0.0"
publisher = "@NAME@"
# The program to start, inside the files folder below.
executable = "@EXE@.exe"
# description = "One line about your app."
# website = "https://example.com"
# The setup file's own icon (a .ico file).
# icon = "icon.ico"

[files]
# Everything in this folder is installed. Build your app into it first.
source = "build"
# exclude = ["*.pdb", "*.log"]

[installer]
# output = "dist/{name}-Setup-{version}.exe"
# theme = "system"          # system, light or dark
# accent = "#2563eb"        # buttons, the progress bar and ticked boxes
# logo = "logo.svg"         # .svg (one colour, follows the theme) or .png
# desktop_shortcut = true   # whether the box starts ticked

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
    let name = guess_name(dir);
    let exe = name.to_lowercase().replace(' ', "-");
    let text = TEMPLATE.replace("@NAME@", &name).replace("@EXE@", &exe);
    std::fs::write(&path, text).with_context(|| format!("couldn't write {}", path.display()))?;
    println!("Wrote {}. Fill in [app] and [files], then run `bach check`.", path.display());
    Ok(())
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
        let exe = name.to_lowercase().replace(' ', "-");
        std::fs::write(dir.join("build").join(format!("{exe}.exe")), b"MZ").unwrap();
        let project = bach_pack::config::load(&dir.join("bach.toml")).unwrap();
        assert_eq!(project.manifest.app.name, name);
        assert!(init(&dir).is_err(), "init must not overwrite");
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
