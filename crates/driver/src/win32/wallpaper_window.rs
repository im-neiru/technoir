use core::{
    ffi::c_void,
    num::NonZeroU8,
    ptr::{self, NonNull},
};

use windows_sys::{
    Win32::{Foundation::GetLastError, UI::WindowsAndMessaging::*},
    w,
};

use renderer::TargetSurface;

use crate::{
    errors::{StartupError, WindowsOsError},
    monitor::Monitor,
    win32::{
        get_client_area::get_client_area,
        handles::{DisplayHandle, WindowHandle},
    },
};

pub(in super::super) struct WallpaperWindow {
    handle: NonNull<c_void>,
    target: TargetSurface,
}

impl super::worker::Worker {
    #[inline]
    pub(crate) fn create_wallpaper_windows(
        &mut self,
    ) -> Result<Box<[WallpaperWindow]>, StartupError> {
        self.monitors = Monitor::get_monitors();

        let mut buf = Vec::with_capacity(self.monitors.len());

        for m in self.monitors.iter() {
            let parent = self.desktop_handles.get_target_parent().as_ptr();
            let Some(handle) = NonNull::new(unsafe {
                CreateWindowExW(
                    WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                    self.make_in_wallpaper_atom(),
                    w!("Technoir"),
                    WS_POPUP,
                    m.bounds.x,
                    m.bounds.y,
                    m.bounds.width as i32,
                    m.bounds.height as i32,
                    parent,
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

            unsafe {
                SetLayeredWindowAttributes(handle.as_ptr(), 0, 255, LWA_ALPHA);
                SetParent(handle.as_ptr(), parent);

                SetWindowPos(
                    handle.as_ptr(),
                    HWND_TOP,
                    m.bounds.x,
                    m.bounds.y,
                    m.bounds.width as i32,
                    m.bounds.height as i32,
                    SWP_SHOWWINDOW | SWP_NOACTIVATE,
                );

                ShowWindow(handle.as_ptr(), SW_SHOW);
            }

            let window_handle = WindowHandle::new(self.instance, handle);
            let [width, height] = get_client_area(handle);

            let target = pollster::block_on(async move {
                TargetSurface::new(DisplayHandle, window_handle, width, height, unsafe {
                    NonZeroU8::new_unchecked(3)
                })
                .await
            })
            .map_err(StartupError::GraphicsInit)?;

            buf.push(WallpaperWindow { handle, target });
        }

        Ok(buf.into_boxed_slice())
    }
}

impl Drop for WallpaperWindow {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.handle.as_ptr());
        }
    }
}
