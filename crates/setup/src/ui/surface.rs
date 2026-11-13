//! Sunken wells, the one surface the setup window uses.
//!
//! GPUI has no inset box-shadows, so the well's lit top edge and outline are
//! painted as absolutely positioned overlay children with rounded borders.
//! Overlays take no layout space (like an inset shadow), and they are added
//! first, so they sit above the fill and below the content. Call these
//! before adding children:
//!
//! ```ignore
//! div().well(theme, px(12.)).child(…)
//! ```

use gpui::{div, px, Div, Hsla, ParentElement, Pixels, Styled};

use crate::theme::Theme;

/// An inset-style stroke drawn as a rounded overlay.
fn edge(radius: Pixels, color: Hsla, top_only: bool) -> Div {
    let overlay = div()
        .absolute()
        .top_0()
        .left_0()
        .size_full()
        .rounded(radius)
        .border_color(color);
    if top_only {
        overlay.border_t(px(1.))
    } else {
        overlay.border(px(1.))
    }
}

pub trait Surfaces: Styled + ParentElement + Sized {
    /// A sunken well: `--well` fill, a 1px outline all round and a top edge
    /// that catches light.
    fn well(self, theme: &Theme, radius: Pixels) -> Self {
        self.bg(theme.well)
            .rounded(radius)
            .child(edge(radius, theme.well_highlight, true))
            .child(edge(radius, theme.well_outline, false))
    }
}

impl Surfaces for Div {}
