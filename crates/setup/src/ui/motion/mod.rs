//! Motion: CSS easings, closed-form springs, a retargetable animated value
//! and the few named motions the setup window uses.

mod easing;
mod os;
mod presets;
mod spring;
mod value;

pub use easing::*;
pub use presets::*;
pub use spring::Spring;
pub use value::{AnimatedValue, Tween};

use gpui::{App, Global, Window};

/// Whether the person asked for less motion. GPUI doesn't expose it, so
/// `init` reads the OS setting once at startup (see os.rs). The shimmer and
/// the Bach mark hold still when it's set.
#[derive(Default)]
pub struct MotionPreferences {
    pub reduced: bool,
}

impl Global for MotionPreferences {}

pub fn reduced_motion(cx: &App) -> bool {
    cx.try_global::<MotionPreferences>()
        .is_some_and(|prefs| prefs.reduced)
}

/// Reads the OS setting into the global.
pub fn init(cx: &mut App) {
    cx.set_global(MotionPreferences {
        reduced: os::reduced_motion_requested(),
    });
}

/// Asks for another frame while anything is still moving. Once everything
/// settles no more frames are requested, so an idle window costs nothing.
pub fn keep_animating(window: &mut Window, animating: bool) {
    if animating {
        window.request_animation_frame();
    }
}
