//! Colour tokens for light and dark, from Bach's design system, plus the
//! app's own accent when its `bach.toml` sets one. No view spells a colour
//! literal of its own.

use gpui::{px, rgb, App, Global, Hsla, Pixels, Rgba, WindowAppearance};

use crate::ui::color::mix;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Light,
    Dark,
}

#[derive(Clone, Debug)]
pub struct Theme {
    pub mode: Mode,
    /// The window.
    pub surface: Hsla,
    /// Sunken icon wells.
    pub well: Hsla,
    pub well_outline: Hsla,
    pub well_highlight: Hsla,
    pub border: Hsla,
    pub foreground: Hsla,
    pub muted_foreground: Hsla,
    /// Buttons, the progress fill and ticked boxes.
    pub primary: Hsla,
    pub primary_hover: Hsla,
    pub primary_foreground: Hsla,
    /// The progress track.
    pub muted: Hsla,
    pub destructive: Hsla,
    /// A checkbox's resting edge.
    pub input: Hsla,
    /// A checkbox's hovered edge.
    pub ring: Hsla,
    /// Hover wash on outline buttons.
    pub hover: Hsla,
}

impl Global for Theme {}

/// The one corner radius.
pub const RADIUS: Pixels = px(12.);

/// Inter, embedded as static instances (see assets.rs). If it can't be
/// registered GPUI falls back to Segoe UI on its own.
pub const FONT_SANS: &str = "Inter";
/// Inter at its display optical size, for headings.
pub const FONT_DISPLAY: &str = "Inter Display";

fn hex(value: u32) -> Hsla {
    rgb(value).into()
}

fn alpha(value: u32, a: f32) -> Hsla {
    let mut color = hex(value);
    color.a = a;
    color
}

impl Theme {
    pub fn light() -> Self {
        Self {
            mode: Mode::Light,
            surface: hex(0xffffff),
            well: hex(0xf3f3f3),
            well_outline: alpha(0x000000, 0.05),
            well_highlight: alpha(0xffffff, 0.9),
            border: alpha(0x000000, 0.07),
            foreground: hex(0x141414),
            muted_foreground: hex(0x696969),
            primary: hex(0x0f0f0f),
            primary_hover: hex(0x292929),
            primary_foreground: hex(0xfafafa),
            muted: hex(0xe1e1e1),
            destructive: hex(0xe7000b),
            input: hex(0xdedede),
            ring: hex(0x808080),
            hover: alpha(0x000000, 0.05),
        }
    }

    pub fn dark() -> Self {
        Self {
            mode: Mode::Dark,
            surface: hex(0x181818),
            well: hex(0x242424),
            well_outline: alpha(0xffffff, 0.05),
            well_highlight: alpha(0xffffff, 0.12),
            border: alpha(0xffffff, 0.06),
            foreground: hex(0xf0f0f0),
            muted_foreground: hex(0xa4a4a4),
            primary: hex(0xdedede),
            primary_hover: hex(0xeeeeee),
            primary_foreground: hex(0x0f0f0f),
            muted: hex(0x303030),
            destructive: hex(0xff6467),
            input: alpha(0xffffff, 0.15),
            ring: hex(0x747474),
            hover: alpha(0xffffff, 0.06),
        }
    }

    pub fn for_appearance(appearance: WindowAppearance) -> Self {
        match appearance {
            WindowAppearance::Dark | WindowAppearance::VibrantDark => Self::dark(),
            WindowAppearance::Light | WindowAppearance::VibrantLight => Self::light(),
        }
    }

    /// Paints buttons, the progress bar and ticked boxes in `accent`
    /// (`0xrrggbb`), with ink that stays readable on it.
    pub fn with_accent(mut self, accent: Option<u32>) -> Self {
        let Some(accent) = accent else {
            return self;
        };
        let color = hex(accent);
        let light_ink = relative_luminance(accent) < 0.45;
        let toward = if self.mode == Mode::Dark { hex(0xffffff) } else { hex(0x000000) };
        self.primary = color;
        self.primary_hover = mix(color, toward, 0.12);
        self.primary_foreground = if light_ink { hex(0xffffff) } else { hex(0x0f0f0f) };
        self
    }
}

/// WCAG relative luminance of an sRGB colour, 0 (black) to 1 (white).
fn relative_luminance(value: u32) -> f32 {
    let color: Rgba = rgb(value);
    let linear = |channel: f32| {
        if channel <= 0.04045 {
            channel / 12.92
        } else {
            ((channel + 0.055) / 1.055).powf(2.4)
        }
    };
    0.2126 * linear(color.r) + 0.7152 * linear(color.g) + 0.0722 * linear(color.b)
}

/// `cx.theme()` anywhere an `App` is reachable.
pub trait ActiveTheme {
    fn theme(&self) -> &Theme;
}

impl ActiveTheme for App {
    fn theme(&self) -> &Theme {
        self.global::<Theme>()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accents_get_readable_ink() {
        let blue = Theme::light().with_accent(Some(0x2563eb));
        assert_eq!(blue.primary_foreground, hex(0xffffff));
        let yellow = Theme::dark().with_accent(Some(0xfacc15));
        assert_eq!(yellow.primary_foreground, hex(0x0f0f0f));
        let plain = Theme::light().with_accent(None);
        assert_eq!(plain.primary, Theme::light().primary);
    }
}
