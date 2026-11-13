//! An animated number that can be retargeted mid-flight, for anything GPUI's
//! fire-and-forget `Animation` can't do: layout glides, height reveals,
//! hover fades. It is plain data — keep it in a view or keyed element state,
//! read `value()` while rendering, and ask for another frame while
//! `is_animating()`:
//!
//! ```ignore
//! let y = self.offset.value();
//! if self.offset.is_animating() {
//!     window.request_animation_frame();
//! }
//! ```
//!
//! No timers, no threads: time is read from `Instant::now()`, so a value
//! nobody renders costs nothing.

use std::time::{Duration, Instant};

use super::easing::Easing;
use super::spring::Spring;

/// A fixed-length eased move, the web's `{ duration, ease }`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Tween {
    pub duration: Duration,
    pub easing: Easing,
}

impl Tween {
    pub const fn new(millis: u64, easing: Easing) -> Self {
        Self {
            duration: Duration::from_millis(millis),
            easing,
        }
    }

    /// As a `gpui::Animation` (for the simple, never-retargeted cases).
    pub fn animation(&self) -> gpui::Animation {
        gpui::Animation::new(self.duration).with_easing(self.easing.easing())
    }

    /// Eased progress `elapsed` into the tween, 0..=1.
    pub fn progress(&self, elapsed: Duration) -> f32 {
        if self.duration.is_zero() {
            return 1.0;
        }
        // Past the end is the end, whatever the easing does with x > 1.
        let t = (elapsed.as_secs_f32() / self.duration.as_secs_f32()).min(1.0);
        self.easing.apply(t)
    }
}

/// How an [`AnimatedValue`] moves toward its target.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Motion {
    Spring(Spring),
    Tween(Tween),
    /// Snap straight to the target.
    Instant,
}

impl From<Spring> for Motion {
    fn from(spring: Spring) -> Self {
        Motion::Spring(spring)
    }
}

impl From<Tween> for Motion {
    fn from(tween: Tween) -> Self {
        Motion::Tween(tween)
    }
}

#[derive(Clone, Debug)]
pub struct AnimatedValue {
    /// How the next move animates.
    motion: Motion,
    /// How the current move animates.
    active: Motion,
    target: f32,
    /// Where the current move started (absolute value).
    from: f32,
    /// Velocity at the start of the current move, units per second.
    velocity: f32,
    start: Instant,
    /// How long the current move lasts; zero means at rest.
    length: Duration,
    /// Distance at which a spring counts as settled. Defaults to a hundredth
    /// of a unit, which suits pixels; use 0.001 for 0..1 progress values.
    precision: f32,
}

impl AnimatedValue {
    /// A value at rest at `value`.
    pub fn new(value: f32, motion: impl Into<Motion>) -> Self {
        let motion = motion.into();
        Self {
            motion,
            active: motion,
            target: value,
            from: value,
            velocity: 0.0,
            start: Instant::now(),
            length: Duration::ZERO,
            precision: 0.01,
        }
    }

    /// Settling precision, in the value's own units.
    pub fn with_precision(mut self, precision: f32) -> Self {
        self.precision = precision.max(1e-6);
        self
    }

    pub fn motion(&self) -> Motion {
        self.motion
    }

    /// Changes how future moves animate; the current move keeps its motion.
    pub fn set_motion(&mut self, motion: impl Into<Motion>) {
        self.motion = motion.into();
    }

    pub fn target(&self) -> f32 {
        self.target
    }

    /// The current value.
    pub fn value(&self) -> f32 {
        self.value_at(Instant::now())
    }

    pub fn value_at(&self, now: Instant) -> f32 {
        self.sample(now).0
    }

    /// The current velocity, units per second.
    pub fn velocity(&self) -> f32 {
        self.sample(Instant::now()).1
    }

    pub fn is_animating(&self) -> bool {
        self.is_animating_at(Instant::now())
    }

    pub fn is_animating_at(&self, now: Instant) -> bool {
        !self.length.is_zero() && now.duration_since(self.start) < self.length
    }

    /// Animates to `target` from wherever the value is right now, keeping
    /// its velocity — so retargeting mid-flight never jumps or stalls.
    /// Setting the target it already has is a no-op.
    pub fn set(&mut self, target: f32) {
        self.set_with(target, self.motion);
    }

    /// Like [`Self::set`], with a one-off motion for this move.
    pub fn set_with(&mut self, target: f32, motion: impl Into<Motion>) {
        if target == self.target {
            return;
        }
        let now = Instant::now();
        let (value, velocity) = self.sample(now);
        self.begin(now, value, velocity, target, motion.into());
    }

    /// Jumps to `value` with no animation.
    pub fn jump(&mut self, value: f32) {
        self.target = value;
        self.from = value;
        self.velocity = 0.0;
        self.length = Duration::ZERO;
    }

    /// Moves the current value by `delta` without touching velocity or
    /// target: the FLIP step for layout glides. When an element's layout
    /// jumps by `-delta`, shifting its visual offset by `delta` keeps it on
    /// screen exactly where it was, then the spring carries it home.
    pub fn shift(&mut self, delta: f32) {
        if delta == 0.0 {
            return;
        }
        let now = Instant::now();
        let (value, velocity) = self.sample(now);
        let motion = self.motion;
        self.begin(now, value + delta, velocity, self.target, motion);
    }

    fn begin(&mut self, now: Instant, value: f32, velocity: f32, target: f32, motion: Motion) {
        self.from = value;
        self.velocity = velocity;
        self.target = target;
        self.start = now;
        self.length = match motion {
            Motion::Instant => Duration::ZERO,
            Motion::Tween(tween) => tween.duration,
            Motion::Spring(spring) => {
                spring.settle_time(value - target, velocity, self.precision)
            }
        };
        // The move keeps the motion it started with even if set_motion runs.
        self.active = motion;
    }

    fn sample(&self, now: Instant) -> (f32, f32) {
        if !self.is_animating_at(now) {
            return (self.target, 0.0);
        }
        let elapsed = now.duration_since(self.start);
        match self.active {
            Motion::Instant => (self.target, 0.0),
            Motion::Tween(tween) => {
                let t = elapsed.as_secs_f32();
                let span = self.target - self.from;
                let value = self.from + span * tween.progress(elapsed);
                // Finite difference: tweens only need velocity when a spring
                // takes over from them.
                let h = 1.0 / 240.0;
                let ahead = self.from
                    + span * tween.progress(Duration::from_secs_f32(t + h));
                (value, (ahead - value) / h)
            }
            Motion::Spring(spring) => {
                let (x, v) =
                    spring.evaluate(self.from - self.target, self.velocity, elapsed.as_secs_f32());
                (self.target + x, v)
            }
        }
    }
}
