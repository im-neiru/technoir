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

pub(crate) const WM_APP_TERMINATE: u32 = WM_APP + 1;

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe fn enter_loop(driver: &mut WallpaperDriver) {
    {
        let ptr = unsafe { NonNull::new_unchecked(driver) };

        for s in driver.screens.iter_mut() {
            if let Some(target) = s.target.as_mut() {
                target.show(ptr);
            }
        }
    }

    // let ref_time = std::time::Instant::now();
    // let mut fft = crate::SpectrumAudioLoopback::new();

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

        // fft.poll();
        // let elapsed = ref_time.elapsed().as_secs_f32();

        // for screen in &mut driver.screens {
        //     if !screen.is_filled
        //         && let Some(target) = screen.target.as_mut()
        //     {
        //         target.visualizer.render(&mut fft, elapsed);
        //     }
        // }

        Sleep(17);
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(crate) unsafe extern "system" fn window_proc(
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
            let _width = (lparam & 0xFFFF) as u32;
            let _height = (lparam >> 16) as u32;

            let width = (lparam & 0xFFFF) as u32;
            let height = (lparam >> 16) as u32;

            driver.resize_target(hwnd, width, height);

            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
