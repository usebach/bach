//! Using setup without a mouse.
//!
//! Tab and Shift+Tab (or the arrow keys) walk through the page's controls:
//! the rows of choices, then the buttons. Space or Enter does what a click
//! would; Enter with nothing focused presses the page's main button, and
//! Escape cancels. A focus ring shows only once the keyboard is used, like
//! the web's `:focus-visible`, so mouse users never see one.
//!
//! GPUI's own focus handling would need a focus handle per row; one list
//! of targets per page keeps the order obvious and the code small.

use gpui::{Context, KeyDownEvent, Window};

use crate::view::{SetupView, Stage, Toggle};

/// Something the keyboard can land on.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Target {
    /// The Location row, when the folder may change.
    Location,
    Toggle(Toggle),
    /// A link button: the website, or the uninstaller's feedback page.
    Link,
    /// Cancel or Close: the quiet button.
    Cancel,
    /// The second button: Try again on the requirement page.
    Secondary,
    /// The page's main button: I agree, Install, Open, Try again.
    Primary,
}

impl SetupView {
    /// The page's controls in Tab order.
    pub fn targets(&self) -> Vec<Target> {
        let buttons = |primary: bool| {
            let mut buttons = vec![Target::Cancel];
            if primary {
                buttons.push(Target::Primary);
            }
            buttons
        };
        match &self.stage {
            Stage::Requirement(_) => vec![Target::Cancel, Target::Secondary, Target::Primary],
            Stage::License | Stage::Failed(_) => buttons(true),
            Stage::Ready => self.row_targets().into_iter().flatten().chain(buttons(true)).collect(),
            Stage::Working { .. } => Vec::new(),
            Stage::Done => {
                let link = self.done_link().is_some().then_some(Target::Link);
                link.into_iter().chain(buttons(self.has_open_button())).collect()
            }
        }
    }

    /// The link the last page offers: the website after installing, the
    /// feedback page after removing.
    pub fn done_link(&self) -> Option<&str> {
        let manifest = &self.package.manifest;
        if self.is_removing() {
            manifest.uninstall.feedback_url.as_deref()
        } else if manifest.install.website_button {
            manifest.app.website.as_deref()
        } else {
            None
        }
    }

    /// Whether the last page offers to open the app.
    pub fn has_open_button(&self) -> bool {
        !self.is_removing() && self.package.manifest.install.open_button
    }

    fn focused_target(&self) -> Option<Target> {
        self.focused.and_then(|index| self.targets().get(index).copied())
    }

    pub fn is_focused(&self, target: Target) -> bool {
        self.focused_target() == Some(target)
    }

    pub fn on_key(&mut self, event: &KeyDownEvent, window: &mut Window, cx: &mut Context<Self>) {
        let key = event.keystroke.key.as_str();
        let count = self.targets().len();
        match key {
            "tab" | "down" | "up" if count > 0 => {
                let back = key == "up" || (key == "tab" && event.keystroke.modifiers.shift);
                self.focused = Some(match (self.focused, back) {
                    (None, false) => 0,
                    (None, true) => count - 1,
                    (Some(index), false) => (index + 1) % count,
                    (Some(index), true) => (index + count - 1) % count,
                });
                self.scroll_focused_into_view();
                cx.notify();
            }
            "space" => {
                if let Some(target) = self.focused_target() {
                    self.activate(target, window, cx);
                }
            }
            "enter" => {
                let target = self.focused_target().unwrap_or(Target::Primary);
                self.activate(target, window, cx);
            }
            "escape" => self.close(cx),
            _ => {}
        }
    }

    fn activate(&mut self, target: Target, window: &mut Window, cx: &mut Context<Self>) {
        match target {
            Target::Location => self.choose_folder(window, cx),
            Target::Toggle(toggle) => self.toggle(toggle, cx),
            Target::Cancel => self.close(cx),
            Target::Secondary => self.recheck_requirements(cx),
            Target::Link if self.is_removing() => self.open_feedback(),
            Target::Link => self.open_website(),
            Target::Primary => match self.stage {
                Stage::Requirement(_) => self.get_requirement(),
                Stage::License => self.accept_license(cx),
                Stage::Failed(_) if self.app_is_open() => self.close_app_and_retry(cx),
                Stage::Ready | Stage::Failed(_) => self.start(cx),
                Stage::Done if self.has_open_button() => self.open_app(cx),
                Stage::Done => self.close(cx),
                Stage::Working { .. } => {}
            },
        }
    }

    /// Keeps a focused row visible in a card that scrolls.
    fn scroll_focused_into_view(&self) {
        let Some(target) = self.focused_target() else {
            return;
        };
        if let Some(row) = self.row_targets().iter().position(|row| *row == Some(target)) {
            self.choices_scroll.scroll_to_item(row);
        }
    }
}
