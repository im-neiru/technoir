use core::{
    ffi::c_void,
    mem,
    num::NonZeroU16,
    ptr::{self, NonNull},
};

use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
};

use windows_sys::Win32::{
    Foundation::{ERROR_CLASS_ALREADY_EXISTS, GetLastError},
    UI::WindowsAndMessaging::*,
};

use crate::{
    graphics::{Context, WindowSurface},
    wallpaper::{ScreenBounds, WallpaperDriver, event_loop::window_proc},
};

pub struct WallpaperTarget {
    pub(in crate::wallpaper) hwnd: NonNull<c_void>,
    hinstance: NonNull<c_void>,
    classname: [u16; 96],

    pub(in crate::wallpaper) surface: WindowSurface,
}

impl WallpaperTarget {
    pub(in crate::wallpaper) async fn new(
        screen_name: &str,
        bounds: &ScreenBounds,
        hinstance: NonNull<c_void>,
        parent: NonNull<c_void>,
        context: &Context,
    ) -> Self {
        unsafe {
            let classname = Self::build_classname(screen_name);

            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance.as_ptr(),
                hIcon: ptr::null_mut(),
                hCursor: LoadCursorW(ptr::null_mut(), IDC_ARROW),
                hbrBackground: ptr::null_mut(),
                lpszMenuName: ptr::null_mut(),
                lpszClassName: classname.as_ptr(),
            };

            if RegisterClassW(&wnd_class) == 0 {
                let err = GetLastError();
                if err != ERROR_CLASS_ALREADY_EXISTS {
                    panic!("Failed to register window class: {}", err);
                }
            }

            let Some(hwnd) = NonNull::new(CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                classname.as_ptr(),
                classname.as_ptr(),
                WS_POPUP,
                0,
                0,
                100,
                100,
                parent.as_ptr(),
                ptr::null_mut(),
                hinstance.as_ptr(),
                ptr::null_mut(),
            )) else {
                let err = GetLastError();
                panic!("Failed to create window: {}", err);
            };

            SetLayeredWindowAttributes(hwnd.as_ptr(), 0, 255, LWA_ALPHA);

            SetParent(hwnd.as_ptr(), parent.as_ptr());

            SetWindowPos(
                hwnd.as_ptr(),
                HWND_TOP,
                bounds.x,
                bounds.y,
                bounds.width as i32,
                bounds.height as i32,
                SWP_SHOWWINDOW | SWP_NOACTIVATE,
            );

            let (width, height) = {
                let mut rect = mem::zeroed();

                GetClientRect(hwnd.as_ptr() as _, &mut rect);

                (
                    NonZeroU16::new_unchecked((rect.right - rect.left).max(1) as u16),
                    NonZeroU16::new_unchecked((rect.bottom - rect.top).max(1) as u16),
                )
            };

            let target = wgpu::SurfaceTargetUnsafe::RawHandle {
                raw_display_handle: Some(RawDisplayHandle::Windows(WindowsDisplayHandle::new())),
                raw_window_handle: RawWindowHandle::Win32({
                    let mut h = Win32WindowHandle::new(hwnd.addr().cast_signed());

                    h.hinstance = Some(hinstance.addr().cast_signed());

                    h
                }),
            };

            let surface = context.create_window_surface(target, width, height);

            ShowWindow(hwnd.as_ptr(), SW_HIDE);

            Self {
                hwnd,
                hinstance,
                classname,
                surface,
            }
        }
    }

    #[inline]
    pub(crate) fn resize(&mut self, context: &Context) {
        let (width, height) = unsafe {
            let mut rect = mem::zeroed();

            GetClientRect(self.hwnd.as_ptr() as _, &mut rect);

            (
                NonZeroU16::new_unchecked((rect.right - rect.left).max(1) as u16),
                NonZeroU16::new_unchecked((rect.bottom - rect.top).max(1) as u16),
            )
        };

        self.surface.resize(context, width, height);
    }

    fn build_classname(screen_name: &str) -> [u16; 96] {
        let base = "TechNoirTarget ";

        let mut buf = [0u16; 96];
        let mut i = 0;

        for ch in base.encode_utf16() {
            if i >= buf.len() - 1 {
                break;
            }
            buf[i] = ch;
            i += 1;
        }

        for ch in screen_name.encode_utf16() {
            if i >= buf.len() - 1 {
                break;
            }
            buf[i] = ch;
            i += 1;
        }

        buf[i] = 0;

        buf
    }

    pub(in crate::wallpaper) fn show(&mut self, driver: NonNull<WallpaperDriver>) {
        unsafe {
            SetWindowLongPtrA(
                self.hwnd.as_ptr(),
                GWL_USERDATA,
                driver.addr().get().cast_signed(),
            );

            ShowWindow(self.hwnd.as_ptr(), SW_SHOW);
        }
    }
}

impl Drop for WallpaperTarget {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.hwnd.as_ptr());
            UnregisterClassW(self.classname.as_ptr(), self.hinstance.as_ptr());
        }
    }
}
