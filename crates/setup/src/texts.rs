//! The window's words: the built-in texts in the chosen language, with the
//! developer's own texts from `[text]` on top and the app's name, version
//! and publisher filled in.

use std::collections::HashMap;

use bach_pack::i18n::{self, Key, Language};
use bach_pack::manifest::{Component, Manifest};

pub struct Texts {
    pub language: &'static Language,
    custom: HashMap<Key, String>,
    values: Vec<(&'static str, String)>,
}

impl Texts {
    /// Texts in the language the user prefers (or `forced`, from `--lang`),
    /// among those the app offers.
    pub fn new(manifest: &Manifest, forced: Option<&str>) -> Self {
        let preferred = match forced {
            Some(code) => vec![code.to_string()],
            None => crate::system::host::ui_languages(),
        };
        let language = i18n::pick(&manifest.locale.languages, &preferred);
        Self::in_language(manifest, language)
    }

    pub fn in_language(manifest: &Manifest, language: &'static Language) -> Self {
        let custom = Key::ALL
            .iter()
            .filter_map(|key| manifest.custom_text(*key, language.code).map(|text| (*key, text.to_string())))
            .collect();
        let app = &manifest.app;
        let values = vec![
            ("name", app.name.clone()),
            ("version", app.version.clone()),
            ("publisher", app.publisher.clone()),
            ("description", app.description.clone().unwrap_or_default()),
        ];
        Self { language, custom, values }
    }

    /// The text for `key`, with the app's details filled in.
    pub fn get(&self, key: Key) -> String {
        self.with(key, &[])
    }

    /// The text for `key`, with `extra` placeholders filled in as well.
    pub fn with(&self, key: Key, extra: &[(&str, &str)]) -> String {
        let template = self.custom.get(&key).map(String::as_str).unwrap_or_else(|| self.language.strings.get(key));
        let mut values: Vec<(&str, &str)> = extra.to_vec();
        values.extend(self.values.iter().map(|(name, value)| (*name, value.as_str())));
        i18n::fill(template, &values)
    }

    /// A component's title or description in this language.
    pub fn component(&self, component: &Component, manifest: &Manifest) -> (String, Option<String>) {
        let code = self.language.code;
        let fallbacks = &manifest.locale.languages;
        let fill = |text: &str| i18n::fill(text, &self.values.iter().map(|(k, v)| (*k, v.as_str())).collect::<Vec<_>>());
        let title = i18n::resolve(&component.title, code, fallbacks).map(fill).unwrap_or_else(|| component.id.clone());
        let description = component
            .description
            .as_ref()
            .and_then(|text| i18n::resolve(text, code, fallbacks))
            .map(fill);
        (title, description)
    }
}

/// A byte count people can read: "1.5 MB".
pub fn size(bytes: u64) -> String {
    const UNITS: [&str; 4] = ["KB", "MB", "GB", "TB"];
    let mut value = bytes as f64 / 1024.0;
    let mut unit = 0;
    while value >= 1024.0 && unit < UNITS.len() - 1 {
        value /= 1024.0;
        unit += 1;
    }
    format!("{value:.1} {}", UNITS[unit])
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_read_naturally() {
        assert_eq!(size(1536), "1.5 KB");
        assert_eq!(size(5 * 1024 * 1024), "5.0 MB");
        assert_eq!(size(3 << 30), "3.0 GB");
    }
}
