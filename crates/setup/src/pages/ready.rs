//! The first pages: the license, and the card of choices.

use bach_pack::i18n::Key;
use bach_pack::manifest::Manifest;
use gpui::{
    div, px, AnyElement, Context, InteractiveElement, IntoElement, ParentElement, Pixels, SharedString,
    StatefulInteractiveElement, Styled,
};

use super::{footer, subtitle_text, PageLayout};
use crate::keyboard::Target;
use crate::theme::{ActiveTheme, Theme, RADIUS};
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
                .focused(self.is_focused(Target::Cancel))
                .on_click(cx.listener(|this, _, _, cx| this.close(cx)))
                .into_any_element(),
            FormButton::new("setup-accept", texts.get(Key::license_accept))
                .focused(self.is_focused(Target::Primary))
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
        let go = FormButton::new("setup-go", action)
            .focused(self.is_focused(Target::Primary))
            .on_click(cx.listener(|this, _, _, cx| this.start(cx)));
        let go = if self.is_removing() { go.destructive() } else { go };
        // The card itself scrolls, so a focused row can be scrolled to by
        // its index; the scrollbar sits over its right edge.
        let list = div()
            .id("setup-choices")
            .track_scroll(&self.choices_scroll)
            .on_scroll_wheel(cx.listener(|_, _, _, cx| cx.notify()))
            .size_full()
            .overflow_y_scroll()
            .rounded(RADIUS)
            .border_1()
            .border_color(theme.border)
            .children(rows);
        let choices = div()
            .relative()
            .mt(px(28.))
            .min_h_0()
            .flex()
            .flex_col()
            .child(list)
            .children(self.scrollbar(&theme));
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
                            .focused(self.is_focused(Target::Cancel))
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

    /// A thin thumb showing where the card is scrolled, when it scrolls.
    fn scrollbar(&self, theme: &Theme) -> Option<AnyElement> {
        let handle = &self.choices_scroll;
        let hidden = handle.max_offset().height;
        let visible = handle.bounds().size.height;
        if hidden < px(1.) || visible < px(1.) {
            return None;
        }
        let total = visible + hidden;
        let thumb = (visible * (visible / total)).max(px(24.));
        let scrolled = (-handle.offset().y).max(px(0.)).min(hidden);
        let inset = px(6.);
        let track = visible - inset * 2. - thumb;
        let top: Pixels = inset + track.max(px(0.)) * (scrolled / hidden);
        Some(
            div()
                .absolute()
                .top(top)
                .right(px(3.))
                .w(px(4.))
                .h(thumb)
                .rounded(px(2.))
                .bg(theme.muted_foreground.opacity(0.45))
                .into_any_element(),
        )
    }

    /// What the keyboard can reach on each row, in order; None for a row it
    /// skips (the Location row when the folder is fixed).
    pub(crate) fn row_targets(&self) -> Vec<Option<Target>> {
        self.choices()
            .into_iter()
            .map(|choice| match choice.action {
                Action::Location if self.can_change_folder() => Some(Target::Location),
                Action::Location => None,
                Action::Toggle(toggle) => Some(Target::Toggle(toggle)),
            })
            .collect()
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
        let target = match choice.action {
            Action::Location => Target::Location,
            Action::Toggle(toggle) => Target::Toggle(toggle),
        };
        let row = Row::new(choice.title)
            .icon(choice.icon)
            .divided(index > 0)
            .last(last)
            .focused(self.is_focused(target));
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
            // A newer version is installed: say so plainly before replacing it.
            Some(installed) if self.downgrade => (
                texts.get(Key::downgrade_title).into(),
                Some(texts.with(Key::downgrade_subtitle, &[("old_version", installed.version.as_str())]).into()),
                texts.get(Key::downgrade_button).into(),
            ),
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
