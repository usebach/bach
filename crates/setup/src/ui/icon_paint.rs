//! The element behind `ui::Icon`. GPUI's own `svg()` only paints when it is
//! given a colour of its own, so an icon without `.color()` would vanish.
//! This one falls back to the inherited text colour at paint time, which is
//! what the web's `currentColor` does.

use gpui::{
    point, px, size, App, Bounds, Element, ElementId, GlobalElementId, Hsla,
    InspectorElementId, IntoElement, LayoutId, Pixels, Point, Radians, SharedString, Style,
    TransformationMatrix, Window,
};

pub struct IconPaint {
    path: SharedString,
    size: Pixels,
    color: Option<Hsla>,
    scale: f32,
    rotate: Radians,
    offset: Point<Pixels>,
}

impl IconPaint {
    pub fn new(
        path: SharedString,
        size: Pixels,
        color: Option<Hsla>,
        scale: f32,
        rotate: Radians,
        offset: Point<Pixels>,
    ) -> Self {
        Self {
            path,
            size,
            color,
            scale,
            rotate,
            offset,
        }
    }

    /// Scale and rotate about the centre, then offset; the same order
    /// GPUI's `Transformation` uses.
    fn matrix(&self, bounds: Bounds<Pixels>, scale_factor: f32) -> TransformationMatrix {
        let center = bounds.center();
        let back = point(-center.x, -center.y);
        TransformationMatrix::unit()
            .translate(center.scale(scale_factor) + self.offset.scale(scale_factor))
            .rotate(self.rotate)
            .scale(size(self.scale, self.scale))
            .translate(back.scale(scale_factor))
    }
}

impl IntoElement for IconPaint {
    type Element = Self;

    fn into_element(self) -> Self::Element {
        self
    }
}

impl Element for IconPaint {
    type RequestLayoutState = ();
    type PrepaintState = ();

    fn id(&self) -> Option<ElementId> {
        None
    }

    fn source_location(&self) -> Option<&'static std::panic::Location<'static>> {
        None
    }

    fn request_layout(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        window: &mut Window,
        cx: &mut App,
    ) -> (LayoutId, Self::RequestLayoutState) {
        let style = Style {
            size: size(self.size.into(), self.size.into()),
            flex_grow: 0.,
            flex_shrink: 0.,
            ..Style::default()
        };
        (window.request_layout(style, None, cx), ())
    }

    fn prepaint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        _bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _window: &mut Window,
        _cx: &mut App,
    ) -> Self::PrepaintState {
    }

    fn paint(
        &mut self,
        _id: Option<&GlobalElementId>,
        _inspector_id: Option<&InspectorElementId>,
        bounds: Bounds<Pixels>,
        _request_layout: &mut Self::RequestLayoutState,
        _prepaint: &mut Self::PrepaintState,
        window: &mut Window,
        cx: &mut App,
    ) {
        let color = self.color.unwrap_or_else(|| window.text_style().color);
        if color.a <= 0. {
            return;
        }
        let matrix = self.matrix(bounds, window.scale_factor());
        if let Err(err) = window.paint_svg(bounds, self.path.clone(), matrix, color, cx) {
            eprintln!("bach-setup: couldn't paint {}: {err:#}", self.path);
        }
    }
}

/// `point(px(x), px(y))` from the builder's plain floats.
pub fn offset(x: f32, y: f32) -> Point<Pixels> {
    point(px(x), px(y))
}
