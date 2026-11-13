//! The first pages: the license, and the card of choices.

use bach_pack::i18n::Key;
use bach_pack::manifest::Manifest;
use gpui::{div, px, AnyElement, Context, InteractiveElement, IntoElement, ParentElement, SharedString, StatefulInteractiveElement, Styled};

use super::{footer, subtitle_text, PageLayout};
use crate::theme::{ActiveTheme, RADIUS};
use crate::ui::prelude::*;
use crate::view::{SetupView, Toggle};

/// One row of the card before it's drawn.
struct Choice {
    title: String,
    description: Option<String>,
    icon: &'static str,
    action: Action,
}

enum Action {
    /// Shows where the app goes; clicking picks another folder when allowed.
    Location,
    Toggle(Toggle),
}

/// How many rows the install page shows, so the window can make room.
pub fn row_count(manifest: &Manifest, removing: bool) -> usize {
    if removing {
        return 1 + usize::from(!manifest.uninstall.data.is_empty());
    }
    let install = &manifest.install;
    let offered = [install.desktop_shortcut, install.start_menu_shortcut, install.start_with_windows, install.add_to_path]
        .iter()
        .filter(|choice| choice.offered())
        .count();
    1 + manifest.components.len() + offered
}

impl SetupView {
    pub(super) fn license_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let texts = &self.texts;
        let license = self.package.manifest.license(texts.language.code).unwrap_or_default().to_string();
        let paragraphs = license.split("\n\n").map(|paragraph| {
            div().text_size(px(13.)).line_height(px(19.)).child(SharedString::from(paragraph.trim().to_string()))
        });
        let text = div()
            .id("setup-license")
            .size_full()
            .overflow_y_scroll()
            .p(px(16.))
            .flex()
            .flex_col()
            .gap(px(10.))
            .children(paragraphs);
        // The card fills the room left, so a short license and a long one
        // make the same calm page; a long one scrolls inside it.
        let content = div()
            .mt(px(24.))
            .flex_1()
            .min_h_0()
            .rounded(RADIUS)
            .border_1()
            .border_color(theme.border)
            .child(text);
        let buttons = vec![
            FormButton::new("setup-cancel", texts.get(Key::cancel))
                .outline()
                .on_click(cx.listener(|this, _, _, cx| this.close(cx)))
                .into_any_element(),
            FormButton::new("setup-accept", texts.get(Key::license_accept))
                .on_click(cx.listener(|this, _, _, cx| this.accept_license(cx)))
                .into_any_element(),
        ];
        PageLayout {
            mark: self.mark(false, &theme),
            subtitle: Some(subtitle_text(texts.get(Key::license_subtitle), &theme)),
            content: Some(content.into_any_element()),
            grow: true,
            ..PageLayout::new(texts.get(Key::license_title), footer(Some(self.meta()), buttons, &theme))
        }
        .render()
    }

    pub(super) fn ready_page(&mut self, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let choices = self.choices();
        let count = choices.len();
        let rows: Vec<AnyElement> = choices
            .into_iter()
            .enumerate()
            .map(|(index, choice)| self.row(choice, index, index + 1 == count, cx))
            .collect();

        let (title, subtitle, action) = self.ready_copy();
        let go = FormButton::new("setup-go", action).on_click(cx.listener(|this, _, _, cx| this.start(cx)));
        let go = if self.is_removing() { go.destructive() } else { go };
        let choices = div()
            .id("setup-choices")
            .mt(px(28.))
            .min_h_0()
            .overflow_y_scroll()
            .child(card(rows, cx));
        PageLayout {
            mark: self.mark(false, &theme),
            subtitle: subtitle.map(|text| subtitle_text(text, &theme)),
            content: Some(choices.into_any_element()),
            ..PageLayout::new(
                title,
                footer(
                    Some(self.meta()),
                    vec![
                        FormButton::new("setup-cancel", self.texts.get(Key::cancel))
                            .outline()
                            .on_click(cx.listener(|this, _, _, cx| this.close(cx)))
                            .into_any_element(),
                        go.into_any_element(),
                    ],
                    &theme,
                ),
            )
        }
        .render()
    }

    /// The rows of the card, in order.
    fn choices(&self) -> Vec<Choice> {
        let texts = &self.texts;
        let manifest = &self.package.manifest;
        let mut rows = vec![Choice {
            title: texts.get(Key::location),
            description: Some(self.dir.display().to_string()),
            icon: "folder-filled",
            action: Action::Location,
        }];
        if self.is_removing() {
            if !manifest.uninstall.data.is_empty() {
                rows.push(Choice {
                    title: texts.get(Key::delete_data),
                    description: Some(texts.get(Key::delete_data_description)),
                    icon: "trash-filled",
                    action: Action::Toggle(Toggle::DeleteData),
                });
            }
            return rows;
        }
        for (index, component) in manifest.components.iter().enumerate() {
            let (title, description) = texts.component(component, manifest);
            rows.push(Choice { title, description, icon: "puzzle-filled", action: Action::Toggle(Toggle::Component(index)) });
        }
        let install = &manifest.install;
        let builtin = [
            (install.desktop_shortcut, Key::desktop_shortcut, Key::desktop_shortcut_description, "device-desktop-filled", Toggle::DesktopShortcut),
            (install.start_menu_shortcut, Key::start_menu, Key::start_menu_description, "layout-grid-filled", Toggle::StartMenu),
            (install.start_with_windows, Key::start_with_windows, Key::start_with_windows_description, "bolt-filled", Toggle::StartWithWindows),
            (install.add_to_path, Key::add_to_path, Key::add_to_path_description, "terminal-2", Toggle::AddToPath),
        ];
        for (choice, title, description, icon, toggle) in builtin {
            if choice.offered() {
                rows.push(Choice {
                    title: texts.get(title),
                    description: Some(texts.get(description)),
                    icon,
                    action: Action::Toggle(toggle),
                });
            }
        }
        rows
    }

    fn row(&self, choice: Choice, index: usize, last: bool, cx: &mut Context<Self>) -> AnyElement {
        let row = Row::new(choice.title).icon(choice.icon).divided(index > 0).last(last);
        let row = match choice.description {
            Some(description) => row.description(description),
            None => row,
        };
        match choice.action {
            Action::Location if self.can_change_folder() => row
                .control(
                    FormButton::new("setup-change-folder", self.texts.get(Key::change))
                        .outline()
                        .on_click(cx.listener(|this, _, window, cx| this.choose_folder(window, cx))),
                )
                .on_click("setup-row-location", cx.listener(|this, _, window, cx| this.choose_folder(window, cx)))
                .into_any_element(),
            Action::Location => row.into_any_element(),
            Action::Toggle(toggle) => row
                .control(
                    Checkbox::new(("setup-check", index), self.is_on(toggle))
                        .on_toggle(cx.listener(move |this, _, _, cx| this.toggle(toggle, cx))),
                )
                .on_click(("setup-row", index), cx.listener(move |this, _, _, cx| this.toggle(toggle, cx)))
                .into_any_element(),
        }
    }

    /// Heading, copy line and button label for the first page.
    fn ready_copy(&self) -> (SharedString, Option<SharedString>, SharedString) {
        let texts = &self.texts;
        let version = self.package.version();
        if self.is_removing() {
            return (
                texts.get(Key::remove_title).into(),
                Some(texts.get(Key::remove_subtitle).into()),
                texts.get(Key::remove_button).into(),
            );
        }
        match &self.existing {
            None => {
                // No built-in line: the app's own description, if it has one.
                let subtitle = texts.get(Key::install_subtitle);
                let subtitle = if subtitle.trim().is_empty() {
                    self.package.manifest.app.description.clone().filter(|text| !text.trim().is_empty())
                } else {
                    Some(subtitle)
                };
                (texts.get(Key::install_title).into(), subtitle.map(Into::into), texts.get(Key::install_button).into())
            }
            Some(installed) if installed.version == version => (
                texts.get(Key::reinstall_title).into(),
                Some(texts.get(Key::reinstall_subtitle).into()),
                texts.get(Key::reinstall_button).into(),
            ),
            Some(installed) => {
                let old = if installed.version.is_empty() { "?" } else { installed.version.as_str() };
                (
                    texts.get(Key::update_title).into(),
                    Some(texts.with(Key::update_subtitle, &[("old_version", old)]).into()),
                    texts.get(Key::update_button).into(),
                )
            }
        }
    }
}
