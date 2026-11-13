//! The loader: `IconLoader2` turning once a second (SPIN).
//!
//! ```ignore
//! Spinner::new().size(20.).color(theme.muted_foreground)
//! ```
//!
//! The rotation comes from the wall clock, not from when the spinner
//! mounted, so every spinner on screen turns in step and a re-keyed one
//! never restarts from the top.

use std::time::{SystemTime, UNIX_EPOCH};

use gpui::{div, px, App, Hsla, IntoElement, ParentElement, RenderOnce, Styled, Window};

use super::icon::Icon;
use super::motion::SPIN;

#[derive(IntoElement)]
pub struct Spinner {
    size: f32,
    stroke: Option<f32>,
    color: Option<Hsla>,
}

impl Spinner {
    pub fn new() -> Self {
        Self {
            size: 16.0,
            stroke: None,
            color: None,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    pub fn stroke(mut self, stroke: f32) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }
}

impl Default for Spinner {
    fn default() -> Self {
        Self::new()
    }
}

/// The spin angle right now, shared by every loader.
pub fn spin_degrees() -> f32 {
    let period = SPIN.as_millis().max(1);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis())
        .unwrap_or(0);
    (now % period) as f32 / period as f32 * 360.0
}

impl RenderOnce for Spinner {
    fn render(self, window: &mut Window, _cx: &mut App) -> impl IntoElement {
        let mut icon = Icon::new("loader-2").size(self.size).rotate(spin_degrees());
        if let Some(stroke) = self.stroke {
            icon = icon.stroke(stroke);
        }
        if let Some(color) = self.color {
            icon = icon.color(color);
        }
        // Spinning never settles; the frame request stops once the spinner
        // is no longer rendered.
        window.request_animation_frame();
        div()
            .flex_none()
            .size(px(self.size))
            .flex()
            .items_center()
            .justify_center()
            .child(icon)
    }
}
