//! The system folder picker, parented to the app window so it stays on top
//! of it. Every "choose a folder" in the app goes through here.
//!
//! GPUI's Windows window can hand out its window handle but panics when
//! asked for its display handle (`unimplemented!()` in gpui 0.2), and rfd
//! asks for both. [`DialogParent`] answers with the real window handle and
//! the Windows display handle, which carries no data, so it never reaches
//! that panic.

use std::path::PathBuf;

use futures::future::BoxFuture;
use futures::FutureExt;
use gpui::Window;
use raw_window_handle::{
    DisplayHandle, HandleError, HasDisplayHandle, HasWindowHandle, RawWindowHandle, WindowHandle,
};

/// Opens the folder picker. Resolves to None when the user cancels.
pub fn pick_folder(title: &str, window: &Window) -> BoxFuture<'static, Option<PathBuf>> {
    dialog(title, window)
        .pick_folder()
        .map(|picked| picked.map(|handle| handle.path().to_path_buf()))
        .boxed()
}

fn dialog(title: &str, window: &Window) -> rfd::AsyncFileDialog {
    let dialog = rfd::AsyncFileDialog::new().set_title(title);
    match DialogParent::of(window) {
        Some(parent) => dialog.set_parent(&parent),
        None => dialog,
    }
}

/// A window handle copied out of the GPUI window, plus a display handle
/// that doesn't need one from GPUI.
struct DialogParent(RawWindowHandle);

impl DialogParent {
    fn of(window: &Window) -> Option<Self> {
        let handle = HasWindowHandle::window_handle(window).ok()?;
        Some(Self(handle.as_raw()))
    }
}

impl HasWindowHandle for DialogParent {
    fn window_handle(&self) -> Result<WindowHandle<'_>, HandleError> {
        // Safety: rfd reads the handle while building the dialog, during
        // the call that borrows this parent; the app window is alive then.
        Ok(unsafe { WindowHandle::borrow_raw(self.0) })
    }
}

impl HasDisplayHandle for DialogParent {
    fn display_handle(&self) -> Result<DisplayHandle<'_>, HandleError> {
        #[cfg(windows)]
        {
            Ok(DisplayHandle::windows())
        }
        #[cfg(not(windows))]
        {
            Err(HandleError::NotSupported)
        }
    }
}
