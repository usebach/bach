//! Colour mixing for transitions. Mixing happens in premultiplied sRGB, as
//! CSS colour transitions do, so a fade from a transparent wash never dips
//! through grey.

use gpui::{Hsla, Rgba};

/// `a` at `t = 0`, `b` at `t = 1`.
pub fn mix(a: Hsla, b: Hsla, t: f32) -> Hsla {
    let t = t.clamp(0.0, 1.0);
    if t <= 0.0 {
        return a;
    }
    if t >= 1.0 {
        return b;
    }
    let (a, b) = (Rgba::from(a), Rgba::from(b));
    let alpha = a.a + (b.a - a.a) * t;
    let channel = |x: f32, xa: f32, y: f32, ya: f32| {
        let premultiplied = x * xa + (y * ya - x * xa) * t;
        if alpha > 0.0 {
            premultiplied / alpha
        } else {
            0.0
        }
    };
    Rgba {
        r: channel(a.r, a.a, b.r, b.a),
        g: channel(a.g, a.a, b.g, b.a),
        b: channel(a.b, a.a, b.b, b.a),
        a: alpha,
    }
    .into()
}

/// `color` mixed `amount` of the way toward `toward` — CSS
/// `color-mix(in srgb, color, toward amount)`.
pub fn tint(color: Hsla, toward: Hsla, amount: f32) -> Hsla {
    mix(color, toward, amount)
}

/// The same colour, fully transparent: the start of a fade-in wash.
pub fn clear(color: Hsla) -> Hsla {
    color.opacity(0.0)
}
