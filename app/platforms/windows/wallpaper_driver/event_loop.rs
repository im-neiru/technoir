use core::{mem, ptr};

use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    Graphics::Gdi::*,
    UI::WindowsAndMessaging::*,
};

use crate::platforms::windows::messages::WM_APP_TERMINATE;

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe fn enter_loop() {
    let mut msg = mem::zeroed();

    'outer: loop {
        while PeekMessageW(&mut msg, ptr::null_mut(), 0, 0, PM_REMOVE) != 0 {
            if msg.message == WM_QUIT {
                break 'outer;
            }

            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_APP_TERMINATE => {
            println!("GOING TO TERMINATE");

            DestroyWindow(hwnd);
            0
        }
        WM_CLOSE => 0,
        WM_PAINT => {
            unsafe {
                println!("Render");
                let mut ps: PAINTSTRUCT = mem::zeroed();
                let hdc = BeginPaint(hwnd, &mut ps);

                let mut rect = mem::zeroed();
                GetClientRect(hwnd, &mut rect);

                let brush = CreateSolidBrush(0x0f0f0f);
                FillRect(hdc, &rect, brush);
                DeleteObject(brush);

                EndPaint(hwnd, &ps);

                // TODO
            }

            0
        }
        WM_SIZE => {
            let _width = (lparam & 0xFFFF) as u32;
            let _height = (lparam >> 16) as u32;
            // TODO
            0
        }
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}
