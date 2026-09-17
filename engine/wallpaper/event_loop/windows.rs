use core::{
    mem,
    ptr::{self, NonNull},
};

use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::Threading::Sleep,
    UI::WindowsAndMessaging::*,
};

use crate::wallpaper::WallpaperDriver;

pub(in crate::wallpaper) const WM_APP_TERMINATE: u32 = WM_APP + 1;

#[allow(unsafe_op_in_unsafe_fn)]
pub(in crate::wallpaper) unsafe fn enter_loop(driver: &mut WallpaperDriver) {
    {
        let ptr = unsafe { NonNull::new_unchecked(driver) };

        for s in driver.screens.iter_mut() {
            if let Some(target) = s.target.as_mut() {
                target.show(ptr);
            }
        }
    }

    let mut msg = mem::zeroed();

    'outer: loop {
        while PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
            if msg.message == WM_QUIT {
                break 'outer;
            }

            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        driver.poll_desktop_state();

        Sleep(17);
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(in crate::wallpaper) unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let driver_ptr = unsafe { GetWindowLongPtrA(hwnd, GWL_USERDATA) as *mut WallpaperDriver };

    if driver_ptr.is_null() {
        return unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) };
    }

    let driver = unsafe { &mut *driver_ptr };

    match msg {
        WM_APP_TERMINATE => {
            DestroyWindow(hwnd);
            0
        }
        WM_CLOSE => 0,
        WM_SIZE => {
            driver.resize_target(hwnd);

            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
