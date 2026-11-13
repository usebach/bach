//! `installer.languages`, `installer.license` and `[text]`: what the window
//! says, and in which languages.

use std::collections::BTreeMap;
use std::path::Path;

use anyhow::{bail, Context, Result};

use crate::i18n::{self, Key, Localized, ANY_LANGUAGE};
use crate::manifest::Locale;

/// The languages the app offers, checked. Empty means every built-in one.
pub fn languages(value: Option<&[String]>) -> Result<Vec<String>> {
    let Some(list) = value else {
        return Ok(Vec::new());
    };
    if list.is_empty() {
        bail!("installer.languages is empty; leave it out to use every language, or list some like [\"en\", \"de\"]");
    }
    let mut out: Vec<String> = Vec::new();
    for code in list {
        let code = code.trim().to_ascii_lowercase();
        if i18n::language(&code).is_none() {
            bail!("installer.languages has \"{code}\", which Bach doesn't speak; use {}", known_codes());
        }
        if out.contains(&code) {
            bail!("installer.languages lists \"{code}\" twice");
        }
        out.push(code);
    }
    Ok(out)
}

fn known_codes() -> String {
    i18n::LANGUAGES.iter().map(|language| format!("\"{}\"", language.code)).collect::<Vec<_>>().join(", ")
}

/// A text in one language for all (`"Hello"`) or per language
/// (`{ en = "Hello", de = "Hallo" }`).
pub fn localized(value: &toml::Value, key: &str, languages: &[String]) -> Result<Localized> {
    match value {
        toml::Value::String(text) => {
            if text.trim().is_empty() && !key.starts_with("text.") {
                bail!("{key} is empty");
            }
            Ok([(ANY_LANGUAGE.to_string(), text.trim().to_string())].into())
        }
        toml::Value::Table(table) => {
            if table.is_empty() {
                bail!("{key} is an empty table; give at least one language, like {{ en = \"…\" }}");
            }
            let mut out = Localized::new();
            for (code, text) in table {
                let code = code.trim().to_ascii_lowercase();
                if i18n::language(&code).is_none() {
                    bail!("{key}.{code} isn't a language Bach speaks; use {}", known_codes());
                }
                if !languages.is_empty() && !languages.contains(&code) {
                    bail!("{key}.{code} is a language installer.languages leaves out");
                }
                let Some(text) = text.as_str() else {
                    bail!("{key}.{code} must be text in quotes");
                };
                out.insert(code, text.trim().to_string());
            }
            Ok(out)
        }
        _ => bail!("{key} must be text in quotes, or a table of texts by language like {{ en = \"…\", de = \"…\" }}"),
    }
}

/// `[text]`: replacements for the built-in texts.
pub fn locale(text: &BTreeMap<String, toml::Value>, languages: &[String], warnings: &mut Vec<String>) -> Result<Locale> {
    let mut out = BTreeMap::new();
    let mut untranslated = Vec::new();
    for (name, value) in text {
        let at = format!("text.{name}");
        let Some(key) = Key::parse(name) else {
            bail!("{at} isn't a text the installer shows; docs/config.md lists them all");
        };
        let localized = localized(value, &at, languages)?;
        for text in localized.values() {
            for placeholder in i18n::placeholders(text) {
                let known = i18n::COMMON_PLACEHOLDERS.contains(&placeholder)
                    || i18n::LANGUAGES[0].strings.get(key).contains(&format!("{{{placeholder}}}"))
                    || (key == Key::install_subtitle && placeholder == "description");
                if !known {
                    bail!("{at} uses {{{placeholder}}}, which this text can't fill in; use {{name}}, {{version}} or {{publisher}}");
                }
            }
        }
        if localized.contains_key(ANY_LANGUAGE) && languages.len() != 1 {
            untranslated.push(at);
        }
        out.insert(key.name().to_string(), localized);
    }
    if !untranslated.is_empty() {
        let offered = if languages.is_empty() { i18n::LANGUAGES.len() } else { languages.len() };
        warnings.push(format!(
            "{} {} one language, but the installer speaks {offered}; translate {} like {{ en = \"…\", de = \"…\" }}, \
             or set installer.languages = [\"en\"] to use one language",
            untranslated.join(", "),
            if untranslated.len() == 1 { "is in" } else { "are in" },
            if untranslated.len() == 1 { "it" } else { "them" },
        ));
    }
    Ok(Locale { languages: languages.to_vec(), text: out, license: None })
}

/// `installer.license`: a text file, or one per language.
pub fn license(value: Option<&toml::Value>, root: &Path, languages: &[String]) -> Result<Option<Localized>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let files = localized(value, "installer.license", languages)?;
    let mut out = Localized::new();
    for (code, relative) in files {
        let at = if code == ANY_LANGUAGE { "installer.license".to_string() } else { format!("installer.license.{code}") };
        let path = root.join(&relative);
        if !path.is_file() {
            bail!("{at} is \"{relative}\", but {} isn't a file", path.display());
        }
        let bytes = std::fs::read(&path).with_context(|| format!("couldn't read {}", path.display()))?;
        if bytes.len() > crate::manifest::MAX_LICENSE {
            bail!("{at} is longer than {} KB; use a plain-text summary", crate::manifest::MAX_LICENSE / 1024);
        }
        let text = String::from_utf8(bytes)
            .map_err(|_| anyhow::anyhow!("{at} ({relative}) isn't UTF-8 text; save it as UTF-8"))?;
        let text = text.trim_start_matches('\u{feff}').replace("\r\n", "\n").trim().to_string();
        if text.is_empty() {
            bail!("{at} ({relative}) is empty");
        }
        out.insert(code, text);
    }
    Ok(Some(out))
}
