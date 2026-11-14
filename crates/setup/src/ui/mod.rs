//! The setup window's UI primitives, carried over from the Bach desktop app
//! and trimmed to what an installer needs. `use crate::ui::prelude::*;`
//! brings in everything a page uses.

// A small component library: not every builder method is used by today's
// pages, and keeping each component's API whole makes new pages cheap.
#![allow(dead_code)]

pub mod appear;
pub mod bach_mark;
pub mod checkbox;
pub mod color;
pub mod folder_dialog;
pub mod form_button;
pub mod icon;
mod icon_paint;
pub mod interaction;
pub mod motion;
pub mod progress_bar;
pub mod row;
pub mod shimmer;
pub mod spinner;
pub mod surface;
pub mod typography;

pub use appear::Appear;
pub use bach_mark::BachMark;
pub use checkbox::Checkbox;
pub use form_button::FormButton;
pub use icon::Icon;
pub use progress_bar::ProgressBar;
pub use row::Row;
pub use shimmer::Shimmer;
pub use typography::TypeScale;

/// Everything a page usually needs, in one import.
pub mod prelude {
    pub use super::{
        Appear, BachMark, Checkbox, FormButton, Icon, ProgressBar, Row, Shimmer, TypeScale,
    };
}

/// Motion preferences. Call once at startup, before opening the window.
pub fn init(cx: &mut gpui::App) {
    motion::init(cx);
}
