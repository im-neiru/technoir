mod event_handler;

use core::{
    ffi::c_void,
    mem,
    num::NonZeroU16,
    ptr::{self, NonNull},
};

use windows_sys::{
    Win32::{System::LibraryLoader::GetModuleHandleW, UI::WindowsAndMessaging::*},
    w,
};

use super::desktop_handles::DesktopHandles;
use crate::errors::WindowsOsError;

pub(in super::super) struct Worker {
    pub(super) instance: NonNull<c_void>,
    pub(super) ui_atom: NonZeroU16,
    pub(super) wallpaper_atom: NonZeroU16,
    pub(super) desktop_handles: DesktopHandles,
}

impl Worker {
    pub(in super::super) fn new() -> Result<Self, WindowsOsError> {
        let desktop_handles =
            DesktopHandles::find().ok_or(WindowsOsError::FailedToRetrieveDesktopHandles)?;

        let Some(instance) = NonNull::new(unsafe { GetModuleHandleW(ptr::null()) }) else {
            return Err(WindowsOsError::FailedToGetModuleHandle);
        };

        let ui_class = WNDCLASSEXW {
            cbSize: mem::size_of::<WNDCLASSEXW>() as u32,
            style: CS_HREDRAW | CS_VREDRAW | CS_OWNDC,
            lpfnWndProc: Some(event_handler::event_handler),
            cbClsExtra: 0,
            cbWndExtra: 0,
            hInstance: instance.as_ptr(),
            hIcon: ptr::null_mut(),
            hCursor: unsafe { LoadCursorW(ptr::null_mut(), IDC_ARROW) },
            hbrBackground: ptr::null_mut(),
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
            hIcon: ptr::null_mut(),
            hCursor: unsafe { LoadCursorW(ptr::null_mut(), IDC_ARROW) },
            hbrBackground: ptr::null_mut(),
            lpszMenuName: ptr::null(),
            lpszClassName: w!("technoir_wallpaper"),
            hIconSm: ptr::null_mut(),
        };

        let Some(ui_atom) = NonZeroU16::new(unsafe { RegisterClassExW(&ui_class) }) else {
            return Err(WindowsOsError::FailedToRegisterWindowClass);
        };

        let Some(wallpaper_atom) = NonZeroU16::new(unsafe { RegisterClassExW(&wallpaper_class) })
        else {
            return Err(WindowsOsError::FailedToRegisterWindowClass);
        };

        Ok(Self {
            instance,
            ui_atom,
            wallpaper_atom,
            desktop_handles,
        })
    }

    pub(in super::super) fn run(self) {
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
