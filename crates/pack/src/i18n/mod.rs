//! Every sentence the setup window shows, in every language Bach speaks.
//!
//! [`Key`] names each text; [`Strings`] holds one language's texts, so a
//! language that forgets a text doesn't compile. `bach.toml` can replace any
//! text (`[text]`), in one language or per language, and limit the window to
//! some languages (`installer.languages`). At run time the setup window picks
//! the user's Windows display language from those, falling back to the
//! first one the app lists.
//!
//! Texts may use placeholders: `{name}`, `{version}`, `{publisher}` always,
//! and a few that only make sense in one text (`{old_version}`, `{path}`,
//! `{size}`, `{free}`, `{error}`). [`fill`] puts the values in.

mod asian;
mod eastern;
mod en;
mod germanic;
mod romance;

use std::collections::BTreeMap;

/// Text per language code; the key `*` stands for every language.
pub type Localized = BTreeMap<String, String>;

/// The key a single, untranslated text is stored under.
pub const ANY_LANGUAGE: &str = "*";

macro_rules! texts {
    ($($key:ident),* $(,)?) => {
        /// The name of one text the window shows.
        #[allow(non_camel_case_types)]
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub enum Key { $($key),* }

        impl Key {
            /// Every key, in declaration order.
            pub const ALL: &'static [Key] = &[$(Key::$key),*];

            /// The key as written in `bach.toml` (`[text] install_title`).
            pub fn name(self) -> &'static str {
                match self { $(Key::$key => stringify!($key)),* }
            }

            pub fn parse(name: &str) -> Option<Key> {
                match name { $(stringify!($key) => Some(Key::$key),)* _ => None }
            }
        }

        /// One language's texts.
        pub struct Strings { $(pub $key: &'static str),* }

        impl Strings {
            pub fn get(&self, key: Key) -> &'static str {
                match key { $(Key::$key => self.$key),* }
            }
        }
    };
}

texts! {
    window_install, window_remove,
    install_title, install_subtitle, install_button,
    reinstall_title, reinstall_subtitle, reinstall_button,
    update_title, update_subtitle, update_button,
    remove_title, remove_subtitle, remove_button,
    license_title, license_subtitle, license_accept,
    cancel, close, retry, change, open, open_short, footer, preview,
    location, choose_folder,
    desktop_shortcut, desktop_shortcut_description,
    start_menu, start_menu_description,
    start_with_windows, start_with_windows_description,
    add_to_path, add_to_path_description,
    delete_data, delete_data_description,
    installing, updating, removing,
    step_preparing, step_copying, step_tidying, step_uninstaller, step_shortcuts, step_registering,
    step_removing, step_removing_shortcuts, step_unregistering, step_deleting_data, step_cleaning, step_done,
    installed_title, installed_subtitle, installed_subtitle_plain, updated_title,
    removed_title, removed_subtitle, removed_data_deleted, removed_data_kept,
    install_failed, update_failed, remove_failed,
    error_still_open, error_in_use, error_busy, error_disk_full, error_no_access,
    error_create, error_write, error_delete, error_read_setup, error_damaged,
    error_uninstaller, error_shortcut, error_register, error_unregister, error_system,
    error_delete_data, error_not_found, error_open_failed,
    problem_title, problem_damaged,
    downgrade_title, downgrade_subtitle, downgrade_button, close_app, error_close_app,
    requirement_title, requirement_subtitle, requirement_button, step_downloading, error_download,
    website, feedback,
}

/// A language the setup window speaks.
pub struct Language {
    /// A short code like `de` or `zh` (simplified Chinese).
    pub code: &'static str,
    /// The language's own name for itself.
    pub name: &'static str,
    pub strings: &'static Strings,
}

/// Every built-in language. English comes first: it's the last resort.
pub const LANGUAGES: &[Language] = &[
    Language { code: "en", name: "English", strings: &en::EN },
    Language { code: "de", name: "Deutsch", strings: &germanic::DE },
    Language { code: "nl", name: "Nederlands", strings: &germanic::NL },
    Language { code: "fr", name: "Français", strings: &romance::FR },
    Language { code: "es", name: "Español", strings: &romance::ES },
    Language { code: "it", name: "Italiano", strings: &romance::IT },
    Language { code: "pt", name: "Português", strings: &romance::PT },
    Language { code: "pl", name: "Polski", strings: &eastern::PL },
    Language { code: "ru", name: "Русский", strings: &eastern::RU },
    Language { code: "tr", name: "Türkçe", strings: &eastern::TR },
    Language { code: "ja", name: "日本語", strings: &asian::JA },
    Language { code: "ko", name: "한국어", strings: &asian::KO },
    Language { code: "zh", name: "简体中文", strings: &asian::ZH },
];

/// The built-in language with this code, if Bach speaks it.
pub fn language(code: &str) -> Option<&'static Language> {
    LANGUAGES.iter().find(|language| language.code.eq_ignore_ascii_case(code))
}

/// Placeholders every text may use.
pub const COMMON_PLACEHOLDERS: &[&str] = &["name", "version", "publisher"];
/// Placeholders some texts fill in; harmless (left as is) elsewhere.
pub const OTHER_PLACEHOLDERS: &[&str] = &["old_version", "path", "size", "free", "error", "description", "requirement"];

/// Picks the language to show: the first of the user's `preferred`
/// languages (like `de-DE`, most preferred first) that the app `offers`,
/// matched exactly or by its first part, else the app's first language.
/// An empty `offers` means every built-in language.
pub fn pick(offers: &[String], preferred: &[String]) -> &'static Language {
    let all: Vec<String>;
    let offers: &[String] = if offers.is_empty() {
        all = LANGUAGES.iter().map(|language| language.code.to_string()).collect();
        &all
    } else {
        offers
    };
    for wanted in preferred {
        let wanted = wanted.trim().replace('_', "-").to_ascii_lowercase();
        let primary = wanted.split('-').next().unwrap_or_default();
        let found = offers
            .iter()
            .find(|code| code.eq_ignore_ascii_case(&wanted))
            .or_else(|| offers.iter().find(|code| code.eq_ignore_ascii_case(primary)))
            .and_then(|code| language(code));
        if let Some(found) = found {
            return found;
        }
    }
    offers.iter().find_map(|code| language(code)).unwrap_or(&LANGUAGES[0])
}

/// The text for `language` out of a localized one: its own, then the
/// any-language text, then the fallbacks in order.
pub fn resolve<'a>(text: &'a Localized, language: &str, fallbacks: &[String]) -> Option<&'a str> {
    text.get(language)
        .or_else(|| text.get(ANY_LANGUAGE))
        .or_else(|| fallbacks.iter().find_map(|code| text.get(code)))
        .or_else(|| text.get("en"))
        .or_else(|| text.values().next())
        .map(String::as_str)
}

/// Replaces `{placeholder}`s with their values; unknown ones stay as they are.
pub fn fill(template: &str, values: &[(&str, &str)]) -> String {
    let mut out = String::with_capacity(template.len() + 16);
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        out.push_str(&rest[..open]);
        let after = &rest[open + 1..];
        match after.find('}') {
            Some(close) if is_placeholder_name(&after[..close]) => {
                let name = &after[..close];
                match values.iter().find(|(key, _)| *key == name) {
                    Some((_, value)) => out.push_str(value),
                    None => {
                        out.push('{');
                        out.push_str(name);
                        out.push('}');
                    }
                }
                rest = &after[close + 1..];
            }
            _ => {
                out.push('{');
                rest = after;
            }
        }
    }
    out.push_str(rest);
    out
}

/// The placeholders a text uses, for checking a config's texts.
pub fn placeholders(template: &str) -> Vec<&str> {
    let mut found = Vec::new();
    let mut rest = template;
    while let Some(open) = rest.find('{') {
        let after = &rest[open + 1..];
        match after.find('}') {
            Some(close) if is_placeholder_name(&after[..close]) => {
                found.push(&after[..close]);
                rest = &after[close + 1..];
            }
            _ => rest = after,
        }
    }
    found
}

fn is_placeholder_name(name: &str) -> bool {
    !name.is_empty() && name.chars().all(|c| c.is_ascii_lowercase() || c == '_')
}

#[cfg(test)]
mod tests {
    use super::*;

    fn codes(list: &[&str]) -> Vec<String> {
        list.iter().map(|code| code.to_string()).collect()
    }

    #[test]
    fn every_language_has_every_text() {
        for language in LANGUAGES {
            for key in Key::ALL {
                let text = language.strings.get(*key);
                // Only the install subtitle may be empty: it falls back to the
                // app's description.
                if *key != Key::install_subtitle {
                    assert!(!text.trim().is_empty(), "{} has no {}", language.code, key.name());
                }
                for name in placeholders(text) {
                    assert!(
                        COMMON_PLACEHOLDERS.contains(&name) || OTHER_PLACEHOLDERS.contains(&name),
                        "{}.{} uses {{{name}}}",
                        language.code,
                        key.name()
                    );
                }
            }
        }
    }

    #[test]
    fn translations_keep_the_english_placeholders() {
        let english = &LANGUAGES[0];
        for language in &LANGUAGES[1..] {
            for key in Key::ALL {
                let mut want = placeholders(english.strings.get(*key));
                let mut have = placeholders(language.strings.get(*key));
                want.sort_unstable();
                have.sort_unstable();
                want.dedup();
                have.dedup();
                assert_eq!(want, have, "{}.{}", language.code, key.name());
            }
        }
    }

    #[test]
    fn languages_are_picked_like_windows_would() {
        assert_eq!(pick(&[], &codes(&["de-DE"])).code, "de");
        assert_eq!(pick(&[], &codes(&["pt-BR", "en-US"])).code, "pt");
        assert_eq!(pick(&[], &codes(&["sv-SE", "fr-FR"])).code, "fr");
        assert_eq!(pick(&[], &codes(&["sv-SE"])).code, "en");
        // An app that only speaks German speaks German to everyone.
        assert_eq!(pick(&codes(&["de"]), &codes(&["en-US"])).code, "de");
        assert_eq!(pick(&codes(&["fr", "de"]), &codes(&["de-AT"])).code, "de");
        assert_eq!(pick(&codes(&["fr", "de"]), &codes(&["ja-JP"])).code, "fr");
    }

    #[test]
    fn placeholders_fill_in() {
        let values = [("name", "Hello"), ("version", "2.0")];
        assert_eq!(fill("Install {name} {version}", &values), "Install Hello 2.0");
        assert_eq!(fill("{unknown} stays, { not one }", &values), "{unknown} stays, { not one }");
        assert_eq!(fill("trailing {", &values), "trailing {");
        assert_eq!(placeholders("{name} and {old_version} {}"), ["name", "old_version"]);
    }

    #[test]
    fn localized_texts_fall_back() {
        let mut text = Localized::new();
        text.insert("de".into(), "Hallo".into());
        text.insert("fr".into(), "Bonjour".into());
        assert_eq!(resolve(&text, "de", &[]), Some("Hallo"));
        assert_eq!(resolve(&text, "ja", &codes(&["fr"])), Some("Bonjour"));
        text.insert(ANY_LANGUAGE.into(), "Hi".into());
        assert_eq!(resolve(&text, "ja", &codes(&["fr"])), Some("Hi"));
    }
}
