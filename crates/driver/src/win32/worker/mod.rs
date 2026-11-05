mod event_handler;
mod render_worker;

use core::{
    ffi::c_void,
    mem,
    num::NonZeroU16,
    ptr::{self, NonNull},
};

use windows_sys::{
    Win32::{
        Graphics::Gdi::COLOR_WINDOW, System::LibraryLoader::GetModuleHandleW,
        UI::WindowsAndMessaging::*,
    },
    core::PCWSTR,
    w,
};

use super::{desktop_handles::DesktopHandles, wallpaper_window::WallpaperWindow};
use crate::errors::{StartupError, WindowsOsError};

pub(in super::super) struct Worker {
    pub(super) instance: NonNull<c_void>,
    pub(super) ui_atom: NonZeroU16,
    pub(super) wallpaper_atom: NonZeroU16,
    pub(super) desktop_handles: DesktopHandles,
    pub(super) wallpaper_windows: Vec<WallpaperWindow>,
    pub(super) render_worker: render_worker::RenderWorker,
}

impl Worker {
    pub(in super::super) fn new() -> Result<Self, StartupError> {
        let desktop_handles = DesktopHandles::find().ok_or(StartupError::WindowOs(
            WindowsOsError::FailedToRetrieveDesktopHandles,
        ))?;

        let Some(instance) = NonNull::new(unsafe { GetModuleHandleW(ptr::null()) }) else {
            return Err(StartupError::WindowOs(
                WindowsOsError::FailedToGetModuleHandle,
            ));
        };

        let app_icon = unsafe {
            LoadImageW(
                instance.as_ptr(),
                1 as PCWSTR,
                IMAGE_ICON,
                0,
                0,
                LR_DEFAULTSIZE | LR_SHARED,
            )
        } as HICON;

        let ui_class = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
            lpfnWndProc: Some(event_handler::event_handler),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.as_ptr(),
            hIcon: app_icon,
            hCursor: unsafe { LoadCursorW(ptr::null_mut(), IDC_ARROW) },
            hbrBackground: (COLOR_WINDOW + 1) as _,
            lpszMenuName: ptr::null(),
            lpszClassName: w!("technoir_ui"),
            hIconSm: ptr::null_mut(),
        };

        let wallpaper_class = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
            lpfnWndProc: Some(event_handler::event_handler),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.as_ptr(),
            hIcon: app_icon,
            hCursor: unsafe { LoadCursorW(ptr::null_mut(), IDC_ARROW) },
            hbrBackground: (COLOR_WINDOW + 1) as _,
            lpszMenuName: ptr::null(),
            lpszClassName: w!("technoir_wallpaper"),
            hIconSm: ptr::null_mut(),
        };

        let (Some(ui_atom), Some(wallpaper_atom)) = (
            NonZeroU16::new(unsafe { RegisterClassExW(&ui_class) }),
            NonZeroU16::new(unsafe { RegisterClassExW(&wallpaper_class) }),
        ) else {
            return Err(StartupError::WindowOs(
                WindowsOsError::FailedToRegisterWindowClass,
            ));
        };

        let wallpaper_windows = Self::create_wallpaper_windows(
            instance,
            wallpaper_atom.get() as usize as *const u16,
            &desktop_handles,
        )?;

        let render_worker = render_worker::RenderWorker::new(instance, &wallpaper_windows)?;

        Ok(Self {
            instance,
            ui_atom,
            wallpaper_atom,
            desktop_handles,
            wallpaper_windows,
            render_worker,
        })
    }

    pub(in super::super) fn run(self) {
        self.render_worker.resume();

        unsafe {
            let mut msg = mem::zeroed();

            loop {
                let result = GetMessageW(&mut msg, ptr::null_mut(), 0, 0);

                match result {
                    -1 => {
                        panic!("GetMessageA failed");
                    }
                    0 => break,
                    _ => {
                        TranslateMessage(&msg);
                        DispatchMessageA(&msg);
                    }
                }
            }
        }
    }

    #[inline(always)]
    pub(super) const fn make_in_ui_atom(&self) -> *const u16 {
        self.ui_atom.get() as usize as *const u16
    }

    #[inline(always)]
    pub(super) const fn make_in_wallpaper_atom(&self) -> *const u16 {
        self.wallpaper_atom.get() as usize as *const u16
    }
}

impl Drop for Worker {
    fn drop(&mut self) {
        unsafe {
            UnregisterClassW(self.make_in_ui_atom(), self.instance.as_ptr());
            UnregisterClassW(self.make_in_wallpaper_atom(), self.instance.as_ptr());
        };
    }
}
