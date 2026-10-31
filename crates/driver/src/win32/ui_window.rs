use core::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use windows_sys::{
    Win32::{Foundation::GetLastError, UI::WindowsAndMessaging::*},
    w,
};

use crate::errors::WindowsOsError;

pub(crate) struct UiWindow {
    handle: NonNull<c_void>,
}

impl super::worker::Worker {
    #[inline]
    pub(crate) fn create_ui_window(&self) -> Result<UiWindow, WindowsOsError> {
        let Some(handle) = NonNull::new(unsafe {
            CreateWindowExW(
                0,
                self.make_in_atom(),
                w!("Technoir"),
                WS_OVERLAPPEDWINDOW,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                ptr::null_mut(),
                ptr::null_mut(),
                self.instance.as_ptr(),
                ptr::null_mut(),
            )
        }) else {
            return Err(WindowsOsError::FailedToCreateWindow {
                error_code: unsafe { GetLastError() },
            });
        };

        Ok(UiWindow { handle })
    }
}

impl UiWindow {
    #[inline]
    pub(crate) fn show(&self) {
        unsafe { ShowWindow(self.handle.as_ptr(), SW_SHOW) };
    }
}

impl Drop for UiWindow {
    fn drop(&mut self) {
        unsafe { DestroyWindow(self.handle.as_ptr()) };
    }
}
