//! Shimmering text (SHIMMER): a dim base with a full-strength band sweeping
//! left to right every 720ms, the web's `.text-shimmer`.
//!
//! ```ignore
//! Shimmer::new("Planning the work").color(theme.foreground)
//! ```
//!
//! It inherits font, size and line height from its parent and wraps like
//! normal text. GPUI can't clip a gradient to glyphs, so the band is painted
//! as a few vertical strips, each re-drawing the same shaped text in its own
//! colour under a content mask. Layout is shaped once per frame and cached by
//! GPUI's line cache, so the strips only cost glyph draws.
//!
//! The band position comes from the wall clock, so every live label sweeps
//! in step and a label whose text changes never restarts its sweep (the web
//! syncs its per-glyph animations to the document timeline for the same
//! reason). With reduced motion the label renders flat in the muted colour.

use std::time::{SystemTime, UNIX_EPOCH};

use gpui::{
    point, px, size, App, AvailableSpace, Bounds, ContentMask, Element, ElementId,
    GlobalElementId, Hsla, InspectorElementId, IntoElement, LayoutId, Pixels, SharedString,
    TextAlign, Window, WrappedLine,
};

use super::motion::{reduced_motion, SHIMMER};
use crate::theme::ActiveTheme;

/// How many strips draw the band. Each is a few px wide on typical labels,
/// which reads as a smooth gradient.
const BAND_STRIPS: usize = 12;

pub struct Shimmer {
    text: SharedString,
    color: Option<Hsla>,
    active: bool,
}

impl Shimmer {
    pub fn new(text: impl Into<SharedString>) -> Self {
        Self {
            text: text.into(),
            color: None,
            active: true,
        }
    }

    /// The band's full-strength colour (default: the theme's foreground).
    /// The base is this colour at 50%.
    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Turns the sweep off; the text renders flat in `color`. Lets one
    /// element serve both the live and the settled state without remounting.
    pub fn active(mut self, active: bool) -> Self {
        self.active = active;
        self
    }
}

impl IntoElement for Shimmer {
    type Element = Self;

    fn into_element(self) -> Self {
        self
    }
}

pub struct ShimmerLayout {
    lines: Vec<WrappedLine>,
    line_height: Pixels,
}

/// The sweep's phase right now, 0..1.
fn phase() -> f32 {
    let period = SHIMMER.as_millis().max(1);
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|since| since.as_millis())
        .unwrap_or(0);
    (now % period) as f32 / period as f32
}

/// The band's alpha multiplier at `x` (0..1 across the label): 1 at the
/// band centre falling to 0 at ±0.25 of the width — the web's
/// `base 40%, foreground 50%, base 60%` over a 250%-wide gradient.
fn band_strength(x: f32, centre: f32) -> f32 {
    (1.0 - (x - centre).abs() / 0.25).max(0.0)
}

impl Element for Shimmer {
    type RequestLayoutState = ();
    type PrepaintState = Option<ShimmerLayout>;

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static core::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        _cx: &mut App,
    ) -> (LayoutId, ()) {
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = style.line_height.to_pixels(font_size.into(), window.rem_size());
        let text = self.text.clone();
        let run = style.to_run(text.len());

        let layout_id = window.request_measured_layout(
            Default::default(),
            move |known, available, window, _cx| {
                let wrap_width = known.width.or(match available.width {
                    AvailableSpace::Definite(width) => Some(width),
                    _ => None,
                });
                let Ok(lines) = window.text_system().shape_text(
                    text.clone(),
                    font_size,
                    std::slice::from_ref(&run),
                    wrap_width,
                    None,
                ) else {
                    return size(px(0.), line_height);
                };
                let mut measured = size(px(0.), px(0.));
                for line in &lines {
                    let line_size = line.size(line_height);
                    measured.width = measured.width.max(line_size.width).ceil();
                    measured.height += line_size.height;
                }
                measured
            },
        );
        (layout_id, ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut (),
        window: &mut Window,
        _cx: &mut App,
    ) -> Option<ShimmerLayout> {
        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let line_height = style.line_height.to_pixels(font_size.into(), window.rem_size());
        let run = style.to_run(self.text.len());
        let lines = window
            .text_system()
            .shape_text(
                self.text.clone(),
                font_size,
                std::slice::from_ref(&run),
                Some(bounds.size.width + px(0.5)),
                None,
            )
            .ok()?;
        Some(ShimmerLayout {
            lines: lines.into_iter().collect(),
            line_height,
        })
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut (),
        layout: &mut Option<ShimmerLayout>,
        window: &mut Window,
        cx: &mut App,
    ) {
        let Some(layout) = layout.take() else { return };
        let theme = cx.theme();
        let full = self.color.unwrap_or(theme.foreground);
        let muted = theme.muted_foreground;
        let reduced = reduced_motion(cx);

        let style = window.text_style();
        let font_size = style.font_size.to_pixels(window.rem_size());
        let wrap_width = Some(bounds.size.width + px(0.5));
        let mut paint_in = |color: Hsla, clip: Option<Bounds<Pixels>>, window: &mut Window| {
            let mut run = style.to_run(self.text.len());
            run.color = color;
            let lines = if color == style.color {
                layout.lines.clone()
            } else {
                match window.text_system().shape_text(
                    self.text.clone(),
                    font_size,
                    std::slice::from_ref(&run),
                    wrap_width,
                    None,
                ) {
                    Ok(lines) => lines.into_iter().collect(),
                    Err(_) => return,
                }
            };
            let draw = |window: &mut Window, cx: &mut App| {
                let mut origin = bounds.origin;
                for line in &lines {
                    let _ = line.paint(origin, layout.line_height, TextAlign::Left, None, window, cx);
                    origin.y += line.size(layout.line_height).height;
                }
            };
            match clip {
                Some(clip) => window.with_content_mask(Some(ContentMask { bounds: clip }), |window| {
                    draw(window, cx)
                }),
                None => draw(window, cx),
            }
        };

        if !self.active || reduced {
            let flat = if self.active { muted } else { full };
            paint_in(flat, None, window);
            return;
        }

        // The band centre sweeps from -0.25 to 1.25 of the width, so it
        // enters and leaves fully outside the text and the loop is seamless.
        let width = bounds.size.width.max(px(1.));
        let centre = -0.25 + 1.5 * phase();
        let band_left = bounds.left() + width * (centre - 0.25);
        let band_right = bounds.left() + width * (centre + 0.25);
        let base = full.opacity(0.5);
        let column = |left: Pixels, right: Pixels| {
            Bounds::from_corners(point(left, bounds.top()), point(right, bounds.bottom()))
        };

        if band_left > bounds.left() {
            paint_in(base, Some(column(bounds.left(), band_left.min(bounds.right()))), window);
        }
        if band_right < bounds.right() {
            paint_in(base, Some(column(band_right.max(bounds.left()), bounds.right())), window);
        }
        let strip = (band_right - band_left) / BAND_STRIPS as f32;
        for index in 0..BAND_STRIPS {
            let left = band_left + strip * index as f32;
            let right = left + strip;
            if right <= bounds.left() || left >= bounds.right() {
                continue;
            }
            let middle = ((left + right) / 2. - bounds.left()) / width;
            let strength = band_strength(middle, centre);
            let color = full.opacity(0.5 + 0.5 * strength);
            paint_in(
                color,
                Some(column(left.max(bounds.left()), right.min(bounds.right()))),
                window,
            );
        }

        // The sweep never settles; frames stop once the label stops drawing.
        window.request_animation_frame();
    }
}
