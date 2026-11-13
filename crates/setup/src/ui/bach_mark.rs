//! The Bach mark, live: the default mark while setup works, from
//! brand/bach-mark-animated.svg. Twelve
//! petals flip across their own axes on a 2.4s cycle, starting 40ms apart.
//!
//! GPUI scales each petal around its fill centre, then rotates it into
//! place and translates it along its axis. Reduced motion keeps it still.

use std::time::Instant;

use gpui::{
    div, point, px, radians, size, svg, App, ElementId, IntoElement,
    ParentElement, RenderOnce, Styled, Transformation, Window,
};

use super::motion::{reduced_motion, PETAL_FLIP_EASE};
use crate::theme::ActiveTheme;

struct Petal {
    /// Distance from the mark's centre to the petal's fill centre.
    distance: f32,
    /// Side of the exported square box, in viewBox units.
    side: f32,
}

/// Exported by scripts/export-brand.mjs, in clockwise order from the top.
const PETALS: [Petal; 4] = [
    Petal { distance: 83.001, side: 112.0 },
    Petal { distance: 75.001, side: 97.0 },
    Petal { distance: 66.999, side: 81.0 },
    Petal { distance: 59.000, side: 64.0 },
];

/// The source viewBox: `-160 -160 320 320`.
const VIEW_SIZE: f32 = 320.0;
const PETAL_COUNT: usize = 12;
const CASCADE_DELAY: f32 = 0.04;

const CYCLE: f32 = 2.4;
/// Keyframes: flipped over at 32%, home again at 62%, resting to 100%.
const FLIPPED_AT: f32 = 0.32;
const HOME_AT: f32 = 0.62;
/// At full flip: pushed 9 units outward, 1.2× along the axis, mirrored.
const PUSH: f32 = 9.0;

/// How far into its flip a petal is, 0 (rest) → 1 (fully flipped).
fn flip_amount(elapsed: f32, delay: f32) -> f32 {
    if elapsed < delay {
        return 0.0;
    }
    let t = ((elapsed - delay) % CYCLE) / CYCLE;
    if t < FLIPPED_AT {
        PETAL_FLIP_EASE.apply(t / FLIPPED_AT)
    } else if t < HOME_AT {
        1.0 - PETAL_FLIP_EASE.apply((t - FLIPPED_AT) / (HOME_AT - FLIPPED_AT))
    } else {
        0.0
    }
}

#[derive(IntoElement)]
pub struct BachMark {
    id: ElementId,
    size: f32,
    slot: f32,
}

impl BachMark {
    /// The mark at 26px, centred in a 20px slot.
    pub fn new(id: impl Into<ElementId>) -> Self {
        Self {
            id: id.into(),
            size: 26.0,
            slot: 20.0,
        }
    }

    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// The layout slot it's centred in (it may overflow it).
    pub fn slot(mut self, slot: f32) -> Self {
        self.slot = slot;
        self
    }
}

struct MarkState {
    mounted: Instant,
}

impl RenderOnce for BachMark {
    fn render(self, window: &mut Window, cx: &mut App) -> impl IntoElement {
        let state = window.use_keyed_state(self.id, cx, |_, _| MarkState {
            mounted: Instant::now(),
        });
        let still = reduced_motion(cx);
        let elapsed = if still {
            0.0
        } else {
            state.read(cx).mounted.elapsed().as_secs_f32()
        };
        if !still {
            window.request_animation_frame();
        }

        let color = cx.theme().foreground;
        let scale = self.size / VIEW_SIZE;

        let petals = (0..PETAL_COUNT).map(|index| {
            let shape = index % PETALS.len();
            let petal = &PETALS[shape];
            let u = flip_amount(elapsed, index as f32 * CASCADE_DELAY);
            let axis = (-90.0 + index as f32 * 30.0).to_radians();
            let (sin, cos) = axis.sin_cos();
            let push = PUSH * u * scale;
            let side = petal.side * scale;
            let distance = petal.distance * scale;
            let origin = (self.size - side) / 2.0;
            svg()
                .path(format!("bach/petal-{}.svg", shape + 1))
                .absolute()
                .left(px(origin + distance * cos))
                .top(px(origin + distance * sin))
                .size(px(side))
                .text_color(color)
                .with_transformation(
                    Transformation::scale(size(1.0 + 0.2 * u, 1.0 - 2.0 * u))
                        .with_rotation(radians(axis))
                        .with_translation(point(px(push * cos), px(push * sin))),
                )
        });

        let offset = (self.slot - self.size) / 2.0;
        div()
            .relative()
            .flex_none()
            .size(px(self.slot))
            .child(
                div()
                    .absolute()
                    .left(px(offset))
                    .top(px(offset))
                    .size(px(self.size))
                    .children(petals),
            )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn petals_rest_between_flips() {
        assert_eq!(flip_amount(0.0, 0.0), 0.0);
        assert!((flip_amount(CYCLE * FLIPPED_AT, 0.0) - 1.0).abs() < 1e-3);
        assert_eq!(flip_amount(CYCLE * 0.8, 0.0), 0.0);
        // Waiting out the cascade delay counts as rest.
        assert_eq!(flip_amount(0.1, 0.2), 0.0);
    }
}
