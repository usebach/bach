//! The setup window's state: which page it's on, the choices on the first
//! page, and the running job. Drawing lives in `pages.rs`.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use futures::channel::mpsc;
use futures::StreamExt;
use gpui::{Context, SharedString, Task, Window};

use crate::launch::Mode;
use crate::package::Package;
use crate::system::{self, Installed};
use crate::work::{self, Event, InstallPlan, RemovePlan};

/// The bar gets this long to glide to full before the last page shows.
const FINISH_HOLD: Duration = Duration::from_millis(650);

#[derive(Clone, Debug)]
pub enum Stage {
    /// The first page: options and the go button.
    Ready,
    Working {
        progress: f32,
        label: SharedString,
    },
    Done,
    Failed(SharedString),
}

pub struct SetupView {
    pub package: Arc<Package>,
    pub mode: Mode,
    /// Pretend: nothing is installed or removed.
    pub preview: bool,
    /// An install already on this computer (install mode only).
    pub existing: Option<Installed>,
    /// Where the app goes (or is, when removing).
    pub dir: PathBuf,
    pub desktop_shortcut: bool,
    pub delete_data: bool,
    pub stage: Stage,
    /// Counts attempts, so each one's page and bar start fresh.
    pub attempt: usize,
    job: Option<Task<()>>,
    picker: Option<Task<()>>,
}

impl SetupView {
    pub fn new(package: Arc<Package>, mode: Mode, preview: bool) -> Self {
        let manifest = &package.manifest;
        let (existing, dir) = match &mode {
            Mode::Install => {
                let existing = system::installed(manifest);
                let dir = existing
                    .as_ref()
                    .map(|installed| installed.dir.clone())
                    .unwrap_or_else(|| system::default_install_dir(&manifest.install.folder));
                (existing, dir)
            }
            Mode::Remove { dir } => (None, dir.clone()),
        };
        // An update keeps the desktop shortcut as it was; a first install
        // offers the config's default.
        let desktop_shortcut = match &existing {
            Some(_) => system::desktop_shortcut(manifest).is_ok_and(|path| path.exists()),
            None => manifest.install.desktop_shortcut,
        };
        Self {
            package,
            mode,
            preview,
            existing,
            dir,
            desktop_shortcut,
            delete_data: false,
            stage: Stage::Ready,
            attempt: 0,
            job: None,
            picker: None,
        }
    }

    pub fn is_removing(&self) -> bool {
        matches!(self.mode, Mode::Remove { .. })
    }

    pub fn toggle_desktop_shortcut(&mut self, cx: &mut Context<Self>) {
        self.desktop_shortcut = !self.desktop_shortcut;
        cx.notify();
    }

    pub fn toggle_delete_data(&mut self, cx: &mut Context<Self>) {
        self.delete_data = !self.delete_data;
        cx.notify();
    }

    /// Opens the folder picker; the pick becomes `<folder>\<app folder>`
    /// unless it already is one.
    pub fn choose_folder(&mut self, window: &mut Window, cx: &mut Context<Self>) {
        let title = format!("Choose where to install {}", self.package.name());
        let picked = crate::ui::folder_dialog::pick_folder(&title, window);
        let folder_name = self.package.manifest.install.folder.clone();
        self.picker = Some(cx.spawn(async move |this, cx| {
            let Some(folder) = picked.await else {
                return;
            };
            this.update(cx, |this, cx| {
                this.dir = install_folder(folder, &folder_name);
                this.picker = None;
                cx.notify();
            })
            .ok();
        }));
    }

    /// Starts installing or removing on a worker thread.
    pub fn start(&mut self, cx: &mut Context<Self>) {
        if matches!(self.stage, Stage::Working { .. }) {
            return;
        }
        let (sender, mut events) = mpsc::unbounded::<Event>();
        let job = match (&self.mode, self.preview) {
            (_, true) => Job::Pretend {
                name: self.package.name().to_string(),
                removing: self.is_removing(),
            },
            (Mode::Install, false) => Job::Install(InstallPlan {
                package: self.package.clone(),
                dir: self.dir.clone(),
                desktop_shortcut: self.desktop_shortcut,
            }),
            (Mode::Remove { .. }, false) => Job::Remove(RemovePlan {
                package: self.package.clone(),
                dir: self.dir.clone(),
                delete_data: self.delete_data,
            }),
        };
        std::thread::spawn(move || {
            let report = |progress: f32, label: &str| {
                sender.unbounded_send(Event::Progress(progress, label.to_string())).ok();
            };
            let result = match &job {
                Job::Install(plan) => work::install(plan, &report),
                Job::Remove(plan) => work::remove(plan, &report),
                Job::Pretend { name, removing } => work::pretend(name, *removing, &report),
            };
            sender.unbounded_send(Event::Finished(result)).ok();
        });

        self.attempt = self.attempt.saturating_add(1);
        self.stage = Stage::Working {
            progress: 0.0,
            label: "Getting ready".into(),
        };
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
                let name = self.package.name();
                self.stage = Stage::Failed(format!("{name} is installed, but it didn't start: {err}").into());
                cx.notify();
            }
        }
    }

    pub fn close(&mut self, cx: &mut Context<Self>) {
        cx.quit();
    }
}

enum Job {
    Install(InstallPlan),
    Remove(RemovePlan),
    Pretend { name: String, removing: bool },
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
