use std::{ffi::c_void, ptr::NonNull};

use renderer::{
    WgpuHasDisplayHandle,
    rwh::{self, HasDisplayHandle, HasWindowHandle, Win32WindowHandle},
};

#[derive(Debug)]
pub(super) struct DisplayHandle;

impl HasDisplayHandle for DisplayHandle {
    #[inline(always)]
    fn display_handle(&self) -> Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
        Ok(rwh::DisplayHandle::windows())
    }
}

impl From<DisplayHandle> for Box<dyn WgpuHasDisplayHandle> {
    #[inline]
    fn from(val: DisplayHandle) -> Self {
        Box::new(DisplayHandle)
    }
}

pub struct WindowHandle(rwh::Win32WindowHandle);

impl WindowHandle {
    #[inline(always)]
    pub(super) fn new(instance: NonNull<c_void>, window_handle: NonNull<c_void>) -> Self {
        let mut handle = Win32WindowHandle::new(window_handle.addr().cast_signed());

        handle.hinstance = Some(instance.addr().cast_signed());

        Self(handle)
    }
}

impl HasWindowHandle for WindowHandle {
    #[inline(always)]
    fn window_handle(&self) -> Result<rwh::WindowHandle<'_>, rwh::HandleError> {
        Ok(unsafe { rwh::WindowHandle::borrow_raw(rwh::RawWindowHandle::Win32(self.0)) })
    }
}
