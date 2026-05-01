use core::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use windows_sys::Win32::{
    Foundation::{HANDLE, HWND},
    System::Threading::CreateThread,
};

use crate::{
    utils::get_desktop_handles,
    wallpaper::{Screen, event_loop::enter_loop, renderer::WallpaperRenderer},
};

pub struct WallpaperDriver {
    screens: Vec<Screen>,
    renderer: WallpaperRenderer,
    worker: HANDLE,
}

impl WallpaperDriver {
    pub(crate) async fn new(wgpu_instance: &wgpu::Instance, hinstance: NonNull<c_void>) -> Self {
        let mut screens = Screen::get_screens();

        let renderer = WallpaperRenderer::new(wgpu_instance, None, screens.as_mut_slice())
            .await
            .unwrap();

        let desktop_handles = get_desktop_handles();
        let parent = desktop_handles.get_target_parent();

        for s in screens.iter_mut() {
            s.spawn_target(hinstance, parent, wgpu_instance);
        }

        Self {
            screens,
            renderer,
            worker: ptr::null_mut(),
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

    pub fn run(&mut self) {
        if self.worker != ptr::null_mut() {
            return;
        };

        self.worker = unsafe {
            let ptr = self as *mut _ as *mut c_void;

            CreateThread(
                core::ptr::null_mut(),
                0,
                Some(Self::run_thread),
                ptr,
                0,
                core::ptr::null_mut(),
            )
        };

        if self.worker == ptr::null_mut() {
            panic!("Failed to create thread");
        }
    }

    #[inline(always)]
    fn run_internal(&mut self) {
        unsafe { enter_loop(self) }
    }

    unsafe extern "system" fn run_thread(param: *mut c_void) -> u32 {
        let driver = unsafe { &mut *(param as *mut Self) };
        driver.run_internal();
        0
    }
}
