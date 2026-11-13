//! A checkbox: a 16px rounded box with an `--input` edge that firms up on
//! hover, filling with `--primary` and a tick when checked. Fill, edge and
//! tick fade over 150ms. The hit area is a 28px square around the box, so
//! it's easy to land on, and a click never reaches the row underneath.
//!
//! ```ignore
//! Checkbox::new("desktop-shortcut", checked).on_toggle(cx.listener(|this, _, _, cx| …))
//! ```

use std::rc::Rc;

use gpui::{
    div, px, App, ClickEvent, CursorStyle, ElementId, InteractiveElement, IntoElement,
    ParentElement, RenderOnce, StatefulInteractiveElement, Styled, Window,
};

use super::color::{clear, mix};
use super::icon::Icon;
use super::interaction::{use_interaction, Interaction};
use super::motion::{self, AnimatedValue, COLOR_FADE};
use crate::theme::ActiveTheme;

type Handler = Rc<dyn Fn(&ClickEvent, &mut Window, &mut App)>;

#[derive(IntoElement)]
pub struct Checkbox {
    id: ElementId,
    checked: bool,
    on_toggle: Option<Handler>,
}

impl Checkbox {
    pub fn new(id: impl Into<ElementId>, checked: bool) -> Self {
        Self {
            id: id.into(),
            checked,
            on_toggle: None,
        }
    }

    pub fn on_toggle(
        mut self,
        handler: impl Fn(&ClickEvent, &mut Window, &mut App) + 'static,
    ) -> Self {
        self.on_toggle = Some(Rc::new(handler));
        self
    }
}

struct Fill(AnimatedValue);

/// How far the box has faded toward checked, 0..1. Keyed element state, so
/// it lives only while the box keeps rendering.
fn use_fill(id: ElementId, on: bool, window: &mut Window, cx: &mut App) -> f32 {
    let target = if on { 1.0 } else { 0.0 };
    let state = window.use_keyed_state(id, cx, move |_, _| {
        Fill(AnimatedValue::new(target, COLOR_FADE).with_precision(0.001))
    });
    let (value, moving) = state.update(cx, |fill, _| {
        fill.0.set(target);
        (fill.0.value(), fill.0.is_animating())
    });
    motion::keep_animating(window, moving);
    value
}

impl RenderOnce for Checkbox {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let fill_id = ElementId::NamedChild(Box::new(self.id.clone()), "fill".into());
        let fill = use_fill(fill_id, self.checked, window, cx);
        let interaction = use_interaction(self.id.clone(), window, cx);
        let hover = interaction.read(cx).hover();
        let theme = cx.theme();

        let edge = mix(theme.input, theme.ring, hover);
        let square = div()
            .relative()
            .flex()
            .items_center()
            .justify_center()
            .size(px(16.))
            .rounded(px(5.))
            .border(px(1.5))
            .border_color(mix(edge, theme.primary, fill))
            .bg(mix(clear(theme.primary), theme.primary, fill))
            .child(
                Icon::new("check")
                    .size(12.)
                    .stroke(3.)
                    .color(theme.primary_foreground.opacity(fill)),
            );

        let hit = div()
            .id(self.id)
            .flex()
            .flex_none()
            .items_center()
            .justify_center()
            .size(px(28.))
            .rounded(px(8.))
            .cursor(CursorStyle::PointingHand)
            .child(square);
        let hit = match self.on_toggle {
            Some(handler) => hit.on_click(move |event, window, cx| {
                cx.stop_propagation();
                handler(event, window, cx);
            }),
            None => hit,
        };
        Interaction::track(&interaction, hit)
    }
}
