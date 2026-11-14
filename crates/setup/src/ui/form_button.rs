//! The setup window's buttons (shadcn's `Button size="sm"`): 32px tall,
//! 10px sides, radius 12, 14 / 20 medium. Primary fills with `--primary`;
//! outline sits on a hairline with the hover wash; destructive fills red. Both nudge down a pixel
//! while pressed (`active:translate-y-px`), fade their fill over 150ms, and
//! sit at 50% while disabled. A pending button keeps its label and leads
//! with a spinner, so its width never jumps.
//!
//! ```ignore
//! FormButton::new("save-key", "Save").disabled(!can_save).on_click(|_, window, cx| …)
//! FormButton::new("disconnect", "Disconnect").outline().pending(busy)
//! ```

use std::rc::Rc;

use gpui::{
    div, px, App, ClickEvent, CursorStyle, ElementId, FontWeight, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, SharedString, StatefulInteractiveElement, Styled, Window,
};

use super::color::{clear, mix};
use super::interaction::{use_interaction, Interaction};
use super::spinner::Spinner;
use crate::theme::{ActiveTheme, RADIUS};

type ClickHandler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Variant {
    Primary,
    Outline,
    Destructive,
}

#[derive(IntoElement)]
pub struct FormButton {
    id: ElementId,
    label: SharedString,
    variant: Variant,
    disabled: bool,
    pending: bool,
    focused: bool,
    on_click: Option<ClickHandler>,
}

impl FormButton {
    pub fn new(id: impl Into<ElementId>, label: impl Into<SharedString>) -> Self {
        Self {
            id: id.into(),
            label: label.into(),
            variant: Variant::Primary,
            disabled: false,
            pending: false,
            focused: false,
            on_click: None,
        }
    }

    /// The quiet one: a hairline border and the hover wash.
    pub fn outline(mut self) -> Self {
        self.variant = Variant::Outline;
        self
    }

    /// For the irreversible answer: `--destructive` fill, white ink.
    pub fn destructive(mut self) -> Self {
        self.variant = Variant::Destructive;
        self
    }

    pub fn disabled(mut self, disabled: bool) -> Self {
        self.disabled = disabled;
        self
    }

    /// Waiting on the work it started: a spinner, and no more clicks.
    pub fn pending(mut self, pending: bool) -> Self {
        self.pending = pending;
        self
    }

    /// Draws the keyboard focus ring around it.
    pub fn focused(mut self, focused: bool) -> Self {
        self.focused = focused;
        self
    }

    pub fn on_click(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_click = Some(Rc::new(handler));
        self
    }
}

impl RenderOnce for FormButton {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let inert = self.disabled || self.pending;
        let state = use_interaction(self.id.clone(), window, cx);
        let (hover, press) = if inert {
            (0.0, 0.0)
        } else {
            let state = state.read(cx);
            (state.hover(), state.press())
        };
        let theme = cx.theme();
        let (bg, fg, border) = match self.variant {
            Variant::Primary => (
                mix(theme.primary, theme.primary_hover, hover),
                theme.primary_foreground,
                None,
            ),
            Variant::Destructive => (
                mix(theme.destructive, theme.destructive.opacity(0.88), hover),
                gpui::white(),
                None,
            ),
            Variant::Outline => (
                mix(clear(theme.hover), theme.hover, hover),
                theme.foreground,
                Some(theme.border),
            ),
        };

        let mut button = div()
            .id(self.id)
            .relative()
            .top(px(press))
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .gap(px(6.))
            .h(px(32.))
            .px(px(10.))
            .rounded(RADIUS)
            .bg(bg)
            .text_color(fg)
            .text_size(px(14.))
            .line_height(px(20.))
            .font_weight(FontWeight::MEDIUM)
            .whitespace_nowrap()
            .children(self.pending.then(|| Spinner::new().size(14.).color(fg)))
            .child(self.label)
            // A ring 2px outside the button, like a browser's focus outline.
            .children(self.focused.then(|| {
                div()
                    .absolute()
                    .top(px(-4.))
                    .left(px(-4.))
                    .right(px(-4.))
                    .bottom(px(-4.))
                    .rounded(RADIUS + px(4.))
                    .border_2()
                    .border_color(theme.ring)
            }));
        if let Some(border) = border {
            button = button.border_1().border_color(border);
        }
        if self.disabled {
            return button.opacity(0.5).into_any_element();
        }
        if self.pending {
            return button.opacity(0.7).into_any_element();
        }
        let button = button.cursor(CursorStyle::PointingHand);
        let button = match self.on_click {
            // A button inside a clickable row handles its click alone.
            Some(handler) => button.on_click(move |event, window, cx| {
                cx.stop_propagation();
                handler(event, window, cx)
            }),
            None => button,
        };
        Interaction::track(&state, button).into_any_element()
    }
}
