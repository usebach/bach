//! Optional components, link schemes and file types.

use serde::{Deserialize, Serialize};

use crate::i18n::Localized;

/// An optional group of files, offered as a box on the install page.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Component {
    /// Lowercase letters, digits, `-` and `_`; also its `--with` name.
    pub id: String,
    pub title: Localized,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<Localized>,
    /// Whether the box starts ticked on a first install.
    pub default: bool,
}

/// A link scheme like `hello://` that starts the app with the link as its
/// argument. Registered for the user only (`HKCU\Software\Classes`).
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Protocol {
    /// Lowercase, without `://`.
    pub scheme: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// A file extension the app offers to open, listed under "Open with". It
/// never takes over a type another app already opens by default.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FileType {
    /// Lowercase, with its dot: `.hello`.
    pub extension: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

impl FileType {
    /// The ProgID Bach registers for this type: `<app id>.<extension>`,
    /// with spaces removed, so it's plainly the app's own key.
    pub fn prog_id(&self, app_id: &str) -> String {
        let app: String = app_id.chars().filter(|c| !c.is_whitespace()).collect();
        format!("{app}{}", self.extension)
    }
}

/// Schemes Windows or browsers own; registering them would hijack links.
const RESERVED_SCHEMES: &[&str] = &[
    "http", "https", "file", "ftp", "mailto", "news", "nntp", "telnet", "ldap", "tel", "sms",
    "javascript", "vbscript", "data", "about", "blob", "res", "shell", "search", "search-ms",
    "microsoft-edge", "callto", "skype", "steam", "zoommtg", "ws", "wss",
];

/// Whether `scheme` can be registered: 2 to 32 characters, a lowercase
/// letter first, then letters, digits, `+`, `-` or `.`, and not one of the
/// schemes Windows and browsers own.
pub fn is_valid_scheme(scheme: &str) -> bool {
    let mut chars = scheme.chars();
    let first_ok = chars.next().is_some_and(|c| c.is_ascii_lowercase());
    (2..=32).contains(&scheme.len())
        && first_ok
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '+' | '-' | '.'))
        && !RESERVED_SCHEMES.contains(&scheme)
        && !scheme.starts_with("ms-")
}

/// Types that run code or that Windows itself handles; an app may not
/// claim them.
const RESERVED_EXTENSIONS: &[&str] = &[
    "exe", "com", "bat", "cmd", "ps1", "psm1", "lnk", "msi", "msp", "scr", "vbs", "vbe", "js", "jse",
    "wsf", "wsh", "dll", "sys", "reg", "url", "pif", "cpl", "hta", "msc", "inf", "appx", "msix",
];

/// Whether `extension` (with its dot, lowercase) can be registered.
pub fn is_valid_extension(extension: &str) -> bool {
    let Some(name) = extension.strip_prefix('.') else {
        return false;
    };
    (1..=16).contains(&name.len())
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_'))
        && !RESERVED_EXTENSIONS.contains(&name)
}

/// Whether `id` can name a component: 1 to 32 lowercase letters, digits,
/// `-` or `_`, starting with a letter or digit.
pub fn is_valid_component_id(id: &str) -> bool {
    let mut chars = id.chars();
    chars.next().is_some_and(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
        && id.len() <= 32
        && chars.all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '-' | '_'))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schemes() {
        assert!(is_valid_scheme("hello"));
        assert!(is_valid_scheme("my-app+v2"));
        for bad in ["http", "https", "ms-settings", "Hello", "h", "1app", "a b", "file"] {
            assert!(!is_valid_scheme(bad), "{bad}");
        }
    }

    #[test]
    fn extensions() {
        assert!(is_valid_extension(".hello"));
        assert!(is_valid_extension(".my-doc"));
        for bad in [".exe", "hello", ".", ".a.b", ".BAT", ".lnk"] {
            assert!(!is_valid_extension(bad), "{bad}");
        }
    }

    #[test]
    fn prog_ids_belong_to_the_app() {
        let file_type = FileType { extension: ".hello".into(), description: None };
        assert_eq!(file_type.prog_id("My App"), "MyApp.hello");
    }

    #[test]
    fn component_ids() {
        assert!(is_valid_component_id("samples"));
        assert!(is_valid_component_id("lang-de_2"));
        assert!(!is_valid_component_id("Samples"));
        assert!(!is_valid_component_id("-x"));
        assert!(!is_valid_component_id(""));
    }
}
