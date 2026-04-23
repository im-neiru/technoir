use core::{
    ffi::c_void,
    num::NonZero,
    ptr::{self, NonNull},
};

use windows_sys::{
    Win32::{Foundation::*, System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::*},
    core::PCWSTR,
    w,
};

pub struct State {
    manager_win: Option<NonNull<c_void>>,
}

impl State {
    pub(super) fn new(config: &crate::config::Config) -> Self {
        let hinstance = unsafe { NonNull::new(GetModuleHandleW(ptr::null_mut())) }
            .expect("Failed to retrieve module handle");

        Self {
            manager_win: if config.open_manager {
                unsafe { Some(Self::new_manager_win(hinstance)) }
            } else {
                None
            },
        }
    }

    pub(super) fn enter_loop(self) {
        unsafe {
            let mut msg = std::mem::zeroed::<MSG>();

            while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe {
            if let Some(manager_hwnd) = self.manager_win {
                DestroyWindow(manager_hwnd.as_ptr() as _);
            }
        }
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    match msg {
        WM_DESTROY => {
            PostQuitMessage(0);
            0
        }
        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

impl State {
    const SANDBOX_WIN_NAME: PCWSTR = w!("TechNoir");

    unsafe fn new_manager_win(hinstance: NonNull<c_void>) -> NonNull<c_void> {
        let wc = unsafe {
            let cursor = LoadCursorW(ptr::null_mut(), IDC_ARROW);

            WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(window_proc),
                hInstance: hinstance.as_ptr(),
                lpszClassName: Self::SANDBOX_WIN_NAME,
                hCursor: cursor,

                ..std::mem::zeroed()
            }
        };

        unsafe { NonZero::new(RegisterClassW(&wc)) }.expect("Failed to register window class");

        NonNull::new(unsafe {
            CreateWindowExW(
                0,
                Self::SANDBOX_WIN_NAME,
                Self::SANDBOX_WIN_NAME,
                WS_OVERLAPPEDWINDOW | WS_VISIBLE,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                800,
                600,
                ptr::null_mut(),
                ptr::null_mut(),
                hinstance.as_ptr(),
                ptr::null_mut(),
            )
        })
        .expect("Failed to create window")
    }
}
