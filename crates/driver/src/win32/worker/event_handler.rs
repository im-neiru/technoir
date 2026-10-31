use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::WindowsAndMessaging::*,
};

pub(super) unsafe extern "system" fn event_handler(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_CLOSE => unsafe {
            DestroyWindow(hwnd);

            0
        },

        WM_DESTROY => unsafe {
            PostQuitMessage(0);

            0
        },

        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}
