//! The choices card: a bordered card that stacks rows on hairline dividers,
//! and a row with an icon in a well, a title over a muted description, and
//! a control pinned right.
//!
//! ```ignore
//! card([Row::new("Desktop shortcut").icon("device-desktop-filled").control(checkbox).into_any_element()], cx)
//! ```

use gpui::{
    div, px, AnyElement, App, FontWeight, IntoElement, ParentElement, RenderOnce, SharedString,
    Styled, Window,
};

use super::icon::Icon;
use super::surface::Surfaces;
use crate::theme::{ActiveTheme, RADIUS};

/// A bordered card; rows divide themselves with hairlines.
pub fn card(rows: impl IntoIterator<Item = AnyElement>, cx: &App) -> impl IntoElement {
    div()
        .rounded(RADIUS)
        .border_1()
        .border_color(cx.theme().border)
        .children(rows)
}

#[derive(IntoElement)]
pub struct Row {
    title: SharedString,
    description: Option<SharedString>,
    icon: Option<&'static str>,
    control: Option<AnyElement>,
    divided: bool,
}

impl Row {
    pub fn new(title: impl Into<SharedString>) -> Self {
        Self {
            title: title.into(),
            description: None,
            icon: None,
            control: None,
            divided: false,
        }
    }

    pub fn description(mut self, text: impl Into<SharedString>) -> Self {
        self.description = Some(text.into());
        self
    }

    /// An 18px icon in a 36px well.
    pub fn icon(mut self, name: &'static str) -> Self {
        self.icon = Some(name);
        self
    }

    /// Pinned right, beside the copy.
    pub fn control(mut self, element: impl IntoElement) -> Self {
        self.control = Some(element.into_any_element());
        self
    }

    /// Draws the hairline above: every row but a card's first.
    pub fn divided(mut self, divided: bool) -> Self {
        self.divided = divided;
        self
    }
}

impl RenderOnce for Row {
    fn render(self, _window: &mut Window, cx: &mut App) -> impl IntoElement {
        let theme = cx.theme();
        let icon = self.icon.map(|name| {
            div()
                .relative()
                .flex()
                .flex_none()
                .items_center()
                .justify_center()
                .size(px(36.))
                .well(theme, RADIUS)
                .text_color(theme.foreground)
                .child(div().relative().flex().child(Icon::new(name).size(18.)))
        });
        let copy = div()
            .flex_1()
            .min_w_0()
            .child(
                div()
                    .text_size(px(14.))
                    .line_height(px(20.))
                    .font_weight(FontWeight::MEDIUM)
                    .child(self.title),
            )
            .children(self.description.map(|text| {
                div()
                    .mt(px(2.))
                    .text_size(px(13.))
                    .line_height(px(18.))
                    .text_color(theme.muted_foreground)
                    .child(text)
            }));
        let mut row = div()
            .p(px(16.))
            .flex()
            .items_center()
            .gap(px(12.))
            .children(icon)
            .child(copy)
            .children(self.control.map(|control| div().flex_none().child(control)));
        if self.divided {
            row = row.border_t_1().border_color(theme.border);
        }
        row
    }
}
