use core::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE, HWND},
    System::{
        LibraryLoader::GetModuleHandleA,
        Threading::{CreateThread, INFINITE, WaitForSingleObject},
    },
    UI::WindowsAndMessaging::PostMessageW,
};

use crate::{
    utils::{DesktopWatcher, get_desktop_handles},
    wallpaper::{
        Screen,
        event_loop::{WM_APP_TERMINATE, enter_loop},
        renderer::WallpaperRenderer,
    },
};

pub struct WallpaperDriver {
    pub(in crate::wallpaper) screens: Vec<Screen>,
    instance: wgpu::Instance,
    renderer: Option<WallpaperRenderer>,
    watcher: Option<DesktopWatcher>,
    worker: HANDLE,
}

impl WallpaperDriver {
    pub(crate) async fn new(instance: wgpu::Instance) -> Self {
        let screens = Screen::get_screens();

        Self {
            screens,
            instance,
            renderer: None,
            worker: ptr::null_mut(),
            watcher: None,
        }
    }

    pub(in crate::wallpaper) fn resize_target(&mut self, hwnd: HWND, width: u32, height: u32) {
        let Some(renderer) = self.renderer.as_ref() else {
            return;
        };

        for s in self.screens.iter_mut() {
            if let Some(target) = s.target.as_mut()
                && target.hwnd.as_ptr() == hwnd
            {
                target.resize(&renderer.device, width, height);
            }
        }
    }

    pub fn run(&mut self) {
        if !self.worker.is_null() {
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

        if self.worker.is_null() {
            panic!("Failed to create thread");
        }
    }

    #[inline(always)]
    fn run_internal(&mut self) {
        let hinstance = unsafe {
            NonNull::new(GetModuleHandleA(ptr::null_mut())).expect("Failed to retrieve hinstance")
        };

        self.watcher = Some(DesktopWatcher::new().expect("Failed to create desktop watcher"));

        let renderer = smol::block_on(async {
            WallpaperRenderer::new(&self.instance, None, self.screens.as_mut_slice()).await
        })
        .unwrap();

        self.renderer = Some(renderer);

        let desktop_handles = get_desktop_handles();
        let parent = desktop_handles.get_target_parent();

        for s in self.screens.iter_mut() {
            s.spawn_target(hinstance, parent, &self.instance);
        }

        unsafe { enter_loop(self) }
    }

    unsafe extern "system" fn run_thread(param: *mut c_void) -> u32 {
        let driver = unsafe { &mut *(param as *mut Self) };
        driver.run_internal();
        0
    }

    pub fn terminate(&mut self) {
        if self.worker.is_null() {
            return;
        };

        for s in self.screens.iter() {
            if let Some(target) = s.target.as_ref() {
                unsafe { PostMessageW(target.hwnd.as_ptr(), WM_APP_TERMINATE, 0, 0) };
            }
        }

        unsafe {
            WaitForSingleObject(self.worker, INFINITE);
            CloseHandle(self.worker);
        };

        self.worker = ptr::null_mut();
    }

    #[inline]
    pub fn poll_desktop_state(&mut self) {
        let Some(watcher) = self.watcher.as_ref() else {
            return;
        };

        let Some(entry) = watcher.poll() else {
            return;
        };

        for s in self.screens.iter_mut() {
            if s.hmonitor == entry.monitor_handle() {
                println!(
                    "{:x} {:x} {}",
                    s.hmonitor.addr(),
                    entry.window_handle().addr(),
                    entry.is_full()
                );
                s.is_filled = entry.is_full()
            }
        }
    }
}

impl Drop for WallpaperDriver {
    fn drop(&mut self) {
        self.terminate();
    }
}
