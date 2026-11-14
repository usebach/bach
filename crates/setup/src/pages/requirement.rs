//! The page for something the app needs that isn't on the computer yet.

use bach_pack::i18n::Key;
use gpui::{AnyElement, Context, IntoElement};

use super::{footer, page_icon, subtitle_text, PageLayout};
use crate::keyboard::Target;
use crate::theme::ActiveTheme;
use crate::ui::prelude::*;
use crate::view::SetupView;

impl SetupView {
    pub(super) fn requirement_page(&mut self, index: usize, cx: &mut Context<Self>) -> AnyElement {
        let theme = cx.theme().clone();
        let texts = self.texts.clone();
        let name = self
            .package
            .manifest
            .requirements
            .get(index)
            .map(|requirement| requirement.name.clone())
            .unwrap_or_default();
        let values = [("requirement", name.as_str())];
        let buttons = vec![
            FormButton::new("setup-cancel", texts.get(Key::cancel))
                .outline()
                .focused(self.is_focused(Target::Cancel))
                .on_click(cx.listener(|this, _, _, cx| this.close(cx)))
                .into_any_element(),
            FormButton::new("setup-recheck", texts.get(Key::retry))
                .outline()
                .focused(self.is_focused(Target::Secondary))
                .on_click(cx.listener(|this, _, _, cx| this.recheck_requirements(cx)))
                .into_any_element(),
            FormButton::new("setup-get", texts.with(Key::requirement_button, &values))
                .focused(self.is_focused(Target::Primary))
                .on_click(cx.listener(|this, _, _, _| this.get_requirement()))
                .into_any_element(),
        ];
        PageLayout {
            centered: true,
            mark: Some(page_icon("puzzle-filled", theme.foreground)),
            subtitle: Some(subtitle_text(texts.with(Key::requirement_subtitle, &values), &theme)),
            ..PageLayout::new(texts.with(Key::requirement_title, &values), footer(None, buttons, &theme))
        }
        .render()
    }
}
