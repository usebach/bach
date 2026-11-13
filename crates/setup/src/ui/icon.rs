//! Tabler icons, exported from the web app's set (scripts/export-icons.mjs).
//!
//! ```ignore
//! Icon::new("folder-filled").size(13.).color(theme.foreground_soft)
//! Icon::new("plus").size(15.).stroke(2.5)        // needs plus@2.5.svg
//! ```
//!
//! Without `.color()` the icon takes the inherited text colour, like the
//! web's `currentColor`.

use gpui::{px, radians, App, Hsla, IntoElement, Point, RenderOnce, SharedString, Window};

use super::icon_paint::{offset, IconPaint};

#[derive(IntoElement, Clone)]
pub struct Icon {
    name: SharedString,
    size: f32,
    stroke: Option<f32>,
    color: Option<Hsla>,
    scale: f32,
    rotate_deg: f32,
    offset: Point<f32>,
}

impl Icon {
    /// An icon by its exported kebab-case name, e.g. "circle-check-filled".
    pub fn new(name: impl Into<SharedString>) -> Self {
        Self {
            name: name.into(),
            size: 16.0,
            stroke: None,
            color: None,
            scale: 1.0,
            rotate_deg: 0.0,
            offset: Point::default(),
        }
    }

    /// Box size in px, the web's `size={…}`.
    pub fn size(mut self, size: f32) -> Self {
        self.size = size;
        self
    }

    /// An outline icon's stroke width in the 24-unit grid (the web's
    /// `stroke={…}`). Tabler's default is 2; other widths need an
    /// `IconName@width` export.
    pub fn stroke(mut self, stroke: f32) -> Self {
        self.stroke = Some(stroke);
        self
    }

    pub fn color(mut self, color: Hsla) -> Self {
        self.color = Some(color);
        self
    }

    /// Paint-time scale about the centre; layout is unaffected.
    pub fn scale(mut self, scale: f32) -> Self {
        self.scale = scale;
        self
    }

    /// Paint-time rotation in degrees, clockwise, about the centre.
    pub fn rotate(mut self, degrees: f32) -> Self {
        self.rotate_deg = degrees;
        self
    }

    /// Paint-time offset in px; layout is unaffected.
    pub fn offset(mut self, x: f32, y: f32) -> Self {
        self.offset = Point { x, y };
        self
    }

    /// The icon's box size in px.
    pub fn box_size(&self) -> f32 {
        self.size
    }

    /// The asset path this icon resolves to.
    pub fn path(&self) -> SharedString {
        match self.stroke {
            Some(stroke) if (stroke - 2.0).abs() > f32::EPSILON => {
                format!("icons/{}@{}.svg", self.name, trim_float(stroke)).into()
            }
            _ => crate::assets::icon(&self.name),
        }
    }
}

/// 2.5 → "2.5", 3.0 → "3", matching the export script's file names.
fn trim_float(value: f32) -> String {
    let text = format!("{value}");
    text.trim_end_matches(".0").to_string()
}

impl RenderOnce for Icon {
    fn render(self, _window: &mut Window, _cx: &mut App) -> impl IntoElement {
        IconPaint::new(
            self.path(),
            px(self.size),
            self.color,
            self.scale,
            radians(self.rotate_deg.to_radians()),
            offset(self.offset.x, self.offset.y),
        )
    }
}
