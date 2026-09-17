mod destroy;
mod event_loop;
mod init;

use core::{ffi::c_void, ptr::NonNull};

use windows_sys::Win32::UI::WindowsAndMessaging::{PostMessageW, SW_HIDE, SW_SHOW, ShowWindow};

use event_loop::{WM_APP_TERMINATE, WM_USER_TRAY};

pub(crate) use event_loop::enter_loop;

use crate::graphics::{Context, WindowSurface};

pub struct Manager {
    hwnd: NonNull<c_void>,
    hinstance: NonNull<c_void>,
    is_open: bool,
    ui: ui::ManagerUi,
    ctx: Context,
    surface: WindowSurface,
}

impl Manager {
    pub(super) fn render(&mut self) {
        self.ui.render();
    }
}

impl Manager {
    pub(super) fn show(&mut self) {
        if !self.is_open {
            unsafe { ShowWindow(self.hwnd.as_ptr(), SW_SHOW) };
            self.is_open = true;
        }
    }

    pub(super) fn hide(&mut self) {
        if self.is_open {
            unsafe { ShowWindow(self.hwnd.as_ptr(), SW_HIDE) };
            self.is_open = false;
        }
    }

    pub(super) fn terminate_program(&mut self) {
        unsafe { PostMessageW(self.hwnd.as_ptr(), WM_APP_TERMINATE, 0, 0) };
    }
}
