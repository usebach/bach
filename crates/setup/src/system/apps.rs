//! Other programs: closing the app (politely) so an update can continue,
//! opening a web page, and checking what the app needs before it installs.

use std::path::{Path, PathBuf};

use bach_pack::manifest::Check;
use windows::core::{HSTRING, PCWSTR, PWSTR};
use windows::Win32::Foundation::ERROR_SUCCESS;
use windows::Win32::System::RestartManager::{
    RmEndSession, RmRegisterResources, RmShutdown, RmStartSession, CCH_RM_SESSION_KEY,
};
use windows::Win32::UI::Shell::ShellExecuteW;
use windows::Win32::UI::WindowsAndMessaging::SW_SHOWNORMAL;
use windows_registry::{CURRENT_USER, LOCAL_MACHINE};

/// Asks every program using `files` to close, the way Windows Update does:
/// each gets the chance to ask about unsaved work, and nothing is killed.
/// True once they've all closed.
pub fn close_programs_using(files: &[PathBuf]) -> bool {
    let names: Vec<HSTRING> = files.iter().map(|file| HSTRING::from(file.as_os_str())).collect();
    let pointers: Vec<PCWSTR> = names.iter().map(|name| PCWSTR(name.as_ptr())).collect();
    let mut session = 0u32;
    let mut key = [0u16; CCH_RM_SESSION_KEY as usize + 1];
    // Safety: the key buffer is CCH_RM_SESSION_KEY + 1 long as documented,
    // the names outlive the calls, and the session is always ended.
    unsafe {
        if RmStartSession(&mut session, None, PWSTR(key.as_mut_ptr())) != ERROR_SUCCESS {
            return false;
        }
        let closed = RmRegisterResources(session, Some(&pointers), None, None) == ERROR_SUCCESS
            // 0: no forced shutdown; programs that refuse stay open.
            && RmShutdown(session, 0, None) == ERROR_SUCCESS;
        let _ = RmEndSession(session);
        closed
    }
}

/// Opens a web page in the user's browser (http and https only, so a
/// manifest can never start a program this way).
pub fn open_url(url: &str) {
    if !(url.starts_with("https://") || url.starts_with("http://")) {
        return;
    }
    // Safety: plain strings; the shell opens the default browser.
    unsafe {
        ShellExecuteW(None, &HSTRING::from("open"), &HSTRING::from(url), None, None, SW_SHOWNORMAL);
    }
}

/// Whether what the app needs is on this computer. Only reads: files and
/// registry keys any user may look at.
pub fn is_present(check: &Check) -> bool {
    match check {
        Check::Webview2 => {
            const CLIENT: &str = r"Microsoft\EdgeUpdate\Clients\{F3017226-FE2A-4295-8BDF-00C3A9A7E4C5}";
            [format!(r"SOFTWARE\WOW6432Node\{CLIENT}"), format!(r"SOFTWARE\{CLIENT}")]
                .iter()
                .any(|key| has_version(LOCAL_MACHINE, key))
                || has_version(CURRENT_USER, &format!(r"Software\{CLIENT}"))
        }
        Check::Vcredist => LOCAL_MACHINE
            .open(r"SOFTWARE\Microsoft\VisualStudio\14.0\VC\Runtimes\x64")
            .and_then(|key| key.get_u32("Installed"))
            .is_ok_and(|installed| installed == 1),
        Check::Dotnet { version, desktop } => {
            let runtime = if *desktop { "Microsoft.WindowsDesktop.App" } else { "Microsoft.NETCore.App" };
            let roots = [std::env::var_os("ProgramFiles"), std::env::var_os("ProgramW6432")];
            roots.into_iter().flatten().any(|root| {
                let shared = Path::new(&root).join("dotnet").join("shared").join(runtime);
                std::fs::read_dir(shared).is_ok_and(|entries| {
                    entries.flatten().any(|entry| {
                        let name = entry.file_name().to_string_lossy().to_string();
                        name == *version || name.starts_with(&format!("{version}."))
                    })
                })
            })
        }
        Check::File { path } => expand(path).is_some_and(|path| path.exists()),
        Check::Registry { key } => {
            let (hive, rest) = key.split_once('\\').unwrap_or((key, ""));
            match hive.to_ascii_uppercase().as_str() {
                "HKLM" => LOCAL_MACHINE.open(rest).is_ok(),
                "HKCU" => CURRENT_USER.open(rest).is_ok(),
                _ => false,
            }
        }
    }
}

fn has_version(hive: &windows_registry::Key, key: &str) -> bool {
    hive.open(key)
        .and_then(|key| key.get_string("pv"))
        .is_ok_and(|version| !version.is_empty() && version != "0.0.0.0")
}

/// `%SystemRoot%\System32\x.dll` with the environment filled in. None when
/// a variable isn't set.
fn expand(template: &str) -> Option<PathBuf> {
    let mut out = String::new();
    let mut rest = template;
    while let Some(start) = rest.find('%') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let end = after.find('%')?;
        out.push_str(&std::env::var(&after[..end]).ok()?);
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    Some(PathBuf::from(out))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checks_look_without_failing() {
        assert!(is_present(&Check::File { path: r"%SystemRoot%\System32\kernel32.dll".into() }));
        assert!(!is_present(&Check::File { path: r"%SystemRoot%\no-such-file.dll".into() }));
        assert!(!is_present(&Check::File { path: r"%NO_SUCH_VARIABLE%\x".into() }));
        assert!(is_present(&Check::Registry { key: r"HKCU\Software".into() }));
        assert!(!is_present(&Check::Registry { key: r"HKCU\Software\No Such Key Bach".into() }));
        // These depend on the machine; they must answer, not crash.
        let _ = is_present(&Check::Webview2);
        let _ = is_present(&Check::Vcredist);
        let _ = is_present(&Check::Dotnet { version: "8.0".into(), desktop: false });
    }

    #[test]
    fn closing_nothing_succeeds() {
        let missing = std::env::temp_dir().join("bach-no-such-program.exe");
        assert!(close_programs_using(&[missing]));
    }
}

#[cfg(test)]
mod closing {
    use super::*;

    /// Starts the example app (a message box) and asks it to close the way
    /// "Close and continue" does. Needs `cargo build -p hello` first:
    /// `cargo test -p bach-setup -- --ignored closes_a_running_app`.
    #[test]
    #[ignore]
    fn closes_a_running_app() {
        let built = std::env::current_exe().unwrap().parent().unwrap().parent().unwrap().join("hello.exe");
        let copy = std::env::temp_dir().join(format!("bach-close-{}.exe", std::process::id()));
        std::fs::copy(&built, &copy).expect("build the hello example first");
        let mut app = std::process::Command::new(&copy).spawn().unwrap();
        std::thread::sleep(std::time::Duration::from_millis(1500));
        assert!(close_programs_using(std::slice::from_ref(&copy)), "the app should agree to close");
        let mut exited = false;
        for _ in 0..50 {
            if app.try_wait().unwrap().is_some() {
                exited = true;
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        if !exited {
            let _ = app.kill();
        }
        let _ = std::fs::remove_file(&copy);
        assert!(exited, "the app is still running");
    }
}
