use core::{
    ffi::c_void,
    num::NonZeroU8,
    ptr::{self, NonNull},
};

use renderer::TargetSurface;
use windows_sys::{
    Win32::{Foundation::GetLastError, UI::WindowsAndMessaging::*},
    w,
};

use crate::{
    errors::{StartupError, WindowsOsError},
    win32::{
        get_client_area::get_client_area,
        handles::{DisplayHandle, WindowHandle},
    },
};

pub(crate) struct UiWindow {
    handle: NonNull<c_void>,
    target: TargetSurface,
}

impl super::worker::Worker {
    #[inline]
    pub(crate) fn create_ui_window(&self) -> Result<UiWindow, StartupError> {
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
            return Err(StartupError::WindowOs(
                WindowsOsError::FailedToCreateWindow {
                    error_code: unsafe { GetLastError() },
                },
            ));
        };

        let window_handle = WindowHandle::new(self.instance, handle);
        let [width, height] = get_client_area(handle);

        let target = pollster::block_on(async move {
            TargetSurface::new(DisplayHandle, window_handle, width, height, unsafe {
                NonZeroU8::new_unchecked(3)
            })
            .await
        })
        .map_err(StartupError::GraphicsInit)?;

        Ok(UiWindow { handle, target })
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
