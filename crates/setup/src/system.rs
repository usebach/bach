//! What setup asks of Windows: where per-user programs and shortcuts live,
//! `.lnk` shortcuts, and the entry under Settings › Apps that lists the
//! app and starts its uninstaller. Everything here is per user (HKCU, the
//! user's own folders), so setup never needs administrator rights.
//!
//! These calls block; setup runs them on its worker thread, apart from the
//! quick registry read at startup.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use bach_pack::manifest::{Manifest, UNINSTALLER};
use windows::core::{Interface, GUID, HSTRING};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CoUninitialize, IPersistFile,
    CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED,
};
use windows::Win32::UI::Shell::{
    FOLDERID_Desktop, FOLDERID_Programs, FOLDERID_UserProgramFiles, IShellLinkW, SHChangeNotify,
    SHGetKnownFolderPath, ShellLink, KF_FLAG_DEFAULT, SHCNE_ASSOCCHANGED, SHCNF_IDLIST,
};
use windows_registry::CURRENT_USER;

/// An install found in the registry.
#[derive(Clone, Debug)]
pub struct Installed {
    pub dir: PathBuf,
    pub version: String,
}

fn uninstall_key(manifest: &Manifest) -> String {
    format!(r"Software\Microsoft\Windows\CurrentVersion\Uninstall\{}", manifest.app.id)
}

/// The install Settings › Apps lists for this app, if any, and only while
/// its program is still there.
pub fn installed(manifest: &Manifest) -> Option<Installed> {
    let key = CURRENT_USER.open(uninstall_key(manifest)).ok()?;
    let dir = PathBuf::from(key.get_string("InstallLocation").ok()?);
    let version = key.get_string("DisplayVersion").unwrap_or_default();
    manifest
        .executable_in(&dir)
        .is_file()
        .then_some(Installed { dir, version })
}

/// `%LOCALAPPDATA%\Programs\<folder>`, where per-user apps go.
pub fn default_install_dir(folder: &str) -> PathBuf {
    known_folder(&FOLDERID_UserProgramFiles)
        .or_else(|_| {
            std::env::var_os("LOCALAPPDATA")
                .map(|local| PathBuf::from(local).join("Programs"))
                .ok_or_else(|| anyhow!("LOCALAPPDATA isn't set"))
        })
        .map(|programs| programs.join(folder))
        .unwrap_or_else(|_| PathBuf::from(r"C:\").join(folder))
}

/// The shortcut's file name: the app's name, minus anything Windows
/// doesn't allow in a file name.
fn shortcut_name(manifest: &Manifest) -> String {
    let name: String = manifest
        .app
        .name
        .chars()
        .map(|c| if r#"<>:"/\|?*"#.contains(c) || c.is_control() { ' ' } else { c })
        .collect();
    let name = name.trim().trim_end_matches('.');
    let name = if name.is_empty() { manifest.app.id.as_str() } else { name };
    format!("{name}.lnk")
}

/// Where the Start menu shortcut goes.
pub fn start_menu_shortcut(manifest: &Manifest) -> Result<PathBuf> {
    Ok(known_folder(&FOLDERID_Programs)?.join(shortcut_name(manifest)))
}

/// Where the desktop shortcut goes (the desktop may live in OneDrive).
pub fn desktop_shortcut(manifest: &Manifest) -> Result<PathBuf> {
    Ok(known_folder(&FOLDERID_Desktop)?.join(shortcut_name(manifest)))
}

fn known_folder(id: &GUID) -> Result<PathBuf> {
    // Safety: the returned buffer is ours to free, which we do once it's
    // copied into a PathBuf.
    unsafe {
        let raw = SHGetKnownFolderPath(id, KF_FLAG_DEFAULT, None)
            .context("Windows didn't say where that folder is")?;
        let path = raw.to_string().map(PathBuf::from);
        CoTaskMemFree(Some(raw.0 as *const _));
        path.context("Windows returned a folder name that isn't valid text")
    }
}

/// Writes a `.lnk` at `link` that starts `target` in its own folder.
pub fn create_shortcut(link: &Path, target: &Path, description: &str) -> Result<()> {
    if let Some(parent) = link.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let _com = ComThread::enter();
    // Safety: plain COM calls on an interface we own, on a thread with COM
    // initialised for as long as `_com` lives.
    unsafe {
        let shell_link: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
        shell_link.SetPath(&HSTRING::from(target.as_os_str()))?;
        if let Some(dir) = target.parent() {
            shell_link.SetWorkingDirectory(&HSTRING::from(dir.as_os_str()))?;
        }
        shell_link.SetDescription(&HSTRING::from(description))?;
        shell_link.SetIconLocation(&HSTRING::from(target.as_os_str()), 0)?;
        let file: IPersistFile = shell_link.cast()?;
        file.Save(&HSTRING::from(link.as_os_str()), true)?;
    }
    Ok(())
}

/// Lists the app under Settings › Apps, with its uninstaller.
pub fn register(manifest: &Manifest, dir: &Path) -> Result<()> {
    let app = &manifest.app;
    let exe = manifest.executable_in(dir);
    let uninstaller = dir.join(UNINSTALLER);
    let key = CURRENT_USER.create(uninstall_key(manifest))?;
    key.set_string("DisplayName", &app.name)?;
    key.set_string("DisplayVersion", &app.version)?;
    key.set_string("Publisher", &app.publisher)?;
    key.set_string("DisplayIcon", format!("{},0", exe.display()))?;
    key.set_string("InstallLocation", dir.to_string_lossy())?;
    key.set_string("UninstallString", format!("\"{}\"", uninstaller.display()))?;
    if let Some(website) = &app.website {
        key.set_string("URLInfoAbout", website)?;
    }
    let size_kb = manifest.total_size() / 1024;
    key.set_u32("EstimatedSize", u32::try_from(size_kb).unwrap_or(u32::MAX))?;
    key.set_u32("NoModify", 1)?;
    key.set_u32("NoRepair", 1)?;
    Ok(())
}

/// Takes the app off Settings › Apps.
pub fn unregister(manifest: &Manifest) -> Result<()> {
    let key = uninstall_key(manifest);
    match CURRENT_USER.open(&key) {
        Ok(_) => Ok(CURRENT_USER.remove_tree(&key)?),
        // Nothing registered: nothing to remove.
        Err(_) => Ok(()),
    }
}

/// Tells Explorer that shortcuts and icons changed, so the Start menu and
/// the desktop pick them up straight away.
pub fn notify_shell() {
    // Safety: no pointers passed; the flags are the documented pair.
    unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None) };
}

/// COM for the current thread, released when dropped.
struct ComThread {
    owned: bool,
}

impl ComThread {
    fn enter() -> Self {
        // Safety: paired with CoUninitialize in Drop when it succeeded.
        let result = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        Self { owned: result.is_ok() }
    }
}

impl Drop for ComThread {
    fn drop(&mut self) {
        if self.owned {
            // Safety: this thread's own successful CoInitializeEx.
            unsafe { CoUninitialize() };
        }
    }
}
