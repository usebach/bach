//! The pages after the go button: working, done and failed.

use bach_pack::i18n::Key;
use gpui::{div, px, AnyElement, Context, IntoElement, ParentElement, SharedString, Styled};

use super::{footer, page_icon, subtitle_text, PageLayout};
use crate::theme::{ActiveTheme, Theme};
use crate::ui::prelude::*;
use crate::view::SetupView;

impl SetupView {
    pub(super) fn working_page(&self, progress: f32, label: SharedString, theme: &Theme) -> AnyElement {
        let texts = &self.texts;
        let title = if self.is_removing() {
            texts.get(Key::removing)
        } else if self.existing.is_some() {
            texts.get(Key::updating)
        } else {
            texts.get(Key::installing)
        };
        let line = div()
            .mt(px(6.))
            .max_w(px(420.))
            .text_size(px(14.))
            .line_height(px(20.))
            .child(Shimmer::new(label).color(theme.muted_foreground));
        let bar = div()
            .mt(px(32.))
            .px(px(56.))
            .child(ProgressBar::new(("setup-progress", self.attempt), progress));
        PageLayout {
            centered: true,
            mark: self.mark(true, theme),
            subtitle: Some(line.into_any_element()),
            content: Some(bar.into_any_element()),
            ..PageLayout::new(title, footer(Some(self.meta()), Vec::new(), theme))
        }
        .render()
    }

    pub(super) fn done_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let texts = self.texts.clone();
        let close = |primary: bool, cx: &mut Context<Self>| {
            let button = FormButton::new("setup-close", texts.get(Key::close))
                .on_click(cx.listener(|this, _, _, cx| this.close(cx)));
            if primary { button } else { button.outline() }.into_any_element()
        };
        let done_icon = page_icon("circle-check-filled", theme.foreground);
        if self.is_removing() {
            let has_data = !self.package.manifest.uninstall.data.is_empty();
            let subtitle = match (has_data, self.delete_data) {
                (true, true) => texts.get(Key::removed_data_deleted),
                (true, false) => texts.get(Key::removed_data_kept),
                (false, _) => texts.get(Key::removed_subtitle),
            };
            return PageLayout {
                centered: true,
                mark: Some(done_icon),
                subtitle: Some(subtitle_text(subtitle, &theme)),
                ..PageLayout::new(texts.get(Key::removed_title), footer(None, vec![close(true, cx)], &theme))
            }
            .render();
        }
        let title = if self.existing.is_some() { texts.get(Key::updated_title) } else { texts.get(Key::installed_title) };
        let manifest = &self.package.manifest;
        let subtitle = if self.choices.start_menu_shortcut && manifest.install.open_button {
            texts.get(Key::installed_subtitle)
        } else {
            texts.get(Key::installed_subtitle_plain)
        };
        let mut buttons = vec![close(!manifest.install.open_button, cx)];
        if manifest.install.open_button {
            buttons.push(
                FormButton::new("setup-open", open_label(&texts))
                    .on_click(cx.listener(|this, _, _, cx| this.open_app(cx)))
                    .into_any_element(),
            );
        }
        PageLayout {
            centered: true,
            mark: Some(done_icon),
            subtitle: Some(subtitle_text(subtitle, &theme)),
            ..PageLayout::new(title, footer(None, buttons, &theme))
        }
        .render()
    }

    pub(super) fn failed_page(&mut self, message: SharedString, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let texts = &self.texts;
        let title = if self.is_removing() {
            texts.get(Key::remove_failed)
        } else if self.existing.is_some() {
            texts.get(Key::update_failed)
        } else {
            texts.get(Key::install_failed)
        };
        PageLayout {
            centered: true,
            mark: Some(page_icon("alert-triangle-filled", theme.destructive)),
            subtitle: Some(subtitle_text(message, &theme)),
            ..PageLayout::new(
                title,
                footer(
                    None,
                    vec![
                        FormButton::new("setup-close", texts.get(Key::close))
                            .outline()
                            .on_click(cx.listener(|this, _, _, cx| this.close(cx)))
                            .into_any_element(),
                        FormButton::new("setup-retry", texts.get(Key::retry))
                            .on_click(cx.listener(|this, _, _, cx| this.start(cx)))
                            .into_any_element(),
                    ],
                    &theme,
                ),
            )
        }
        .render()
    }
}

/// "Open Hello", or just "Open" when the label would make a wide button.
fn open_label(texts: &crate::texts::Texts) -> String {
    let full = texts.get(Key::open);
    if full.chars().count() > 24 {
        texts.get(Key::open_short)
    } else {
        full
    }
}
