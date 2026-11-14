//! Installing a staged update: wait for the app to close, run the setup file
//! without a window, and start the app again if asked.
//!
//! The setup file does the real work (all or nothing, see `work/place.rs`),
//! so this only has to run it, and to cope with a world where the app was
//! closed a moment ago and a virus scanner still holds one of its files: a
//! failed run is tried again a couple of times before it is given up on, and
//! the staged update is kept for the next round.

use std::path::Path;
use std::process::Command;
use std::time::Duration;

use bach_pack::container;

use super::stage::{self, Staged};
use super::Context;
use crate::system::host;

/// How long `--wait-for` lets the app take to close.
const WAIT_FOR_APP: Duration = Duration::from_secs(10 * 60);
/// Tries at running the setup file, and the pause between them.
const ATTEMPTS: usize = 3;
const PAUSE: Duration = Duration::from_secs(5);

/// Waits for the process that asked for this update (the app) to end.
pub fn wait_for_app(ctx: &Context) -> Result<(), String> {
    match ctx.args.wait_for {
        Some(pid) if !host::wait_for_exit(pid, WAIT_FOR_APP) => {
            Err(format!("{} is still running, so it can't be updated yet", ctx.manifest().app.name))
        }
        _ => Ok(()),
    }
}

/// Checks the staged file once more, then installs it silently. Ok(0) when
/// it is installed, Ok(1) when it couldn't be (it stays staged to try
/// again), Err for a staged file that must not run (it is deleted).
pub fn install(ctx: &Context, staged: &Staged) -> Result<i32, String> {
    let app_id = &ctx.manifest().app.id;
    if let Err(problem) = stage::verify(staged, app_id) {
        stage::clear(app_id);
        return Err(problem);
    }
    let Some(_lock) = host::SetupLock::acquire_updater(app_id) else {
        crate::log::line("Another update of this app is running; leaving this one for later.");
        return Ok(0);
    };
    for attempt in 1..=ATTEMPTS {
        crate::log::line(&format!("Installing {} {} (try {attempt})", ctx.manifest().app.name, staged.version));
        let code = run_setup(ctx, &staged.file)?;
        if code == 0 {
            stage::clear(app_id);
            if ctx.args.launch {
                launch_app(ctx);
            }
            return Ok(0);
        }
        crate::log::line(&format!("The setup file ended with code {code}."));
        if attempt < ATTEMPTS {
            std::thread::sleep(PAUSE);
        }
    }
    eprintln!(
        "{} {} couldn't be installed yet. It stays downloaded, and the next update run tries again.",
        ctx.manifest().app.name,
        staged.version
    );
    Ok(1)
}

/// Runs a setup file with `--silent` and gives back its exit code.
fn run_setup(ctx: &Context, setup: &Path) -> Result<i32, String> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    let mut command = Command::new(setup);
    command.arg("--silent");
    if let Some(lang) = &ctx.args.lang {
        command.args(["--lang", lang]);
    }
    let status = command
        .creation_flags(CREATE_NO_WINDOW)
        .status()
        .map_err(|err| format!("couldn't start the update: {err}"))?;
    Ok(status.code().unwrap_or(1))
}

/// Starts the app that was just updated, detached from this process. It
/// reads the program's name from the new uninstaller, since a new version
/// may have renamed it.
fn launch_app(ctx: &Context) {
    use std::os::windows::process::CommandExt;
    const DETACHED_PROCESS: u32 = 0x0000_0008;
    const CREATE_NEW_PROCESS_GROUP: u32 = 0x0000_0200;
    let dir = &ctx.dir;
    let installed = std::fs::File::open(dir.join(bach_pack::manifest::UNINSTALLER))
        .ok()
        .and_then(|mut file| {
            let layout = container::locate(&mut file).ok()??;
            container::read_manifest(&mut file, &layout).ok()
        })
        .unwrap_or_else(|| ctx.manifest().clone());
    let program = installed.executable_in(dir);
    let started = Command::new(&program)
        .current_dir(dir)
        .creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP)
        .spawn();
    match started {
        Ok(_) => crate::log::line(&format!("Started {}", program.display())),
        Err(err) => crate::log::line(&format!("Couldn't start {}: {err}", program.display())),
    }
}
