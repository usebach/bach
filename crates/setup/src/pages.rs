//! The setup window's pages. One calm column: the mark, a heading and a
//! line of copy, then the choices in a bordered card, and the buttons along
//! the bottom. Each page rises in as it replaces the last, so nothing jumps.
//!
//! Ready → Working (the mark conducts while a bar fills) → Done, or Failed
//! with a readable sentence and "Try again".

use gpui::{
    div, img, px, AnyElement, Context, ElementId, FontWeight, IntoElement, ObjectFit,
    ParentElement, Render, SharedString, Styled, StyledImage, Window,
};

use crate::assets::LOGO_PATH;
use crate::package::Logo;
use crate::theme::{ActiveTheme, Theme, FONT_SANS};
use crate::ui::motion::PAGE_ENTER;
use crate::ui::prelude::*;
use crate::view::{SetupView, Stage};

/// The mark and the page icons share one box, so pages swap in place.
const MARK_SLOT: f32 = 56.;

impl Render for SetupView {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        let theme = cx.theme().clone();
        let (kind, body) = match self.stage.clone() {
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
    fn ready_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let location = Row::new("Location")
            .icon("folder-filled")
            .description(self.dir.display().to_string());
        let rows = if self.is_removing() {
            let mut rows = vec![location.into_any_element()];
            if !self.package.manifest.uninstall.data.is_empty() {
                let description = self
                    .package
                    .manifest
                    .uninstall
                    .data_description
                    .clone()
                    .unwrap_or_else(|| format!("Settings and files {} saved on this computer.", self.package.name()));
                rows.push(
                    Row::new("Also delete my data")
                        .icon("trash-filled")
                        .description(description)
                        .control(
                            Checkbox::new("setup-delete-data", self.delete_data)
                                .on_toggle(cx.listener(|this, _, _, cx| this.toggle_delete_data(cx))),
                        )
                        .divided(true)
                        .into_any_element(),
                );
            }
            rows
        } else {
            let location = match self.existing {
                // An update stays where it is.
                Some(_) => location,
                None => location.control(
                    FormButton::new("setup-change-folder", "Change")
                        .outline()
                        .on_click(cx.listener(|this, _, window, cx| this.choose_folder(window, cx))),
                ),
            };
            vec![
                location.into_any_element(),
                Row::new("Desktop shortcut")
                    .icon("device-desktop-filled")
                    .description(format!("Start {} from your desktop too.", self.package.name()))
                    .control(
                        Checkbox::new("setup-desktop-shortcut", self.desktop_shortcut)
                            .on_toggle(cx.listener(|this, _, _, cx| this.toggle_desktop_shortcut(cx))),
                    )
                    .divided(true)
                    .into_any_element(),
            ]
        };

        let (title, subtitle, action) = self.ready_copy();
        let go = FormButton::new("setup-go", action).on_click(cx.listener(|this, _, _, cx| this.start(cx)));
        let go = if self.is_removing() { go.destructive() } else { go };
        layout(
            false,
            self.mark(false, &theme),
            title,
            subtitle_text(subtitle, &theme),
            Some(div().mt(px(28.)).child(card(rows, cx)).into_any_element()),
            footer(
                Some(self.meta()),
                vec![
                    FormButton::new("setup-cancel", "Cancel")
                        .outline()
                        .on_click(cx.listener(|this, _, _, cx| this.close(cx)))
                        .into_any_element(),
                    go.into_any_element(),
                ],
                &theme,
            ),
        )
    }

    fn working_page(&self, progress: f32, label: SharedString, theme: &Theme) -> AnyElement {
        let name = self.package.name();
        let title = if self.is_removing() {
            format!("Removing {name}")
        } else if self.existing.is_some() {
            format!("Updating {name}")
        } else {
            format!("Installing {name}")
        };
        let line = div()
            .mt(px(6.))
            .text_size(px(14.))
            .line_height(px(20.))
            .child(Shimmer::new(label).color(theme.muted_foreground));
        let bar = div()
            .mt(px(32.))
            .px(px(56.))
            .child(ProgressBar::new(("setup-progress", self.attempt), progress));
        layout(
            true,
            self.mark(true, theme),
            title.into(),
            line.into_any_element(),
            Some(bar.into_any_element()),
            footer(Some(self.meta()), Vec::new(), theme),
        )
    }

    fn done_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let name = self.package.name().to_string();
        let close = |primary: bool, cx: &mut Context<Self>| {
            let button = FormButton::new("setup-close", "Close").on_click(cx.listener(|this, _, _, cx| this.close(cx)));
            if primary { button } else { button.outline() }.into_any_element()
        };
        if self.is_removing() {
            let has_data = !self.package.manifest.uninstall.data.is_empty();
            let subtitle = match (has_data, self.delete_data) {
                (true, true) => "Its data is gone too.".to_string(),
                (true, false) => "Your data is kept, in case you install it again.".to_string(),
                (false, _) => "Thanks for trying it.".to_string(),
            };
            return layout(
                true,
                page_icon("circle-check-filled", theme.foreground),
                format!("{name} is removed").into(),
                subtitle_text(subtitle.into(), &theme),
                None,
                footer(None, vec![close(true, cx)], &theme),
            );
        }
        let title = if self.existing.is_some() {
            format!("{name} is up to date")
        } else {
            format!("{name} is installed")
        };
        layout(
            true,
            page_icon("circle-check-filled", theme.foreground),
            title.into(),
            subtitle_text("Find it in the Start menu, or open it now.".into(), &theme),
            None,
            footer(
                None,
                vec![
                    close(false, cx),
                    FormButton::new("setup-open", open_label(&name))
                        .on_click(cx.listener(|this, _, _, cx| this.open_app(cx)))
                        .into_any_element(),
                ],
                &theme,
            ),
        )
    }

    fn failed_page(&mut self, message: SharedString, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let name = self.package.name();
        let title = if self.is_removing() {
            format!("{name} couldn't be removed")
        } else if self.existing.is_some() {
            format!("{name} couldn't be updated")
        } else {
            format!("{name} couldn't be installed")
        };
        layout(
            true,
            page_icon("alert-triangle-filled", theme.destructive),
            title.into(),
            subtitle_text(message, &theme),
            None,
            footer(
                None,
                vec![
                    FormButton::new("setup-close", "Close")
                        .outline()
                        .on_click(cx.listener(|this, _, _, cx| this.close(cx)))
                        .into_any_element(),
                    FormButton::new("setup-retry", "Try again")
                        .on_click(cx.listener(|this, _, _, cx| this.start(cx)))
                        .into_any_element(),
                ],
                &theme,
            ),
        )
    }

    /// Heading, copy line and button label for the first page.
    fn ready_copy(&self) -> (SharedString, SharedString, &'static str) {
        let name = self.package.name();
        let version = self.package.version();
        if self.is_removing() {
            return (
                format!("Remove {name}").into(),
                format!("This removes {name} and its shortcuts from this computer.").into(),
                "Remove",
            );
        }
        match &self.existing {
            None => (
                format!("Install {name}").into(),
                "It installs just for you, so no administrator rights are needed.".into(),
                "Install",
            ),
            Some(installed) if installed.version == version => (
                format!("Reinstall {name}").into(),
                format!("{name} {version} is already installed. Reinstalling keeps your data.").into(),
                "Reinstall",
            ),
            Some(installed) => {
                let from = if installed.version.is_empty() {
                    format!("{name} is installed.")
                } else {
                    format!("{name} {} is installed.", installed.version)
                };
                (
                    format!("Update {name}").into(),
                    format!("{from} This updates it to {version} and keeps your data.").into(),
                    "Update",
                )
            }
        }
    }

    /// The footer's line: publisher and version, and a preview's warning.
    fn meta(&self) -> SharedString {
        let manifest = &self.package.manifest;
        let line = format!("{} · Version {}", manifest.app.publisher, manifest.app.version);
        if self.preview {
            format!("{line} · Preview").into()
        } else {
            line.into()
        }
    }

    /// The app's logo, or the Bach mark (conducting while setup works).
    fn mark(&self, working: bool, theme: &Theme) -> AnyElement {
        match &self.package.logo {
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
        }
    }
}

/// "Open Hello", or just "Open" when the name would make a wide button.
fn open_label(name: &str) -> String {
    if name.chars().count() > 18 {
        "Open".into()
    } else {
        format!("Open {name}")
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

/// Mark, heading, copy line and the choices; held in the middle for the
/// pages with nothing to choose. The footer is pinned to the bottom.
pub fn layout(
    centered: bool,
    mark: AnyElement,
    title: SharedString,
    subtitle: AnyElement,
    content: Option<AnyElement>,
    footer: AnyElement,
) -> AnyElement {
    div()
        .size_full()
        .flex()
        .flex_col()
        .children(centered.then(|| div().flex_1()))
        .child(
            div()
                .flex()
                .flex_col()
                .items_center()
                .child(div().size(px(MARK_SLOT)).flex().items_center().justify_center().child(mark))
                .child(div().mt(px(18.)).text_heading().text_center().child(title))
                .child(subtitle),
        )
        .children(content)
        .child(div().flex_1())
        .child(footer)
        .into_any_element()
}

/// An icon drawn in the mark's slot.
pub fn page_icon(name: &'static str, color: gpui::Hsla) -> AnyElement {
    Icon::new(name).size(MARK_SLOT - 4.).color(color).into_any_element()
}

pub fn subtitle_text(text: SharedString, theme: &Theme) -> AnyElement {
    div()
        .mt(px(6.))
        .max_w(px(420.))
        .text_center()
        .text_size(px(14.))
        .line_height(px(20.))
        .text_color(theme.muted_foreground)
        .child(text)
        .into_any_element()
}

/// The meta line on the left, the buttons on the right. Always 32px tall,
/// so pages without buttons keep the same frame.
pub fn footer(meta: Option<SharedString>, buttons: Vec<AnyElement>, theme: &Theme) -> AnyElement {
    div()
        .h(px(32.))
        .flex()
        .items_center()
        .gap(px(8.))
        .child(
            div()
                .flex_1()
                .min_w_0()
                .overflow_hidden()
                .whitespace_nowrap()
                .text_meta()
                .font_weight(FontWeight::NORMAL)
                .text_color(theme.muted_foreground)
                .children(meta),
        )
        .children(buttons)
        .into_any_element()
}
