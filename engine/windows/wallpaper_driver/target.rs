use core::{
    ffi::c_void,
    mem,
    ptr::{self, NonNull},
};
use std::collections::VecDeque;

use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
};
use vello::wgpu;
use windows_sys::Win32::{
    Foundation::{ERROR_CLASS_ALREADY_EXISTS, GetLastError},
    UI::WindowsAndMessaging::*,
};

use crate::{Renderer, ScreenBounds};

pub struct WallpaperTarget {
    pub(super) hwnd: NonNull<c_void>,
    hinstance: NonNull<c_void>,
    classname: [u16; 96],
    renderer: Renderer,
}

impl WallpaperTarget {
    pub(super) async fn new(
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
                lpfnWndProc: Some(super::event_loop::window_proc),
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
                WS_POPUP | WS_VISIBLE,
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
                    raw_display_handle: RawDisplayHandle::Windows(WindowsDisplayHandle::new()),

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

            Self {
                hwnd,
                hinstance,
                classname,
                renderer: Renderer::new(wgpu_instance, wgpu_surface, width, height).await,
            }
        }
    }

    #[inline]
    pub(crate) fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
    }

    pub fn render_wave(&mut self, samples: &[f32]) {
        use vello::kurbo::{Affine, BezPath, Stroke};
        use vello::peniko::Color;

        let mut scene = vello::Scene::new();
        let width = self.renderer.width() as f64;
        let height = self.renderer.height() as f64;
        let mid_y = height / 2.0;

        let mut path = BezPath::new();
        let x_step = width / (samples.len() as f64);

        for (i, &sample) in samples.iter().enumerate() {
            let x = i as f64 * x_step;

            let y = mid_y + (sample as f64 * height * 0.4);

            if i == 0 {
                path.move_to((x, y));
            } else {
                path.line_to((x, y));
            }
        }

        scene.fill(
            vello::peniko::Fill::NonZero,
            Affine::IDENTITY,
            Color::BLACK,
            None,
            &vello::kurbo::Rect::new(0.0, 0.0, width, height),
        );

        scene.stroke(
            &Stroke::new(3.0),
            Affine::IDENTITY,
            Color::from_rgb8(0, 255, 200), // Cyan-ish
            None,
            &path,
        );

        self.renderer.render_scene(&scene, Color::BLACK);
    }

    // pub(crate) fn render(&mut self, elapsed: f32) {
    //     use vello::{Scene, kurbo::Circle, peniko::*};
    //     let mut scene = Scene::new();
    //     let base_color = Color::BLACK;

    //     let radius = 50.0;
    //     let cx = 200.0 + 100.0 * (elapsed * 2.0).sin();
    //     let cy = 200.0 + 100.0 * (elapsed * 2.0).cos();
    //     let circle = Circle::new((cx, cy), radius);

    //     scene.fill(
    //         Fill::NonZero,
    //         vello::kurbo::Affine::IDENTITY,
    //         Color::from_rgb8(255, 0, 0),
    //         None,
    //         &circle,
    //     );

    //     self.renderer.render_scene(&scene, base_color);
    // }

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
}

impl Drop for WallpaperTarget {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.hwnd.as_ptr());
            UnregisterClassW(self.classname.as_ptr(), self.hinstance.as_ptr());
        }
    }
}
