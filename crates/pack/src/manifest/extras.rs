//! Requirements, extra shortcuts and the update feed.

use serde::{Deserialize, Serialize};

/// Something the app needs on the computer before it can run. Setup only
/// looks (read-only, no administrator rights needed) and, when it's
/// missing, sends the user to `url` to get it.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Requirement {
    #[serde(flatten)]
    pub check: Check,
    /// What people call it: "Microsoft Edge WebView2".
    pub name: String,
    /// Where to get it (https).
    pub url: String,
}

/// How setup tells whether a requirement is there.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum Check {
    /// The Edge WebView2 runtime (Tauri, WebView2 apps).
    Webview2,
    /// A .NET runtime of this major.minor version, like "8.0". `desktop`
    /// asks for the Windows Desktop runtime (WinForms, WPF).
    Dotnet { version: String, desktop: bool },
    /// The Visual C++ 2015-2022 redistributable (x64).
    Vcredist,
    /// A file, like `%SystemRoot%\System32\foo.dll`.
    File { path: String },
    /// A registry key, like `HKLM\SOFTWARE\Vendor\Thing`.
    Registry { key: String },
}

impl Check {
    /// The name and download page Bach uses when the config gives none.
    pub fn defaults(&self) -> Option<(&'static str, &'static str)> {
        match self {
            Self::Webview2 => Some(("Microsoft Edge WebView2", "https://go.microsoft.com/fwlink/p/?LinkId=2124703")),
            Self::Dotnet { desktop: true, .. } => Some((".NET Desktop Runtime", "https://dotnet.microsoft.com/download/dotnet")),
            Self::Dotnet { desktop: false, .. } => Some((".NET Runtime", "https://dotnet.microsoft.com/download/dotnet")),
            Self::Vcredist => {
                Some(("Microsoft Visual C++ Redistributable", "https://aka.ms/vs/17/release/vc_redist.x64.exe"))
            }
            Self::File { .. } | Self::Registry { .. } => None,
        }
    }
}

/// A Start menu shortcut besides the app's own, to another program in the
/// install folder or to the app with arguments.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Shortcut {
    /// The shortcut's name, as the Start menu shows it.
    pub name: String,
    /// The program, relative to the install folder.
    pub target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arguments: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
}

/// Where the installed app looks for a newer setup file.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Updates {
    /// An https URL serving the JSON `bach build` writes next to the setup
    /// file: `{ "version", "url", "sha256", "notes" }`.
    pub feed: String,
}

/// The JSON a feed serves.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct FeedEntry {
    pub version: String,
    /// The setup file to download (https).
    pub url: String,
    /// Its SHA-256, lowercase hex; a download that doesn't match is refused.
    pub sha256: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Whether `url` is an https URL (the only kind updates and requirement
/// links may use).
pub fn is_https(url: &str) -> bool {
    url.strip_prefix("https://").is_some_and(|rest| !rest.is_empty() && !rest.contains(char::is_whitespace))
}

/// Whether `name` can name a `.lnk` file.
pub fn is_valid_shortcut_name(name: &str) -> bool {
    let name = name.trim();
    !name.is_empty()
        && name.len() <= 100
        && !name.chars().any(|c| r#"<>:"/\|?*"#.contains(c) || c.is_control())
        && !name.ends_with('.')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn requirements_read_their_kind() {
        let json = r#"{"kind":"dotnet","version":"8.0","desktop":true,"name":".NET","url":"https://x"}"#;
        let requirement: Requirement = serde_json::from_str(json).unwrap();
        assert_eq!(requirement.check, Check::Dotnet { version: "8.0".into(), desktop: true });
        assert_eq!(serde_json::to_string(&requirement).unwrap(), json);
    }

    #[test]
    fn links_and_names() {
        assert!(is_https("https://example.com/feed.json"));
        assert!(!is_https("http://example.com"));
        assert!(!is_https("https://"));
        assert!(is_valid_shortcut_name("Hello Docs"));
        assert!(!is_valid_shortcut_name("a/b"));
        assert!(!is_valid_shortcut_name("  "));
    }
}
