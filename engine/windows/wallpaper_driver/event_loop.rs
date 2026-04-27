use core::{
    mem,
    ptr::{self, NonNull},
};

use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    System::Threading::Sleep,
    UI::WindowsAndMessaging::*,
};

use crate::{
    AudioLoopback,
    windows::{
        messages::WM_APP_TERMINATE,
        wallpaper_driver::{WallpaperDriver, target::WallpaperTarget},
    },
};

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn enter_loop(driver: &mut WallpaperDriver) {
    let mut msg = mem::zeroed();

    for screen in &mut driver.screens {
        screen.store_state();
    }

    let mut audio = AudioLoopback::new();

    'outer: loop {
        while PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
            if msg.message == WM_QUIT {
                break 'outer;
            }

            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }

        audio.poll();

        for screen in &mut driver.screens {
            if let Some(target) = screen.target.as_mut() {
                target.render_wave(audio.samples());
            }
        }

        Sleep(1);
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let mut target =
        unsafe { NonNull::new(GetWindowLongPtrA(hwnd, GWL_USERDATA) as *mut WallpaperTarget) };

    match msg {
        WM_APP_TERMINATE => {
            DestroyWindow(hwnd);
            0
        }
        WM_CLOSE => 0,
        WM_SIZE => {
            let _width = (lparam & 0xFFFF) as u32;
            let _height = (lparam >> 16) as u32;

            if let Some(target) = target.as_mut() {
                let target = unsafe { target.as_mut() };
                let width = (lparam & 0xFFFF) as u32;
                let height = (lparam >> 16) as u32;
                target.resize(width, height);
            }

            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
