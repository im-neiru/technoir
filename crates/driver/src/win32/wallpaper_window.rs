use core::{ffi::c_void, ptr::NonNull};

use windows_sys::Win32::UI::WindowsAndMessaging::DestroyWindow;

pub(in super::super) struct WallpaperWindow {
    handle: NonNull<c_void>,
}

impl Drop for WallpaperWindow {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.handle.as_ptr());
        }
    }
}
