//! A damped spring, solved in closed form like motion/react's. Being exact
//! at any `t` means frame drops never change where the motion lands, and a
//! retarget can pick up the current position and velocity mid-flight.

use std::time::Duration;

/// Stiffness / damping / mass, as the web's `{ type: "spring", ... }`.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Spring {
    pub stiffness: f32,
    pub damping: f32,
    pub mass: f32,
}

/// Displacement below which a unit move counts as settled (its speed must
/// also be under ten times that, per second).
const REST_DELTA: f32 = 0.001;
/// Springs that never settle (zero damping) are cut off here.
const MAX_SETTLE: f32 = 10.0;

impl Spring {
    /// A spring with mass 1, the web's default.
    pub const fn new(stiffness: f32, damping: f32) -> Self {
        Self {
            stiffness,
            damping,
            mass: 1.0,
        }
    }

    pub const fn with_mass(mut self, mass: f32) -> Self {
        self.mass = mass;
        self
    }

    fn omega(&self) -> f32 {
        (self.stiffness / self.mass.max(1e-6)).sqrt()
    }

    fn zeta(&self) -> f32 {
        self.damping / (2.0 * (self.stiffness * self.mass).max(1e-6).sqrt())
    }

    /// Displacement from the target and velocity after `t` seconds, starting
    /// `x0` away from the target with velocity `v0` (units per second).
    pub fn evaluate(&self, x0: f32, v0: f32, t: f32) -> (f32, f32) {
        let w0 = self.omega();
        let zeta = self.zeta();
        if t <= 0.0 {
            return (x0, v0);
        }
        if zeta < 1.0 {
            // Underdamped: a decaying oscillation.
            let wd = w0 * (1.0 - zeta * zeta).sqrt();
            let decay = (-zeta * w0 * t).exp();
            let b = (v0 + zeta * w0 * x0) / wd;
            let (sin, cos) = (wd * t).sin_cos();
            let x = decay * (x0 * cos + b * sin);
            let v = decay * ((b * wd - zeta * w0 * x0) * cos - (x0 * wd + zeta * w0 * b) * sin);
            (x, v)
        } else if (zeta - 1.0).abs() < 1e-4 {
            // Critically damped.
            let decay = (-w0 * t).exp();
            let b = v0 + w0 * x0;
            let x = decay * (x0 + b * t);
            let v = decay * (b - w0 * (x0 + b * t));
            (x, v)
        } else {
            // Overdamped: two decaying exponentials.
            let root = (zeta * zeta - 1.0).sqrt();
            let r1 = -w0 * (zeta - root);
            let r2 = -w0 * (zeta + root);
            let c2 = (v0 - r1 * x0) / (r2 - r1);
            let c1 = x0 - c2;
            let (e1, e2) = ((r1 * t).exp(), (r2 * t).exp());
            (c1 * e1 + c2 * e2, c1 * r1 * e1 + c2 * r2 * e2)
        }
    }

    /// How long until a move of `distance` (starting at `velocity`) is within
    /// `precision` of its target and slower than `precision * 10` per second.
    pub fn settle_time(&self, distance: f32, velocity: f32, precision: f32) -> Duration {
        let rest_delta = precision.max(1e-6);
        let rest_speed = rest_delta * 10.0;
        if distance.abs() < rest_delta && velocity.abs() < rest_speed {
            return Duration::ZERO;
        }
        // Stepping is fine here: it runs once per retarget, not per frame.
        let step = 1.0 / 240.0;
        let mut t = step;
        while t < MAX_SETTLE {
            let (x, v) = self.evaluate(distance, velocity, t);
            if x.abs() < rest_delta && v.abs() < rest_speed {
                return Duration::from_secs_f32(t);
            }
            t += step;
        }
        Duration::from_secs_f32(MAX_SETTLE)
    }

    /// How long a unit move takes to settle; use it as a `gpui::Animation`
    /// duration together with [`Spring::progress`].
    pub fn duration(&self) -> Duration {
        self.settle_time(1.0, 0.0, REST_DELTA)
    }

    /// The 0→1 progress curve of a unit move, for `gpui::Animation`:
    /// `Animation::new(spring.duration()).with_easing(spring.progress())`.
    ///
    /// GPUI asserts that eased values stay within 0..=1, so overshoot is
    /// clipped here. Springs that bounce (ICON_POP) should be driven with
    /// [`super::AnimatedValue`] instead, which keeps the overshoot.
    pub fn progress(self) -> impl Fn(f32) -> f32 + 'static {
        let total = self.duration().as_secs_f32();
        move |t| {
            let (x, _) = self.evaluate(1.0, 0.0, t * total);
            (1.0 - x).clamp(0.0, 1.0)
        }
    }

    /// Whether this spring overshoots its target on a plain move.
    pub fn bounces(&self) -> bool {
        self.zeta() < 1.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_regime_settles_at_its_target() {
        for spring in [
            Spring::new(380.0, 38.0).with_mass(0.9),
            Spring::new(600.0, 32.0),
            Spring::new(320.0, 32.0),
            Spring::new(100.0, 20.0),
        ] {
            let end = spring.duration().as_secs_f32();
            let (x, v) = spring.evaluate(1.0, 0.0, end);
            assert!(x.abs() < 0.002 && v.abs() < 0.02, "{spring:?}: {x} {v}");
            assert!(end < 2.0, "{spring:?} took {end}s");
        }
    }

    #[test]
    fn velocity_matches_the_slope() {
        let spring = Spring::new(600.0, 32.0);
        let (x1, v) = spring.evaluate(1.0, 3.0, 0.1);
        let (x2, _) = spring.evaluate(1.0, 3.0, 0.1 + 1e-4);
        assert!(((x2 - x1) / 1e-4 - v).abs() < 0.05);
    }
}
