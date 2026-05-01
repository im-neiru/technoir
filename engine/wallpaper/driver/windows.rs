use core::{ffi::c_void, ptr::NonNull};

use windows_sys::Win32::Foundation::HWND;

use crate::wallpaper::{Screen, renderer::WallpaperRenderer};

pub struct WallpaperDriver {
    window: Option<NonNull<c_void>>,
    screens: Vec<Screen>,
    renderer: WallpaperRenderer,
}

impl WallpaperDriver {
    pub(crate) async fn new(wgpu_instance: wgpu::Instance) -> Self {
        let mut screens = Screen::get_screens();

        let renderer = WallpaperRenderer::new(&wgpu_instance, None, screens.as_mut_slice())
            .await
            .unwrap();

        Self {
            window: None,
            screens,
            renderer,
        }
    }

    pub(in crate::wallpaper) fn resize_target(&mut self, hwnd: HWND, width: u32, height: u32) {
        for s in self.screens.iter_mut() {
            if let Some(target) = s.target.as_mut() {
                if target.hwnd.as_ptr() == hwnd {
                    target.resize(&self.renderer.device, width, height);
                }
            }
        }
    }
}
