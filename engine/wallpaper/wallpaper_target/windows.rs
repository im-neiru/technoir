use core::{
    ffi::c_void,
    mem,
    ptr::{self, NonNull},
};

use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
};

use wgpu::{Surface, SurfaceConfiguration};
use windows_sys::Win32::{
    Foundation::{ERROR_CLASS_ALREADY_EXISTS, GetLastError},
    UI::WindowsAndMessaging::*,
};

use crate::wallpaper::{ScreenBounds, event_loop::window_proc};

pub struct WallpaperTarget {
    pub(in crate::wallpaper) hwnd: NonNull<c_void>,
    hinstance: NonNull<c_void>,
    classname: [u16; 96],

    pub(in crate::wallpaper) wgpu_surface: Surface<'static>,
    pub(in crate::wallpaper) config: SurfaceConfiguration,
}

impl WallpaperTarget {
    pub(in crate::wallpaper) fn new(
        screen_name: &str,
        bounds: &ScreenBounds,
        hinstance: NonNull<c_void>,
        parent: NonNull<c_void>,
        wgpu_instance: &wgpu::Instance,
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

            let wgpu_surface = wgpu_instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: Some(
                        RawDisplayHandle::Windows(WindowsDisplayHandle::new()),
                    ),

                    raw_window_handle: RawWindowHandle::Win32(Win32WindowHandle::new(
                        hwnd.addr().cast_signed(),
                    )),
                })
                .expect("Failed to create wgpu::Surface");

            let (width, height) = {
                let mut rect = mem::zeroed();

                GetClientRect(hwnd.as_ptr() as _, &mut rect);
                (
                    (rect.right - rect.left).max(1) as u32,
                    (rect.bottom - rect.top).max(1) as u32,
                )
            };

            ShowWindow(hwnd.as_ptr(), SW_HIDE);

            // Default should be replaced when the wgpu::Adapter is retrieved
            let config = wgpu::SurfaceConfiguration {
                usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
                format: wgpu::TextureFormat::Rgba8Unorm,
                width,
                height,
                present_mode: wgpu::PresentMode::AutoVsync,
                alpha_mode: wgpu::CompositeAlphaMode::Auto,
                view_formats: vec![],
                desired_maximum_frame_latency: 2,
            };

            Self {
                hwnd,
                hinstance,
                classname,
                wgpu_surface,
                config,
            }
        }
    }

    pub(in crate::wallpaper) fn configure(
        &mut self,
        device: &wgpu::Device,
        adapter: &wgpu::Adapter,
    ) {
        let caps = self.wgpu_surface.get_capabilities(adapter);

        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| *f == wgpu::TextureFormat::Rgba8Unorm)
            .unwrap_or(caps.formats[0]);

        self.config.format = format;

        self.wgpu_surface.configure(&device, &self.config);
    }

    #[inline]
    pub(crate) fn resize(&mut self, device: &wgpu::Device, width: u32, height: u32) {
        self.config.width = width;
        self.config.height = height;
        self.wgpu_surface.configure(&device, &self.config);
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

    pub(in crate::wallpaper) fn show(&mut self) {
        unsafe {
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
