//! Animated hover and press state for controls. GPUI's `.hover()` style
//! snaps; the web fades colours over 150ms (`transition-colors`). Controls
//! call [`use_interaction`] for a per-element state entity, wire it with
//! [`Interaction::track`], and mix their colours by `hover()` / `press()`:
//!
//! ```ignore
//! let state = use_interaction(("pill", id), window, cx);
//! let hover = state.read(cx).hover();
//! let bg = color::mix(theme.primary, theme.primary_hover, hover);
//! Interaction::track(&state, div().id(id).bg(bg)…)
//! ```
//!
//! The state lives only while its element keeps rendering (GPUI keyed
//! element state), so lists of rows never accumulate entries.

use gpui::{App, Context, ElementId, Entity, MouseButton, StatefulInteractiveElement, Window};

use super::motion::{AnimatedValue, Tween, COLOR_FADE};

pub struct Interaction {
    hover: AnimatedValue,
    press: AnimatedValue,
    hovered: bool,
    pressed: bool,
}

impl Interaction {
    fn new(fade: Tween) -> Self {
        Self {
            hover: AnimatedValue::new(0.0, fade).with_precision(0.001),
            press: AnimatedValue::new(0.0, fade).with_precision(0.001),
            hovered: false,
            pressed: false,
        }
    }

    /// Hover progress, 0..1, eased.
    pub fn hover(&self) -> f32 {
        self.hover.value()
    }

    /// Press progress, 0..1, eased.
    pub fn press(&self) -> f32 {
        self.press.value()
    }

    pub fn is_hovered(&self) -> bool {
        self.hovered
    }

    pub fn is_pressed(&self) -> bool {
        self.pressed
    }

    /// Whether a fade is still running; pass to `motion::keep_animating`.
    pub fn is_animating(&self) -> bool {
        self.hover.is_animating() || self.press.is_animating()
    }

    pub fn set_hovered(&mut self, hovered: bool, cx: &mut Context<Self>) {
        if self.hovered == hovered {
            return;
        }
        self.hovered = hovered;
        self.hover.set(if hovered { 1.0 } else { 0.0 });
        if !hovered {
            self.set_pressed(false, cx);
        }
        cx.notify();
    }

    pub fn set_pressed(&mut self, pressed: bool, cx: &mut Context<Self>) {
        if self.pressed == pressed {
            return;
        }
        self.pressed = pressed;
        self.press.set(if pressed { 1.0 } else { 0.0 });
        cx.notify();
    }

    /// Wires hover and press tracking onto an element.
    pub fn track<E: StatefulInteractiveElement>(state: &Entity<Self>, element: E) -> E {
        let hover = state.downgrade();
        let down = state.downgrade();
        let up = state.downgrade();
        let up_out = state.downgrade();
        element
            .on_hover(move |hovered, _, cx| {
                let hovered = *hovered;
                hover.update(cx, |state, cx| state.set_hovered(hovered, cx)).ok();
            })
            .on_mouse_down(MouseButton::Left, move |_, _, cx| {
                down.update(cx, |state, cx| state.set_pressed(true, cx)).ok();
            })
            .on_mouse_up(MouseButton::Left, move |_, _, cx| {
                up.update(cx, |state, cx| state.set_pressed(false, cx)).ok();
            })
            .on_mouse_up_out(MouseButton::Left, move |_, _, cx| {
                up_out.update(cx, |state, cx| state.set_pressed(false, cx)).ok();
            })
    }
}

/// The interaction state for the element keyed `id`, created on first use
/// with the standard 150ms colour fade.
pub fn use_interaction(id: impl Into<ElementId>, window: &mut Window, cx: &mut App) -> Entity<Interaction> {
    use_interaction_with(id, COLOR_FADE, window, cx)
}

/// Like [`use_interaction`], with another fade (picker rows use 100ms).
pub fn use_interaction_with(
    id: impl Into<ElementId>,
    fade: Tween,
    window: &mut Window,
    cx: &mut App,
) -> Entity<Interaction> {
    let state = window.use_keyed_state(id, cx, move |_, _| Interaction::new(fade));
    if state.read(cx).is_animating() {
        window.request_animation_frame();
    }
    state
}
