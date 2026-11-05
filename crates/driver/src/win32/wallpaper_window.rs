use core::{
    ffi::c_void,
    num::NonZeroUsize,
    ptr::{self, NonNull},
};

use windows_sys::{
    Win32::{Foundation::GetLastError, UI::WindowsAndMessaging::*},
    core::PCWSTR,
    w,
};

use crate::{
    errors::{StartupError, WindowsOsError},
    monitor::{Monitor, MonitorHandleValue},
    win32::desktop_handles::DesktopHandles,
};

pub(in super::super) struct WallpaperWindow {
    pub(super) handle: NonNull<c_void>,
    pub(super) monitor: NonZeroUsize,
}

impl super::worker::Worker {
    #[inline]
    pub(crate) fn create_wallpaper_windows(
        instance: NonNull<c_void>,
        atom: PCWSTR,
        desktop_handles: &DesktopHandles,
    ) -> Result<Vec<WallpaperWindow>, StartupError> {
        let monitors = Monitor::get_monitors();

        let mut buf = Vec::with_capacity(monitors.len());

        for m in monitors.iter() {
            let parent = desktop_handles.get_target_parent().as_ptr();
            let Some(handle) = NonNull::new(unsafe {
                CreateWindowExW(
                    WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                    atom.cast(),
                    w!("Technoir"),
                    WS_POPUP,
                    m.bounds.x,
                    m.bounds.y,
                    m.bounds.width as i32,
                    m.bounds.height as i32,
                    parent,
                    ptr::null_mut(),
                    instance.as_ptr(),
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

            let monitor = match m.handle.0 {
                MonitorHandleValue::Win32(monitor) => monitor,
                #[allow(unreachable_patterns)]
                _ => unreachable!(),
            };

            buf.push(WallpaperWindow { handle, monitor });
        }

        Ok(buf)
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
