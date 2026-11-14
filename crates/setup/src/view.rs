//! The setup window's state: which page it's on, the choices on the first
//! page, and the running job. Drawing lives in `pages/`.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use bach_pack::i18n::Key;
use bach_pack::manifest::{Choices, Manifest};
use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{Context, FocusHandle, ScrollHandle, SharedString, Task, Window};

use crate::launch::Mode;
use crate::package::Package;
use crate::system::Installed;
use crate::texts::Texts;
use crate::work::{self, Event, InstallPlan, RemovePlan};

/// The bar gets this long to glide to full before the last page shows.
const FINISH_HOLD: Duration = Duration::from_millis(650);

#[derive(Clone, Debug)]
pub enum Stage {
    /// Something the app needs is missing (a requirement, by index).
    Requirement(usize),
    /// The license to accept before anything else.
    License,
    /// The first page: options and the go button.
    Ready,
    Working {
        progress: f32,
        label: SharedString,
    },
    Done,
    Failed(SharedString),
}

/// One box on the install page.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Toggle {
    DesktopShortcut,
    StartMenu,
    StartWithWindows,
    AddToPath,
    AutoUpdate,
    /// A component, by its index in the manifest.
    Component(usize),
    DeleteData,
}

pub struct SetupView {
    pub package: Arc<Package>,
    pub texts: Arc<Texts>,
    pub mode: Mode,
    /// Pretend: nothing is installed or removed.
    pub preview: bool,
    /// An install already on this computer (install mode only).
    pub existing: Option<Installed>,
    /// Where the app goes (or is, when removing).
    pub dir: PathBuf,
    pub choices: Choices,
    pub delete_data: bool,
    /// The installed version is newer than this one.
    pub downgrade: bool,
    /// The license comes after the requirements are met.
    pub license_pending: bool,
    /// Waiting for the app to close after "Close and continue".
    pub closing_app: bool,
    pub stage: Stage,
    /// Counts attempts, so each one's page and bar start fresh.
    pub attempt: usize,
    /// Receives the keys; see `keyboard.rs`.
    pub focus_handle: FocusHandle,
    /// The control the keyboard is on, by its place in `targets()`. None
    /// until Tab is pressed, so mouse users never see a focus ring.
    pub focused: Option<usize>,
    /// The card of choices, for its scrollbar and for keeping the focused
    /// row in view.
    pub choices_scroll: ScrollHandle,
    job: Option<Task<()>>,
    picker: Option<Task<()>>,
    pub(crate) closer: Option<Task<()>>,
}

impl SetupView {
    pub fn new(package: Arc<Package>, texts: Arc<Texts>, mode: Mode, preview: bool, cx: &mut Context<Self>) -> Self {
        let manifest = &package.manifest;
        let (existing, previous, dir, choices, downgrade) = match &mode {
            Mode::Install => {
                let start = crate::start::install(manifest);
                (start.existing, start.previous, start.dir, start.choices, start.downgrade)
            }
            Mode::Remove { dir } => (None, None, dir.clone(), Choices::default(), false),
        };
        // Screenshots for docs shouldn't show whose computer took them.
        let dir = match std::env::var_os("BACH_PREVIEW_DIR") {
            Some(shown) if preview => PathBuf::from(shown),
            _ => dir,
        };
        let license_pending = needs_license(manifest, previous.as_ref(), &mode, &texts);
        let missing = match &mode {
            // Only reads, so a preview shows the page too.
            Mode::Install => crate::start::missing_requirements(manifest).first().copied(),
            _ => None,
        };
        let stage = match missing {
            Some(index) => Stage::Requirement(index),
            None if license_pending => Stage::License,
            None => Stage::Ready,
        };
        Self {
            package,
            texts,
            mode,
            preview,
            existing,
            dir,
            choices,
            delete_data: false,
            downgrade,
            license_pending,
            closing_app: false,
            stage,
            attempt: 0,
            focus_handle: cx.focus_handle(),
            focused: None,
            choices_scroll: ScrollHandle::new(),
            job: None,
            picker: None,
            closer: None,
        }
    }

    pub fn is_removing(&self) -> bool {
        matches!(self.mode, Mode::Remove { .. })
    }

    pub fn is_on(&self, toggle: Toggle) -> bool {
        let choices = &self.choices;
        match toggle {
            Toggle::DesktopShortcut => choices.desktop_shortcut,
            Toggle::StartMenu => choices.start_menu_shortcut,
            Toggle::StartWithWindows => choices.start_with_windows,
            Toggle::AddToPath => choices.add_to_path,
            Toggle::AutoUpdate => choices.auto_update,
            Toggle::Component(index) => self
                .package
                .manifest
                .components
                .get(index)
                .is_some_and(|component| choices.components.contains(&component.id)),
            Toggle::DeleteData => self.delete_data,
        }
    }

    pub fn toggle(&mut self, toggle: Toggle, cx: &mut Context<Self>) {
        let on = !self.is_on(toggle);
        let choices = &mut self.choices;
        match toggle {
            Toggle::DesktopShortcut => choices.desktop_shortcut = on,
            Toggle::StartMenu => choices.start_menu_shortcut = on,
            Toggle::StartWithWindows => choices.start_with_windows = on,
            Toggle::AddToPath => choices.add_to_path = on,
            Toggle::AutoUpdate => choices.auto_update = on,
            Toggle::Component(index) => {
                if let Some(component) = self.package.manifest.components.get(index) {
                    choices.set(&self.package.manifest, &component.id.clone(), on);
                }
            }
            Toggle::DeleteData => self.delete_data = on,
        }
        cx.notify();
    }

    /// The license is accepted: on to the choices.
    pub fn accept_license(&mut self, cx: &mut Context<Self>) {
        self.license_pending = false;
        self.stage = Stage::Ready;
        self.focused = None;
        cx.notify();
    }

    /// Opens the folder picker; the pick becomes `<folder>\<app folder>`
    /// unless it already is one.
    pub fn choose_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        if self.picker.is_some() || !self.can_change_folder() {
            return;
        }
        let title = self.texts.get(Key::choose_folder);
        let picked = crate::ui::folder_dialog::pick_folder(&title, window);
        let folder_name = self.package.manifest.install.folder.clone();
        self.picker = Some(cx.spawn(async move |this, cx| {
            let picked = picked.await;
            this.update(cx, |this, cx| {
                if let Some(folder) = picked {
                    this.dir = install_folder(folder, &folder_name);
                }
                this.picker = None;
                cx.notify();
            })
            .ok();
        }));
    }

    /// A first install may move; an update stays where it is.
    pub fn can_change_folder(&self) -> bool {
        !self.is_removing() && self.existing.is_none() && self.package.manifest.install.change_folder
    }

    /// Starts installing or removing on a worker thread.
    pub fn start(&mut self, cx: &mut Context<Self>) {
        if matches!(self.stage, Stage::Working { .. }) {
            return;
        }
        let (sender, mut events) = mpsc::unbounded::<Event>();
        let texts = self.texts.clone();
        let job = match (&self.mode, self.preview) {
            (_, true) => Job::Pretend { removing: self.is_removing() },
            (Mode::Install, false) => Job::Install(InstallPlan {
                package: self.package.clone(),
                texts: texts.clone(),
                dir: self.dir.clone(),
                choices: self.choices.clone(),
            }),
            (Mode::Remove { .. }, false) => Job::Remove(RemovePlan {
                package: self.package.clone(),
                texts: texts.clone(),
                dir: self.dir.clone(),
                delete_data: self.delete_data,
            }),
        };
        std::thread::spawn(move || {
            let last = std::cell::RefCell::new(String::new());
            let report = |progress: f32, label: &str| {
                if *last.borrow() != label {
                    crate::log::line(label);
                    *last.borrow_mut() = label.to_string();
                }
                sender.unbounded_send(Event::Progress(progress, label.to_string())).ok();
            };
            // A panic on the worker becomes a failed page, never a crash.
            let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| match &job {
                Job::Install(plan) => work::install(plan, &report),
                Job::Remove(plan) => work::remove(plan, &report),
                Job::Pretend { removing } => work::pretend(&texts, *removing, &report),
            }))
            .unwrap_or_else(|_| Err(texts.get(Key::error_damaged)));
            match &result {
                Ok(()) => crate::log::line("Finished."),
                Err(message) => crate::log::line(&format!("Failed: {message}")),
            }
            sender.unbounded_send(Event::Finished(result)).ok();
        });

        self.attempt = self.attempt.saturating_add(1);
        self.focused = None;
        self.stage = Stage::Working { progress: 0.0, label: self.texts.get(Key::step_preparing).into() };
        cx.notify();
        self.job = Some(cx.spawn(async move |this, cx| {
            while let Some(event) = events.next().await {
                match event {
                    Event::Progress(progress, label) => {
                        let alive = this.update(cx, |this, cx| {
                            this.stage = Stage::Working { progress, label: label.into() };
                            cx.notify();
                        });
                        if alive.is_err() {
                            return;
                        }
                    }
                    Event::Finished(result) => {
                        if result.is_ok() {
                            cx.background_executor().timer(FINISH_HOLD).await;
                        }
                        this.update(cx, |this, cx| {
                            this.stage = match result {
                                Ok(()) => Stage::Done,
                                Err(message) => Stage::Failed(message.into()),
                            };
                            this.focused = None;
                            this.job = None;
                            cx.notify();
                        })
                        .ok();
                        return;
                    }
                }
            }
        }));
    }

    /// Starts the installed app and closes setup.
    pub fn open_app(&mut self, cx: &mut Context<Self>) {
        if self.preview {
            cx.quit();
            return;
        }
        let exe = self.package.manifest.executable_in(&self.dir);
        match std::process::Command::new(&exe).current_dir(&self.dir).spawn() {
            Ok(_) => cx.quit(),
            Err(err) => {
                let message = self.texts.with(Key::error_open_failed, &[("error", &err.to_string())]);
                self.stage = Stage::Failed(message.into());
                cx.notify();
            }
        }
    }

    /// Closing is refused while work runs: stopping halfway is what the
    /// all-or-nothing install avoids.
    pub fn close(&mut self, cx: &mut Context<Self>) {
        if !matches!(self.stage, Stage::Working { .. }) {
            cx.quit();
        }
    }
}

enum Job {
    Install(InstallPlan),
    Remove(RemovePlan),
    Pretend { removing: bool },
}

/// A first install shows the license; an update only when it changed.
fn needs_license(manifest: &Manifest, previous: Option<&Manifest>, mode: &Mode, texts: &Texts) -> bool {
    let code = texts.language.code;
    let Some(license) = manifest.license(code) else {
        return false;
    };
    if matches!(mode, Mode::Remove { .. }) {
        return false;
    }
    previous.and_then(|previous| previous.license(code)) != Some(license)
}

/// Installing into "D:\Apps" means "D:\Apps\Hello"; "D:\Apps\Hello" stays.
fn install_folder(picked: PathBuf, folder_name: &str) -> PathBuf {
    let named_already = picked
        .file_name()
        .is_some_and(|name| name.to_string_lossy().eq_ignore_ascii_case(folder_name));
    if named_already {
        picked
    } else {
        picked.join(folder_name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picked_folders_get_the_app_folder() {
        assert_eq!(install_folder(PathBuf::from(r"D:\Apps"), "Hello"), PathBuf::from(r"D:\Apps\Hello"));
        assert_eq!(install_folder(PathBuf::from(r"D:\Apps\hello"), "Hello"), PathBuf::from(r"D:\Apps\hello"));
    }
}
