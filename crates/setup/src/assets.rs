//! Icons and fonts compiled into the runtime, plus the app's own SVG logo
//! read from the payload, all served to GPUI from memory.

use std::borrow::Cow;
use std::sync::Arc;

use anyhow::Result;
use gpui::{App, AssetSource, SharedString};
use rust_embed::RustEmbed;

use crate::theme::{FONT_DISPLAY, FONT_SANS};

#[derive(RustEmbed)]
#[folder = "assets"]
struct Embedded;

/// The asset path the app's SVG logo is served under.
pub const LOGO_PATH: &str = "payload/logo.svg";

/// The runtime's assets, and the logo when the payload has an SVG one.
pub struct Assets {
    pub logo_svg: Option<Arc<[u8]>>,
}

impl AssetSource for Assets {
    fn load(&self, path: &str) -> Result<Option<Cow<'static, [u8]>>> {
        if path == LOGO_PATH {
            return Ok(self.logo_svg.as_ref().map(|bytes| Cow::Owned(bytes.to_vec())));
        }
        Ok(Embedded::get(path).map(|file| file.data))
    }

    fn list(&self, path: &str) -> Result<Vec<SharedString>> {
        Ok(Embedded::iter()
            .filter(|name| name.starts_with(path))
            .map(SharedString::from)
            .collect())
    }
}

/// The SVG path for an exported Tabler icon, e.g. `icon("folder-filled")`.
pub fn icon(name: &str) -> SharedString {
    format!("icons/{name}.svg").into()
}

/// Inter as static TTF instances (400/500/600 at the text optical size,
/// plus a 500 display cut for headings). DirectWrite's in-memory loader
/// can't read WOFF2, so they ship as plain TTF.
const FONT_FILES: [&str; 4] = [
    "fonts/Inter-Regular.ttf",
    "fonts/Inter-Medium.ttf",
    "fonts/Inter-SemiBold.ttf",
    "fonts/InterDisplay-Medium.ttf",
];

/// Registers the embedded fonts. On failure GPUI's own fallback (Segoe UI)
/// renders everything, so this only explains why.
pub fn load_fonts(cx: &mut App) -> Result<(), String> {
    let fonts: Vec<Cow<'static, [u8]>> = FONT_FILES
        .iter()
        .filter_map(|path| Embedded::get(path).map(|file| file.data))
        .collect();
    if fonts.len() != FONT_FILES.len() {
        return Err("some font files are missing from the build".into());
    }
    cx.text_system().add_fonts(fonts).map_err(|err| format!("{err:#}"))?;
    let names = cx.text_system().all_font_names();
    let has = |family: &str| names.iter().any(|name| name == family);
    if has(FONT_SANS) && has(FONT_DISPLAY) {
        Ok(())
    } else {
        Err("the fonts loaded but their families didn't register".into())
    }
}
