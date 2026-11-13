//! The OS "reduce motion" setting. On Windows that's "Show animations in
//! Windows" (SPI_GETCLIENTAREAANIMATION), which is also what Chromium maps
//! `prefers-reduced-motion` to, so the port flattens shimmer exactly when
//! the browser would.

#[cfg(windows)]
pub fn reduced_motion_requested() -> bool {
    use std::ffi::c_void;

    use windows::Win32::UI::WindowsAndMessaging::{
        SystemParametersInfoW, SPI_GETCLIENTAREAANIMATION, SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS,
    };

    let mut animations_on: i32 = 1;
    // SAFETY: SPI_GETCLIENTAREAANIMATION writes one BOOL (an i32) into the
    // buffer we pass, which lives for the whole call.
    let read = unsafe {
        SystemParametersInfoW(
            SPI_GETCLIENTAREAANIMATION,
            0,
            Some(&mut animations_on as *mut i32 as *mut c_void),
            SYSTEM_PARAMETERS_INFO_UPDATE_FLAGS(0),
        )
    };
    read.is_ok() && animations_on == 0
}

#[cfg(not(windows))]
pub fn reduced_motion_requested() -> bool {
    false
}
