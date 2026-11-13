//! A thin progress bar: a 4px `--muted` track with a `--primary` fill that
//! glides to each new value on a spring, so steps that report in jumps still
//! read as one smooth move.
//!
//! ```ignore
//! ProgressBar::new("install-progress", 0.4)
//! ```
//!
//! The fill's position lives in keyed element state; give the bar a stable
//! id for as long as it tracks the same work, and a new one to start empty.

use gpui::{
    div, px, relative, App, ElementId, IntoElement, ParentElement, RenderOnce, Styled, Window,
};

use super::motion::{keep_animating, AnimatedValue, Spring};
use crate::theme::ActiveTheme;

/// Soft and unhurried: a jump from 0 to 1 takes about half a second.
const FILL_SPRING: Spring = Spring::new(170.0, 26.0);

#[derive(IntoElement)]
pub struct ProgressBar {
    id: ElementId,
    value: f32,
}

impl ProgressBar {
    /// `value` runs from 0 (empty) to 1 (full); anything outside is clamped.
    pub fn new(id: impl Into<ElementId>, value: f32) -> Self {
        Self {
            id: id.into(),
            value: value.clamp(0.0, 1.0),
        }
    }
}

struct FillState {
    fill: AnimatedValue,
}

impl RenderOnce for ProgressBar {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let target = self.value;
        let state = window.use_keyed_state(self.id, cx, |_, _| FillState {
            fill: AnimatedValue::new(0.0, FILL_SPRING).with_precision(0.001),
        });
        let (fill, moving) = state.update(cx, |state, _| {
            if (state.fill.target() - target).abs() > f32::EPSILON {
                state.fill.set(target);
            }
            (state.fill.value(), state.fill.is_animating())
        });
        keep_animating(window, moving);

        let theme = cx.theme();
        div()
            .relative()
            .w_full()
            .h(px(4.))
            .rounded_full()
            .overflow_hidden()
            .bg(theme.muted)
            .child(
                div()
                    .absolute()
                    .left_0()
                    .top_0()
                    .h_full()
                    .w(relative(fill.clamp(0.0, 1.0)))
                    .rounded_full()
                    .bg(theme.primary),
            )
    }
}
