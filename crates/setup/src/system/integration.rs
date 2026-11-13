//! Starting with Windows, link schemes and file types, all per user.
//!
//! Every key setup creates here carries a `BachApp` value with the app id,
//! and setup only ever changes or deletes keys with its own mark. A scheme
//! or file type another program already owns is left as it is, so
//! installing an app never takes links or files away from another one.

use std::path::Path;

use anyhow::{Context, Result};
use bach_pack::manifest::{FileType, Manifest, Protocol};
use windows_registry::{CLASSES_ROOT, CURRENT_USER};

const RUN: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";
const CLASSES: &str = r"Software\Classes";
const MARK: &str = "BachApp";

fn quoted(exe: &Path) -> String {
    format!("\"{}\"", exe.display())
}

fn open_command(exe: &Path) -> String {
    format!("\"{}\" \"%1\"", exe.display())
}

/// Starts the app when the user signs in.
pub fn set_autostart(manifest: &Manifest, exe: &Path) -> Result<()> {
    let key = CURRENT_USER.create(RUN).context("couldn't open the sign-in programs")?;
    key.set_string(&manifest.app.id, quoted(exe)).context("couldn't add the app to the sign-in programs")?;
    Ok(())
}

/// Stops starting the app at sign-in, if setup set it up for this install.
pub fn clear_autostart(manifest: &Manifest, exe: &Path) -> Result<()> {
    let Ok(key) = CURRENT_USER.open(RUN) else {
        return Ok(());
    };
    let Ok(command) = key.get_string(&manifest.app.id) else {
        return Ok(());
    };
    // Only our own entry: the same name could belong to something else.
    if command.to_lowercase().contains(&exe.display().to_string().to_lowercase()) {
        CURRENT_USER
            .options()
            .read()
            .write()
            .open(RUN)
            .and_then(|key| key.remove_value(&manifest.app.id))
            .context("couldn't remove the app from the sign-in programs")?;
    }
    Ok(())
}

/// Whether the per-user key `name` under Classes is ours, someone else's,
/// or free (and no machine-wide one exists either).
enum Owner {
    Us,
    Other,
    Free,
}

fn owner(manifest: &Manifest, name: &str) -> Owner {
    match CURRENT_USER.open(format!(r"{CLASSES}\{name}")) {
        Ok(key) => match key.get_string(MARK) {
            Ok(id) if id == manifest.app.id => Owner::Us,
            _ => Owner::Other,
        },
        Err(_) if CLASSES_ROOT.open(name).is_ok() => Owner::Other,
        Err(_) => Owner::Free,
    }
}

/// Registers `scheme://` to start the app. False when another program
/// owns the scheme, which is left alone.
pub fn register_protocol(manifest: &Manifest, protocol: &Protocol, exe: &Path) -> Result<bool> {
    if matches!(owner(manifest, &protocol.scheme), Owner::Other) {
        return Ok(false);
    }
    let description = protocol.description.clone().unwrap_or_else(|| format!("URL:{}", manifest.app.name));
    let key = CURRENT_USER.create(format!(r"{CLASSES}\{}", protocol.scheme))?;
    key.set_string(MARK, &manifest.app.id)?;
    key.set_string("", description)?;
    key.set_string("URL Protocol", "")?;
    key.create("DefaultIcon")?.set_string("", format!("{},0", exe.display()))?;
    key.create(r"shell\open\command")?.set_string("", open_command(exe))?;
    Ok(true)
}

pub fn unregister_protocol(manifest: &Manifest, protocol: &Protocol) -> Result<()> {
    if matches!(owner(manifest, &protocol.scheme), Owner::Us) {
        // With a subkey named, RegDeleteTree deletes that key itself too.
        CURRENT_USER.remove_tree(format!(r"{CLASSES}\{}", protocol.scheme))?;
    }
    Ok(())
}

/// Offers the app under "Open with" for the extension, and makes it the
/// default only when no program opens that type yet.
pub fn register_file_type(manifest: &Manifest, file_type: &FileType, exe: &Path) -> Result<()> {
    let prog_id = file_type.prog_id(&manifest.app.id);
    if matches!(owner(manifest, &prog_id), Owner::Other) {
        return Ok(());
    }
    let description = file_type.description.clone().unwrap_or_else(|| format!("{} file", manifest.app.name));
    let key = CURRENT_USER.create(format!(r"{CLASSES}\{prog_id}"))?;
    key.set_string(MARK, &manifest.app.id)?;
    key.set_string("", description)?;
    key.create("DefaultIcon")?.set_string("", format!("{},0", exe.display()))?;
    key.create(r"shell\open\command")?.set_string("", open_command(exe))?;

    let extension = CURRENT_USER.create(format!(r"{CLASSES}\{}", file_type.extension))?;
    extension.create("OpenWithProgids")?.set_string(&prog_id, "")?;
    let claimed = CLASSES_ROOT
        .open(&file_type.extension)
        .and_then(|key| key.get_string(""))
        .is_ok_and(|default| !default.is_empty() && default != prog_id);
    if !claimed {
        extension.set_string("", &prog_id)?;
    }
    Ok(())
}

pub fn unregister_file_type(manifest: &Manifest, file_type: &FileType) -> Result<()> {
    let prog_id = file_type.prog_id(&manifest.app.id);
    if let Ok(extension) = CURRENT_USER.options().read().write().open(format!(r"{CLASSES}\{}", file_type.extension)) {
        if extension.get_string("").is_ok_and(|default| default == prog_id) {
            let _ = extension.remove_value("");
        }
        if let Ok(open_with) = extension.options().read().write().open("OpenWithProgids") {
            let _ = open_with.remove_value(&prog_id);
        }
    }
    if matches!(owner(manifest, &prog_id), Owner::Us) {
        CURRENT_USER.remove_tree(format!(r"{CLASSES}\{prog_id}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest(id: &str) -> Manifest {
        let json = format!(
            r#"{{"format":1,"builder":"test",
            "app":{{"id":"{id}","name":"Bach Test","version":"1","publisher":"Bach",
                   "description":null,"website":null,"executable":"t.exe"}},
            "install":{{"folder":"{id}","desktop_shortcut":false}},
            "uninstall":{{"data":[],"data_description":null}},
            "look":{{"theme":"system","accent":null,"logo":null}},
            "files":[{{"path":"t.exe","size":1}}]}}"#
        );
        serde_json::from_str(&json).unwrap()
    }

    /// Registers a throwaway scheme and file type for the current user and
    /// removes them again, checking setup never touches another app's key.
    #[test]
    fn schemes_and_file_types_come_and_go() {
        let tag = std::process::id();
        let ours = manifest(&format!("BachTest{tag}"));
        let theirs = manifest(&format!("Other{tag}"));
        let exe = Path::new(r"C:\Bach Test\t.exe");
        let protocol = Protocol { scheme: format!("bach-test-{tag}"), description: None };
        let file_type = FileType { extension: format!(".bt{tag}"), description: None };
        let key = |name: &str| CURRENT_USER.open(format!(r"{CLASSES}\{name}"));

        assert!(register_protocol(&ours, &protocol, exe).unwrap());
        assert_eq!(key(&protocol.scheme).unwrap().get_string(MARK).unwrap(), ours.app.id);
        assert!(!register_protocol(&theirs, &protocol, exe).unwrap(), "another app's scheme stays");
        unregister_protocol(&theirs, &protocol).unwrap();
        assert!(key(&protocol.scheme).is_ok(), "only the owner removes it");
        unregister_protocol(&ours, &protocol).unwrap();
        assert!(key(&protocol.scheme).is_err());

        register_file_type(&ours, &file_type, exe).unwrap();
        let prog_id = file_type.prog_id(&ours.app.id);
        assert_eq!(key(&file_type.extension).unwrap().get_string("").unwrap(), prog_id);
        unregister_file_type(&ours, &file_type).unwrap();
        assert!(key(&prog_id).is_err());
        assert!(key(&file_type.extension).unwrap().get_string("").is_err());
        let _ = CURRENT_USER.remove_tree(format!(r"{CLASSES}\{}", file_type.extension));
    }

    #[test]
    fn autostart_removes_only_its_own_entry() {
        let ours = manifest(&format!("BachTestRun{}", std::process::id()));
        let exe = Path::new(r"C:\Bach Test\t.exe");
        set_autostart(&ours, exe).unwrap();
        clear_autostart(&ours, Path::new(r"C:\Elsewhere\t.exe")).unwrap();
        assert!(CURRENT_USER.open(RUN).unwrap().get_string(&ours.app.id).is_ok(), "not ours, so it stays");
        clear_autostart(&ours, exe).unwrap();
        assert!(CURRENT_USER.open(RUN).unwrap().get_string(&ours.app.id).is_err());
    }
}
