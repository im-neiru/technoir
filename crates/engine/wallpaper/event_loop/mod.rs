#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "windows")]
pub(in crate::wallpaper) use windows::{WM_APP_TERMINATE, enter_loop, window_proc};
