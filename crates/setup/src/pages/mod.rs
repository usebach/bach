//! The setup window's pages. One calm column: the mark, a heading and a
//! line of copy, then the choices in a bordered card, and the buttons along
//! the bottom. Each page rises in as it replaces the last, so nothing jumps.
//!
//! License (when there is one) → Ready → Working (the mark conducts while
//! a bar fills) → Done, or Failed with a readable sentence and "Try again".
//!
//! - `ready.rs`: the license and the choices.
//! - `finish.rs`: working, done and failed.

mod finish;
mod ready;

use bach_pack::i18n::Key;
use gpui::{
    div, img, px, AnyElement, Context, ElementId, FontWeight, IntoElement, ObjectFit, ParentElement, Render,
    SharedString, Styled, StyledImage, Window,
};

use crate::assets::LOGO_PATH;
use crate::package::Logo;
use crate::theme::{ActiveTheme, Theme, FONT_SANS};
use crate::ui::motion::PAGE_ENTER;
use crate::ui::prelude::*;
use crate::view::{SetupView, Stage};

pub use ready::row_count;

/// The mark and the page icons share one box, so pages swap in place.
const MARK_SLOT: f32 = 56.;

impl Render for SetupView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let (kind, body) = match self.stage.clone() {
            Stage::License => ("license", self.license_page(cx)),
            Stage::Ready => ("ready", self.ready_page(cx)),
            Stage::Working { progress, label } => ("working", self.working_page(progress, label, &theme)),
            Stage::Done => ("done", self.done_page(cx)),
            Stage::Failed(message) => ("failed", self.failed_page(message, cx)),
        };
        let page_id = ElementId::Name(format!("setup-page-{kind}-{}", self.attempt).into());
        frame(Appear::new(page_id, PAGE_ENTER, body).flex_1().min_h_0().flex().flex_col(), &theme)
    }
}

impl SetupView {
    /// The footer's line: publisher and version (or the developer's own
    /// line), and a preview's tag.
    fn meta(&self) -> SharedString {
        let line = self.texts.get(Key::footer);
        if self.preview {
            format!("{line} · {}", self.texts.get(Key::preview)).into()
        } else {
            line.into()
        }
    }

    /// The app's logo, or the Bach mark (conducting while setup works), or
    /// nothing when the app asked for no logo.
    fn mark(&self, working: bool, theme: &Theme) -> Option<AnyElement> {
        if self.package.manifest.look.hide_logo {
            return None;
        }
        Some(match &self.package.logo {
            Some(Logo::Svg(_)) => gpui::svg()
                .path(LOGO_PATH)
                .size(px(MARK_SLOT))
                .text_color(theme.foreground)
                .into_any_element(),
            Some(Logo::Png(image)) => img(image.clone())
                .size(px(MARK_SLOT))
                .object_fit(ObjectFit::Contain)
                .into_any_element(),
            None if working => BachMark::new("setup-mark").size(68.).slot(MARK_SLOT).into_any_element(),
            None => page_icon("bach", theme.foreground),
        })
    }
}

/// The window's padding and type, shared by every view.
pub fn frame(body: impl IntoElement, theme: &Theme) -> impl IntoElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .bg(theme.surface)
        .text_color(theme.foreground)
        .font_family(FONT_SANS)
        .px(px(32.))
        .pt(px(44.))
        .pb(px(24.))
        .child(body)
}

/// One page: mark, heading, copy line, the content and the footer.
pub struct PageLayout {
    /// Held in the middle, for the pages with nothing to choose.
    pub centered: bool,
    pub mark: Option<AnyElement>,
    pub title: SharedString,
    pub subtitle: Option<AnyElement>,
    pub content: Option<AnyElement>,
    /// The content takes all the room left (a long license).
    pub grow: bool,
    pub footer: AnyElement,
}

impl PageLayout {
    pub fn new(title: impl Into<SharedString>, footer: AnyElement) -> Self {
        Self { centered: false, mark: None, title: title.into(), subtitle: None, content: None, grow: false, footer }
    }

    pub fn render(self) -> AnyElement {
        let heading_gap = if self.mark.is_some() { 18. } else { 0. };
        let content = self.content.map(|content| {
            let holder = div().flex().flex_col().min_h_0().child(content);
            if self.grow { holder.flex_1() } else { holder.flex_shrink() }
        });
        div()
            .size_full()
            .flex()
            .flex_col()
            .children(self.centered.then(|| div().flex_1()))
            .child(
                div()
                    .flex()
                    .flex_none()
                    .flex_col()
                    .items_center()
                    .children(
                        self.mark.map(|mark| div().size(px(MARK_SLOT)).flex().items_center().justify_center().child(mark)),
                    )
                    .child(div().mt(px(heading_gap)).max_w(px(460.)).text_heading().text_center().child(self.title))
                    .children(self.subtitle),
            )
            .children(content)
            .children((!self.grow).then(|| div().flex_1()))
            .child(self.footer)
            .into_any_element()
    }
}

/// An icon drawn in the mark's slot.
pub fn page_icon(name: &'static str, color: gpui::Hsla) -> AnyElement {
    Icon::new(name).size(MARK_SLOT - 4.).color(color).into_any_element()
}

pub fn subtitle_text(text: impl Into<SharedString>, theme: &Theme) -> AnyElement {
    div()
        .mt(px(6.))
        .max_w(px(420.))
        .text_center()
        .text_size(px(14.))
        .line_height(px(20.))
        .text_color(theme.muted_foreground)
        .child(text.into())
        .into_any_element()
}

/// The meta line on the left, the buttons on the right. Always 32px tall,
/// so pages without buttons keep the same frame.
pub fn footer(meta: Option<SharedString>, buttons: Vec<AnyElement>, theme: &Theme) -> AnyElement {
    div()
        .mt(px(16.))
        .h(px(32.))
        .flex()
        .flex_none()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_ellipsis()
                .text_meta()
                .font_weight(FontWeight::NORMAL)
                .text_color(theme.muted_foreground)
                .children(meta),
        )
        .children(buttons)
        .into_any_element()
}
