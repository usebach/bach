//! CSS timing functions. `CubicBezier` solves the curve the way browsers do
//! (Newton's method with a bisection fallback), so a 160ms EASE_OUT here
//! lands on the same frames as the web's.

/// A CSS `cubic-bezier(x1, y1, x2, y2)` timing function.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct CubicBezier {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

impl CubicBezier {
    pub const fn new(x1: f32, y1: f32, x2: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }

    /// Progress at time `t` (0..=1). The result can leave 0..=1 for curves
    /// whose y control points do; none of the spec's curves do.
    pub fn ease(&self, t: f32) -> f32 {
        if t <= 0.0 {
            return 0.0;
        }
        if t >= 1.0 {
            return 1.0;
        }
        let s = self.solve_x(t);
        sample(self.y1, self.y2, s)
    }

    /// The curve as a closure, for `gpui::Animation::with_easing`.
    pub fn easing(self) -> impl Fn(f32) -> f32 + 'static {
        move |t| self.ease(t).clamp(0.0, 1.0)
    }

    fn solve_x(&self, x: f32) -> f32 {
        // Newton first: quick and exact for well-behaved curves.
        let mut s = x;
        for _ in 0..8 {
            let error = sample(self.x1, self.x2, s) - x;
            if error.abs() < 1e-6 {
                return s;
            }
            let slope = sample_derivative(self.x1, self.x2, s);
            if slope.abs() < 1e-6 {
                break;
            }
            s -= error / slope;
        }
        // Bisection: always converges, x(s) is monotonic for valid curves.
        let (mut lo, mut hi) = (0.0_f32, 1.0_f32);
        s = x;
        for _ in 0..32 {
            let value = sample(self.x1, self.x2, s);
            if (value - x).abs() < 1e-6 {
                break;
            }
            if value < x {
                lo = s;
            } else {
                hi = s;
            }
            s = (lo + hi) * 0.5;
        }
        s
    }
}

/// One axis of the bezier with end points pinned at 0 and 1.
fn sample(p1: f32, p2: f32, s: f32) -> f32 {
    let u = 1.0 - s;
    3.0 * u * u * s * p1 + 3.0 * u * s * s * p2 + s * s * s
}

fn sample_derivative(p1: f32, p2: f32, s: f32) -> f32 {
    let u = 1.0 - s;
    3.0 * u * u * p1 + 6.0 * u * s * (p2 - p1) + 3.0 * s * s * (1.0 - p2)
}

/// A timing function: linear or a cubic bezier.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Easing {
    Linear,
    Bezier(CubicBezier),
}

impl Easing {
    pub fn apply(&self, t: f32) -> f32 {
        match self {
            Easing::Linear => t.clamp(0.0, 1.0),
            Easing::Bezier(curve) => curve.ease(t),
        }
    }

    /// The curve as a closure, for `gpui::Animation::with_easing`.
    pub fn easing(self) -> impl Fn(f32) -> f32 + 'static {
        move |t| self.apply(t).clamp(0.0, 1.0)
    }
}

/// The app's one ease: `cubic-bezier(0.22, 0.61, 0.36, 1)`.
pub const EASE_OUT: Easing = Easing::Bezier(CubicBezier::new(0.22, 0.61, 0.36, 1.0));
/// CSS's `ease-out`.
pub const EASE_OUT_CSS: Easing = Easing::Bezier(CubicBezier::new(0.0, 0.0, 0.58, 1.0));
/// The Bach mark's petal flip: `cubic-bezier(0.55, 0, 0.3, 1)`.
pub const PETAL_FLIP_EASE: Easing = Easing::Bezier(CubicBezier::new(0.55, 0.0, 0.3, 1.0));
pub const LINEAR: Easing = Easing::Linear;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ends_are_pinned() {
        for easing in [EASE_OUT, EASE_OUT_CSS, PETAL_FLIP_EASE, LINEAR] {
            assert_eq!(easing.apply(0.0), 0.0);
            assert_eq!(easing.apply(1.0), 1.0);
        }
    }

    #[test]
    fn ease_out_front_loads() {
        // cubic-bezier(0.22, 0.61, 0.36, 1) is near 87% at the halfway mark.
        let mid = EASE_OUT.apply(0.5);
        assert!(mid > 0.82 && mid < 0.92, "{mid}");
        assert!((LINEAR.apply(0.25) - 0.25).abs() < 1e-6);
    }
}
