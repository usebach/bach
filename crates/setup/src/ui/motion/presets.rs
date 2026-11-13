//! The named motions. Views use these and never invent their own timings.
//!
//! GPUI has no blur filter and no transforms on non-SVG elements, so
//! opacity and offsets carry every transition.

use std::time::Duration;

use super::easing::{EASE_OUT, EASE_OUT_CSS};
use super::spring::Spring;
use super::value::{Motion, Tween};

/// The short rise of a page entering.
pub const RISE_SPRING: Spring = Spring::new(520.0, 38.0);

/// An entrance or exit made of an opacity fade and a vertical offset.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Transition {
    /// Opacity in and out.
    pub opacity: Tween,
    /// Where the element starts, px below (positive) or above its place.
    pub enter_y: f32,
    /// Where it ends up when leaving.
    pub exit_y: f32,
    /// How y moves.
    pub y: Motion,
}

impl Transition {
    /// The opacity and y offset `elapsed` into the entrance.
    pub fn enter_at(&self, elapsed: Duration) -> (f32, f32) {
        let opacity = self.opacity.progress(elapsed);
        let y = self.enter_y * (1.0 - motion_progress(self.y, elapsed));
        (opacity, y)
    }

    /// The opacity and y offset `elapsed` into the exit.
    pub fn exit_at(&self, elapsed: Duration) -> (f32, f32) {
        let opacity = 1.0 - self.opacity.progress(elapsed);
        let y = self.exit_y * motion_progress(self.y, elapsed);
        (opacity, y)
    }

    /// How long until the entrance (or exit) has fully landed.
    pub fn duration(&self) -> Duration {
        let y = match self.y {
            Motion::Spring(spring) => spring.duration(),
            Motion::Tween(tween) => tween.duration,
            Motion::Instant => Duration::ZERO,
        };
        self.opacity.duration.max(y)
    }
}

/// Unit progress of a motion, keeping spring overshoot.
pub fn motion_progress(motion: Motion, elapsed: Duration) -> f32 {
    match motion {
        Motion::Instant => 1.0,
        Motion::Tween(tween) => tween.progress(elapsed),
        Motion::Spring(spring) => {
            if elapsed >= spring.duration() {
                1.0
            } else {
                1.0 - spring.evaluate(1.0, 0.0, elapsed.as_secs_f32()).0
            }
        }
    }
}

/// A page rising into place as it replaces the last one.
pub const PAGE_ENTER: Transition = Transition {
    opacity: Tween::new(220, EASE_OUT),
    enter_y: 6.0,
    exit_y: 6.0,
    y: Motion::Spring(RISE_SPRING),
};

/// One sweep of the shimmering status line.
pub const SHIMMER: Duration = Duration::from_millis(720);
/// One turn of the spinner.
pub const SPIN: Duration = Duration::from_secs(1);
/// Hover and press colour fades.
pub const COLOR_FADE: Tween = Tween::new(150, EASE_OUT_CSS);
