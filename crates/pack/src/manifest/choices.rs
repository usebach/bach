//! The choices the install page offers, and what the user picked.

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use super::{FileEntry, Manifest};

/// Option ids, for `--with` and `--without` on a silent install. Component
/// ids share this namespace, so they can't use these.
pub const OPTION_DESKTOP: &str = "desktop-shortcut";
pub const OPTION_START_MENU: &str = "start-menu";
pub const OPTION_AUTOSTART: &str = "autostart";
pub const OPTION_PATH: &str = "path";
pub const OPTION_AUTO_UPDATE: &str = "auto-update";

/// How the installer treats one feature.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Choice {
    /// Not done, and not offered.
    #[default]
    Never,
    /// Offered, unticked to begin with.
    Off,
    /// Offered, ticked to begin with.
    On,
    /// Done without asking.
    Always,
}

impl Choice {
    pub fn always() -> Self {
        Self::Always
    }

    /// Whether the install page shows a box for it.
    pub fn offered(self) -> bool {
        matches!(self, Self::Off | Self::On)
    }

    /// Whether it happens unless the user unticks it.
    pub fn starts_on(self) -> bool {
        matches!(self, Self::On | Self::Always)
    }

    /// `true`, `false`, `"always"` or `"never"` (also `"on"` and `"off"`).
    pub fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "never" => Some(Self::Never),
            "off" | "false" => Some(Self::Off),
            "on" | "true" => Some(Self::On),
            "always" => Some(Self::Always),
            _ => None,
        }
    }

    fn word(self) -> &'static str {
        match self {
            Self::Never => "never",
            Self::Off => "off",
            Self::On => "on",
            Self::Always => "always",
        }
    }

    /// The user's answer, given the box's state (ignored unless offered).
    pub fn decide(self, ticked: bool) -> bool {
        match self {
            Self::Never => false,
            Self::Always => true,
            Self::Off | Self::On => ticked,
        }
    }
}

impl Serialize for Choice {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.word())
    }
}

impl<'de> Deserialize<'de> for Choice {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum Raw {
            Bool(bool),
            Word(String),
        }
        match Raw::deserialize(deserializer)? {
            Raw::Bool(true) => Ok(Self::On),
            Raw::Bool(false) => Ok(Self::Off),
            Raw::Word(word) => Self::parse(&word)
                .ok_or_else(|| serde::de::Error::custom(format!("\"{word}\" isn't on, off, always or never"))),
        }
    }
}

/// How the installer treats automatic updates: `Never` without `[updates]`.
pub fn auto_update_setting(manifest: &Manifest) -> Choice {
    manifest.updates.as_ref().map_or(Choice::Never, |updates| updates.auto)
}

/// What the user picked on the install page (or on the command line).
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Choices {
    pub desktop_shortcut: bool,
    pub start_menu_shortcut: bool,
    pub start_with_windows: bool,
    pub add_to_path: bool,
    /// Keep the app up to date in the background (`[updates] auto`). Older
    /// uninstallers don't have it, which reads as not chosen.
    #[serde(default)]
    pub auto_update: bool,
    /// The optional components to install, by id.
    pub components: Vec<String>,
}

impl Choices {
    /// What a first install starts with.
    pub fn defaults(manifest: &Manifest) -> Self {
        let install = &manifest.install;
        Self {
            desktop_shortcut: install.desktop_shortcut.starts_on(),
            start_menu_shortcut: install.start_menu_shortcut.starts_on(),
            start_with_windows: install.start_with_windows.starts_on(),
            add_to_path: install.add_to_path.starts_on(),
            auto_update: auto_update_setting(manifest).starts_on(),
            components: manifest
                .components
                .iter()
                .filter(|component| component.default)
                .map(|component| component.id.clone())
                .collect(),
        }
    }

    /// What an update starts with: the earlier answers for whatever was
    /// offered before, the defaults for anything new.
    pub fn for_update(manifest: &Manifest, previous: &Manifest) -> Self {
        let mut choices = Self::defaults(manifest);
        let Some(before) = &previous.chosen else {
            return choices;
        };
        // Something that never happened before has no answer to keep.
        let was_offered = |choice: Choice| choice != Choice::Never;
        let old = &previous.install;
        if was_offered(old.desktop_shortcut) {
            choices.desktop_shortcut = before.desktop_shortcut;
        }
        if was_offered(old.start_menu_shortcut) {
            choices.start_menu_shortcut = before.start_menu_shortcut;
        }
        if was_offered(old.start_with_windows) {
            choices.start_with_windows = before.start_with_windows;
        }
        if was_offered(old.add_to_path) {
            choices.add_to_path = before.add_to_path;
        }
        if previous.updates.as_ref().is_some_and(|updates| was_offered(updates.auto)) {
            choices.auto_update = before.auto_update;
        }
        choices.components = manifest
            .components
            .iter()
            .filter(|component| {
                let known = previous.components.iter().any(|old| old.id == component.id);
                if known {
                    before.components.contains(&component.id)
                } else {
                    component.default
                }
            })
            .map(|component| component.id.clone())
            .collect();
        choices
    }

    /// Settles every choice against what the manifest allows: `always`
    /// features are on, `never` ones off, unknown components dropped.
    pub fn settle(mut self, manifest: &Manifest) -> Self {
        let install = &manifest.install;
        self.desktop_shortcut = install.desktop_shortcut.decide(self.desktop_shortcut);
        self.start_menu_shortcut = install.start_menu_shortcut.decide(self.start_menu_shortcut);
        self.start_with_windows = install.start_with_windows.decide(self.start_with_windows);
        self.add_to_path = install.add_to_path.decide(self.add_to_path);
        self.auto_update = auto_update_setting(manifest).decide(self.auto_update);
        self.components.retain(|id| manifest.components.iter().any(|component| &component.id == id));
        self.components.sort();
        self.components.dedup();
        self
    }

    /// Turns an option on or off by id. False when the id isn't one this
    /// manifest offers.
    pub fn set(&mut self, manifest: &Manifest, id: &str, on: bool) -> bool {
        let install = &manifest.install;
        let flag = match id {
            OPTION_DESKTOP if install.desktop_shortcut.offered() => &mut self.desktop_shortcut,
            OPTION_START_MENU if install.start_menu_shortcut.offered() => &mut self.start_menu_shortcut,
            OPTION_AUTOSTART if install.start_with_windows.offered() => &mut self.start_with_windows,
            OPTION_PATH if install.add_to_path.offered() => &mut self.add_to_path,
            OPTION_AUTO_UPDATE if auto_update_setting(manifest).offered() => &mut self.auto_update,
            _ => {
                if !manifest.components.iter().any(|component| component.id == id) {
                    return false;
                }
                self.components.retain(|existing| existing != id);
                if on {
                    self.components.push(id.to_string());
                }
                return true;
            }
        };
        *flag = on;
        true
    }

    /// Whether these choices install `file`.
    pub fn installs(&self, file: &FileEntry) -> bool {
        file.component.as_ref().is_none_or(|component| self.components.contains(component))
    }
}

#[cfg(test)]
mod tests {
    use super::super::sample;
    use super::*;

    #[test]
    fn choices_read_from_bools_and_words() {
        let parse = |json: &str| serde_json::from_str::<Choice>(json).unwrap();
        assert_eq!(parse("true"), Choice::On);
        assert_eq!(parse("false"), Choice::Off);
        assert_eq!(parse("\"always\""), Choice::Always);
        assert_eq!(parse("\"never\""), Choice::Never);
        assert!(serde_json::from_str::<Choice>("\"sometimes\"").is_err());
        assert_eq!(serde_json::to_string(&Choice::On).unwrap(), "\"on\"");
    }

    #[test]
    fn updates_keep_earlier_answers() {
        let manifest = sample();
        let mut previous = sample();
        let mut before = Choices::defaults(&manifest);
        before.desktop_shortcut = false;
        before.start_with_windows = true;
        before.components = vec!["samples".into()];
        previous.chosen = Some(before);
        let choices = Choices::for_update(&manifest, &previous);
        assert!(!choices.desktop_shortcut);
        assert!(choices.start_with_windows);
        assert_eq!(choices.components, ["samples"]);
    }

    #[test]
    fn settling_respects_always_and_never() {
        let manifest = sample();
        let choices = Choices {
            desktop_shortcut: true,
            start_menu_shortcut: false,
            start_with_windows: true,
            add_to_path: true,
            auto_update: true,
            components: vec!["samples".into(), "ghost".into()],
        }
        .settle(&manifest);
        assert!(choices.start_menu_shortcut, "always stays on");
        assert!(!choices.add_to_path, "never stays off");
        assert!(!choices.auto_update, "no [updates], no automatic updates");
        assert_eq!(choices.components, ["samples"]);
    }

    #[test]
    fn automatic_updates_follow_the_updates_section() {
        use super::super::Updates;
        let mut manifest = sample();
        assert!(!Choices::defaults(&manifest).auto_update, "no [updates], no automatic updates");
        manifest.updates = Some(Updates { feed: "https://x/latest.json".into(), auto: Choice::On });
        let mut choices = Choices::defaults(&manifest);
        assert!(choices.auto_update, "offered and ticked");
        assert!(choices.set(&manifest, OPTION_AUTO_UPDATE, false));
        assert!(!choices.auto_update);
        // An update remembers the answer...
        let mut previous = manifest.clone();
        previous.chosen = Some(choices);
        assert!(!Choices::for_update(&manifest, &previous).auto_update);
        // ...but a version that never offered it asks fresh, ticked.
        previous.updates = Some(Updates { feed: "https://x/latest.json".into(), auto: Choice::Never });
        assert!(Choices::for_update(&manifest, &previous).auto_update);
        manifest.updates = Some(Updates { feed: "https://x/latest.json".into(), auto: Choice::Never });
        assert!(!Choices { auto_update: true, ..Choices::default() }.settle(&manifest).auto_update);
    }

    #[test]
    fn manifests_from_before_automatic_updates_still_read() {
        let choices: Choices = serde_json::from_str(
            r#"{"desktop_shortcut":true,"start_menu_shortcut":true,"start_with_windows":false,"add_to_path":false,"components":[]}"#,
        )
        .unwrap();
        assert!(!choices.auto_update);
        let updates: super::super::Updates = serde_json::from_str(r#"{"feed":"https://x/latest.json"}"#).unwrap();
        assert_eq!(updates.auto, Choice::Never);
    }

    #[test]
    fn options_are_set_by_id() {
        let manifest = sample();
        let mut choices = Choices::defaults(&manifest);
        assert!(choices.set(&manifest, OPTION_AUTOSTART, true));
        assert!(choices.start_with_windows);
        assert!(choices.set(&manifest, "samples", true));
        assert_eq!(choices.components, ["samples"]);
        assert!(!choices.set(&manifest, OPTION_PATH, true), "path is never offered");
        assert!(!choices.set(&manifest, "nope", true));
    }
}
