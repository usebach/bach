//! `--auto-update`: one background round, run by the app's scheduled task
//! (see `system/task.rs`).
//!
//! It asks the feed, downloads a newer version into the staging folder, and
//! installs it when the app isn't running. When the app is open it leaves
//! the download ready and tries again at the next round, so an update never
//! closes a program somebody is working in. A round that finds nothing to
//! do ends quietly and quickly.

use super::{apply, feed, stage, Context, UPDATE_AVAILABLE};
use crate::system::{self, host, task};
use crate::work;

pub fn run(ctx: &Context) -> Result<i32, String> {
    let manifest = ctx.manifest();
    let app_id = &manifest.app.id;
    let Some(_lock) = host::SetupLock::acquire_updater(app_id) else {
        crate::log::line("Another update round is running.");
        return Ok(0);
    };
    // The app was removed (or moved) and the task outlived it.
    let Some(installed) = system::installed(manifest) else {
        crate::log::line("The app isn't installed any more; removing its update task.");
        let _ = task::unregister(manifest);
        stage::clear(app_id);
        return Ok(0);
    };
    let program = manifest.executable_in(&installed.dir);
    let running = work::is_running(&program);

    // An update that was cut short (a power cut) is undone while the app
    // is closed, so it never starts from half its files.
    if !running {
        if let Some(_setup) = host::SetupLock::acquire(app_id) {
            let undone = work::recover_interrupted(&installed.dir);
            if undone > 0 {
                crate::log::line(&format!("Put back {undone} file(s) of an update that was cut short."));
            }
        }
    }

    let entry = ctx.latest()?;
    ctx.tidy();
    if !feed::is_newer(&entry.version, &installed.version) {
        crate::log::line("Up to date.");
        return Ok(0);
    }
    let staged = super::stage_entry(ctx, &entry)?;
    if running {
        crate::log::line(&format!("{} {} is ready; it installs once the app is closed.", manifest.app.name, staged.version));
        return Ok(UPDATE_AVAILABLE);
    }
    apply::install(ctx, &staged)
}
