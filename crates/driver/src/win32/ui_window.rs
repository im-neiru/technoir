use core::{ffi::c_void, ptr::NonNull};

use windows_sys::Win32::UI::WindowsAndMessaging::DestroyWindow;

pub(in super::super) struct UiWindow {
    handle: NonNull<c_void>,
}

impl Drop for UiWindow {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.handle.as_ptr());
        }
    }
}
