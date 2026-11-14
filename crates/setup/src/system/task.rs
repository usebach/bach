//! The scheduled task that keeps an app up to date in the background.
//!
//! One per-user task, `Bach\<app id>`, runs `uninstall.exe --auto-update`
//! every few hours (and as soon as possible after a time it missed, because
//! the PC was off). It needs no administrator rights, runs hidden, and is
//! taken away again when the user unticks "Automatic updates", and when the
//! app is removed. Task Scheduler is driven through `schtasks.exe`, which
//! every Windows has.

use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use bach_pack::manifest::Manifest;

/// How often the task runs, as an ISO 8601 duration.
const EVERY: &str = "PT4H";
/// Machines don't all ask the update server at the same moment.
const SPREAD: &str = "PT20M";

fn task_name(manifest: &Manifest) -> String {
    format!(r"Bach\{}", manifest.app.id)
}

fn schtasks() -> PathBuf {
    let system = std::env::var_os("SystemRoot").map(PathBuf::from).unwrap_or_else(|| PathBuf::from(r"C:\Windows"));
    system.join("System32").join("schtasks.exe")
}

fn run(args: &[&std::ffi::OsStr]) -> Result<std::process::Output> {
    use std::os::windows::process::CommandExt;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    Command::new(schtasks())
        .args(args)
        .creation_flags(CREATE_NO_WINDOW)
        .output()
        .context("couldn't run Task Scheduler's schtasks.exe")
}

fn escape(text: &str) -> String {
    text.replace('&', "&amp;").replace('<', "&lt;").replace('>', "&gt;").replace('"', "&quot;")
}

/// The task definition: every [`EVERY`], started late rather than never,
/// allowed on battery, hidden, one copy at a time, and stopped after two
/// hours so a stuck download can't run forever.
fn definition(manifest: &Manifest, uninstaller: &Path) -> String {
    let description = escape(&format!("Keeps {} up to date.", manifest.app.name));
    let command = escape(&uninstaller.display().to_string());
    format!(
        r#"<?xml version="1.0" encoding="UTF-16"?>
<Task version="1.2" xmlns="http://schemas.microsoft.com/windows/2004/02/mit/task">
  <RegistrationInfo>
    <Author>Bach</Author>
    <Description>{description}</Description>
  </RegistrationInfo>
  <Triggers>
    <TimeTrigger>
      <Repetition>
        <Interval>{EVERY}</Interval>
        <StopAtDurationEnd>false</StopAtDurationEnd>
      </Repetition>
      <StartBoundary>2020-01-01T00:00:00</StartBoundary>
      <RandomDelay>{SPREAD}</RandomDelay>
      <Enabled>true</Enabled>
    </TimeTrigger>
  </Triggers>
  <Principals>
    <Principal id="Author">
      <LogonType>InteractiveToken</LogonType>
      <RunLevel>LeastPrivilege</RunLevel>
    </Principal>
  </Principals>
  <Settings>
    <MultipleInstancesPolicy>IgnoreNew</MultipleInstancesPolicy>
    <DisallowStartIfOnBatteries>false</DisallowStartIfOnBatteries>
    <StopIfGoingOnBatteries>false</StopIfGoingOnBatteries>
    <StartWhenAvailable>true</StartWhenAvailable>
    <ExecutionTimeLimit>PT2H</ExecutionTimeLimit>
    <Hidden>true</Hidden>
    <Enabled>true</Enabled>
  </Settings>
  <Actions Context="Author">
    <Exec>
      <Command>{command}</Command>
      <Arguments>--auto-update</Arguments>
    </Exec>
  </Actions>
</Task>
"#
    )
}

/// Creates (or replaces) the app's update task, running `uninstaller`.
pub fn register(manifest: &Manifest, uninstaller: &Path) -> Result<()> {
    let file = std::env::temp_dir().join(format!("bach-task-{}-{}.xml", manifest.app.id.replace(' ', "-"), std::process::id()));
    // Task Scheduler reads its XML as UTF-16 with a byte order mark.
    let mut bytes = vec![0xFF, 0xFE];
    for unit in definition(manifest, uninstaller).encode_utf16() {
        bytes.extend(unit.to_le_bytes());
    }
    std::fs::write(&file, bytes).context("couldn't write the update task")?;
    let name = task_name(manifest);
    let result = run(&["/Create".as_ref(), "/TN".as_ref(), name.as_ref(), "/XML".as_ref(), file.as_os_str(), "/F".as_ref()]);
    let _ = std::fs::remove_file(&file);
    let output = result?;
    if !output.status.success() {
        bail!("Task Scheduler refused the update task: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(())
}

/// Deletes the app's update task. Fine when there is none.
pub fn unregister(manifest: &Manifest) -> Result<()> {
    let name = task_name(manifest);
    if !exists(manifest) {
        return Ok(());
    }
    let output = run(&["/Delete".as_ref(), "/TN".as_ref(), name.as_ref(), "/F".as_ref()])?;
    if !output.status.success() {
        bail!("couldn't remove the update task: {}", String::from_utf8_lossy(&output.stderr).trim());
    }
    Ok(())
}

/// Whether the app's update task is there.
pub fn exists(manifest: &Manifest) -> bool {
    let name = task_name(manifest);
    run(&["/Query".as_ref(), "/TN".as_ref(), name.as_ref()]).is_ok_and(|output| output.status.success())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(id: &str, name: &str) -> Manifest {
        let json = format!(
            r#"{{"format":1,"builder":"test",
            "app":{{"id":"{id}","name":"{name}","version":"1","publisher":"Bach",
                   "description":null,"website":null,"executable":"t.exe"}},
            "install":{{"folder":"{id}","desktop_shortcut":false}},
            "uninstall":{{"data":[],"data_description":null}},
            "look":{{"theme":"system","accent":null,"logo":null}},
            "files":[{{"path":"t.exe","size":1}}]}}"#
        );
        serde_json::from_str(&json).unwrap()
    }

    #[test]
    fn the_definition_is_escaped_xml() {
        let xml = definition(&manifest("Tom", "Tom & <Jerry>"), Path::new(r"C:\Tom's Apps\uninstall.exe"));
        assert!(xml.contains("Tom &amp; &lt;Jerry&gt;"), "{xml}");
        assert!(xml.contains(r"<Command>C:\Tom's Apps\uninstall.exe</Command>"));
        assert!(xml.contains("<Arguments>--auto-update</Arguments>"));
        assert!(xml.contains("<StartWhenAvailable>true</StartWhenAvailable>"));
        assert!(!xml.contains("LogonTrigger"), "a logon trigger needs administrator rights");
    }

    /// Creates, finds and deletes a real per-user task with a throwaway name.
    #[test]
    fn a_task_comes_and_goes() {
        let app = manifest(&format!("BachTestTask{}", std::process::id()), "Bach Test Task");
        let program = Path::new(r"C:\Windows\System32\whoami.exe");
        register(&app, program).expect("registers without administrator rights");
        assert!(exists(&app));
        register(&app, program).expect("registering again replaces it");
        unregister(&app).unwrap();
        assert!(!exists(&app));
        unregister(&app).expect("removing what isn't there is fine");
    }
}
