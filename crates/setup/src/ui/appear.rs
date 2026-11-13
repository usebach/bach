//! Entrances and exits made of an opacity fade and a vertical offset — the
//! web's `initial/animate/exit` blocks (TRIM_FADE, RUN_ENTER, ROW_ENTER,
//! BLOCK_ENTER, TOAST…).
//!
//! ```ignore
//! Appear::new(("row", step.position), ROW_ENTER, row)            // fades in on mount
//! Appear::new("heading", TRIM_FADE, heading).initial(false)      // first paint at rest
//! Appear::new(("row", key), ROW_ENTER, row).exiting(since)       // plays the exit
//! ```
//!
//! The entrance starts the first frame the id is rendered and replays when
//! the element comes back after leaving the tree. Exits need the caller to
//! keep rendering the element while it leaves; `ui::Presence` tracks that
//! for keyed lists. The offset is a paint-time shift (`relative().top()`),
//! so neighbours never move.

use std::time::{Duration, Instant};

use gpui::{
    div, px, AnyElement, App, Div, ElementId, IntoElement, ParentElement, RenderOnce,
    StyleRefinement, Styled, Window,
};

use super::motion::Transition;

#[derive(IntoElement)]
pub struct Appear {
    id: ElementId,
    transition: Transition,
    animate_entrance: bool,
    exiting: Option<Instant>,
    base: Div,
    child: AnyElement,
}

impl Appear {
    pub fn new(id: impl Into<ElementId>, transition: Transition, child: impl IntoElement) -> Self {
        Self {
            id: id.into(),
            transition,
            animate_entrance: true,
            exiting: None,
            base: div(),
            child: child.into_any_element(),
        }
    }

    /// `false` mounts at rest, like `AnimatePresence initial={false}`.
    pub fn initial(mut self, animate: bool) -> Self {
        self.animate_entrance = animate;
        self
    }

    /// Plays the exit, which started at `since`.
    pub fn exiting(mut self, since: Instant) -> Self {
        self.exiting = Some(since);
        self
    }

    /// Plays the exit if `since` is set; otherwise behaves as normal.
    pub fn exiting_since(mut self, since: Option<Instant>) -> Self {
        self.exiting = since;
        self
    }
}

impl Styled for Appear {
    fn style(&mut self) -> &mut StyleRefinement {
        self.base.style()
    }
}

struct AppearState {
    mounted: Instant,
    animate: bool,
}

/// Opacity and y offset of a transition right now, and whether it is still
/// moving (keep requesting frames while it is).
pub fn transition_frame(
    transition: &Transition,
    mounted: Instant,
    animate: bool,
    exiting: Option<Instant>,
) -> ((f32, f32), bool) {
    let length = transition.duration();
    match exiting {
        Some(since) => {
            let elapsed = since.elapsed();
            (transition.exit_at(elapsed), elapsed < length)
        }
        None if animate => {
            let elapsed = mounted.elapsed();
            if elapsed >= length {
                ((1.0, 0.0), false)
            } else {
                (transition.enter_at(elapsed), true)
            }
        }
        None => ((1.0, 0.0), false),
    }
}

impl RenderOnce for Appear {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let animate = self.animate_entrance;
        let state = window.use_keyed_state(self.id, cx, move |_, _| AppearState {
            mounted: Instant::now(),
            animate,
        });
        let (mounted, animate) = {
            let state = state.read(cx);
            (state.mounted, state.animate)
        };
        let ((opacity, y), moving) =
            transition_frame(&self.transition, mounted, animate, self.exiting);
        if moving {
            window.request_animation_frame();
        }
        self.base
            .relative()
            .top(px(y))
            .opacity(opacity.clamp(0.0, 1.0))
            .child(self.child)
    }
}

/// Whether an exit that began at `since` has finished.
pub fn exit_finished(transition: &Transition, since: Instant) -> bool {
    since.elapsed() >= transition.duration()
}

/// How long an exit lasts, for pruning.
pub fn exit_length(transition: &Transition) -> Duration {
    transition.duration()
}
