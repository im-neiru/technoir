use windows_sys::Win32::UI::WindowsAndMessaging::{WM_APP, WM_USER};

pub(super) const WM_APP_TERMINATE: u32 = WM_APP + 1;
pub(super) const WM_USER_TRAY: u32 = WM_USER + 1;
