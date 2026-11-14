//! The window's smaller actions: requirements, links, and closing the app
//! so an update can go on. The install itself lives in `view.rs`.

use bach_pack::i18n::Key;
use gpui::Context;

use crate::system::apps;
use crate::view::{SetupView, Stage};

impl SetupView {
    /// Looks again after the user installed what was missing, and moves on
    /// once nothing is.
    pub fn recheck_requirements(&mut self, cx: &mut Context<Self>) {
        let missing = crate::start::missing_requirements(&self.package.manifest).first().copied();
        self.stage = match missing {
            Some(index) => Stage::Requirement(index),
            None if self.license_pending => Stage::License,
            None => Stage::Ready,
        };
        self.focused = None;
        cx.notify();
    }

    /// Opens the download page of the requirement on screen.
    pub fn get_requirement(&mut self) {
        if let Stage::Requirement(index) = self.stage {
            if let Some(requirement) = self.package.manifest.requirements.get(index) {
                apps::open_url(&requirement.url);
            }
        }
    }

    pub fn open_website(&mut self) {
        if let Some(url) = &self.package.manifest.app.website {
            apps::open_url(url);
        }
    }

    pub fn open_feedback(&mut self) {
        if let Some(url) = &self.package.manifest.uninstall.feedback_url {
            apps::open_url(url);
        }
    }

    /// Whether the last attempt failed because the app is open, so the
    /// failed page can offer to close it.
    pub fn app_is_open(&self) -> bool {
        !self.is_removing()
            && matches!(&self.stage, Stage::Failed(message) if message.as_ref() == self.texts.get(Key::error_still_open))
    }

    /// Asks the app to close (it may ask the user to save first), then
    /// tries again. Nothing is ever killed.
    pub fn close_app_and_retry(&mut self, cx: &mut Context<Self>) {
        if self.closing_app || self.preview {
            return;
        }
        self.closing_app = true;
        cx.notify();
        let program = self.package.manifest.executable_in(&self.dir);
        let closing = cx.background_executor().spawn(async move { apps::close_programs_using(&[program]) });
        self.closer = Some(cx.spawn(async move |this, cx| {
            let closed = closing.await;
            this.update(cx, |this, cx| {
                this.closing_app = false;
                this.closer = None;
                if closed {
                    crate::log::line("The app closed; trying again.");
                    this.start(cx);
                } else {
                    crate::log::line("The app didn't close.");
                    this.stage = Stage::Failed(this.texts.get(Key::error_close_app).into());
                    cx.notify();
                }
            })
            .ok();
        }));
    }
}
