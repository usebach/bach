//! The two text styles the setup window uses beyond plain 14px copy.
//!
//! GPUI has no letter-spacing, so the heading's tight tracking comes from
//! Inter's display cut instead.

use gpui::{px, FontWeight, Styled};

use crate::theme::FONT_DISPLAY;

pub trait TypeScale: Styled + Sized {
    fn type_style(self, size: f32, line_height: f32, weight: FontWeight) -> Self {
        self.text_size(px(size))
            .line_height(px(line_height))
            .font_weight(weight)
    }

    /// Page headings: 22 / 32 medium, display cut.
    fn text_heading(self) -> Self {
        self.font_family(FONT_DISPLAY)
            .type_style(22.0, 32.0, FontWeight::MEDIUM)
    }

    /// The footer's version line: 12 / 16.
    fn text_meta(self) -> Self {
        self.type_style(12.0, 16.0, FontWeight::NORMAL)
    }
}

impl<T: Styled + Sized> TypeScale for T {}
