mod desktop_handles;
mod event_loop;
mod screen;
mod target;
mod watcher;

use core::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use rapidhash::RapidHashSet;
use vello::wgpu;
use windows_sys::Win32::{
    Foundation::CloseHandle,
    System::{
        LibraryLoader::GetModuleHandleW,
        Threading::{CreateThread, INFINITE, WaitForSingleObject},
    },
    UI::WindowsAndMessaging::{GetForegroundWindow, IsIconic, IsWindowVisible},
};

use desktop_handles::DesktopHandles;
pub use screen::Screen;

pub struct WallpaperDriver {
    screens: Vec<Screen>,
    handle_set: RapidHashSet<usize>,
    desktop_handles: DesktopHandles,
    thread: Option<NonNull<c_void>>,
    wgpu_instance: wgpu::Instance,
    watcher: Option<watcher::WatcherGuard>,
}

impl WallpaperDriver {
    pub fn new(wgpu_instance: &wgpu::Instance) -> Self {
        let screens = Screen::get_screens();
        let desktop_handles = DesktopHandles::find();

        Self {
            screens,
            desktop_handles,
            thread: None,
            wgpu_instance: wgpu_instance.clone(),
            handle_set: RapidHashSet::default(),
            watcher: None,
        }
    }

    pub fn terminate(&mut self) {
        if let Some(thread) = self.thread {
            unsafe {
                for s in &mut self.screens {
                    s.send_terminate();
                }

                WaitForSingleObject(thread.as_ptr(), INFINITE);
            }

            self.thread = None;
        }
    }

    pub fn run(&mut self) {
        self.watcher = watcher::start_watching();

        let ptr = self as *mut _ as *mut c_void;

        let thread = unsafe {
            NonNull::new(CreateThread(
                core::ptr::null_mut(),
                0,
                Some(Self::run_thread),
                ptr,
                0,
                core::ptr::null_mut(),
            ))
            .expect("Failed to create thread")
        };

        self.thread = Some(thread);
    }

    #[inline]
    pub fn contains_handle(&self, handle: NonNull<c_void>) -> bool {
        self.handle_set.contains(&handle.addr().get())
    }

    pub fn init(&mut self) {
        for screen in &mut self.screens {
            if let Some(handle) = screen.init() {
                self.handle_set.insert(handle.get());
            }
        }
    }

    #[inline]
    pub fn has_overlay(&self) -> bool {
        let fg_hwnd = unsafe { GetForegroundWindow() };

        let Some(fg_hwnd) = NonNull::new(fg_hwnd) else {
            return false;
        };

        if self.contains_handle(fg_hwnd) {
            return false;
        }

        unsafe {
            if IsWindowVisible(fg_hwnd.as_ptr()) == 0 {
                return false;
            }

            if IsIconic(fg_hwnd.as_ptr()) != 0 {
                return false;
            }

            true
        }
    }

    fn run_internal(&mut self) {
        self.desktop_handles.redraw();

        let hinstance = NonNull::new(unsafe { GetModuleHandleW(ptr::null_mut()) })
            .expect("Failed to retrieve module handle");
        let target = self.desktop_handles.get_target_parent();

        for s in self.screens.iter_mut() {
            s.spawn_target(hinstance, target, &self.wgpu_instance);
        }

        unsafe { event_loop::enter_loop(self) }
    }

    unsafe extern "system" fn run_thread(param: *mut c_void) -> u32 {
        let driver = unsafe { &mut *(param as *mut Self) };
        driver.run_internal();
        0
    }
}

impl Drop for WallpaperDriver {
    fn drop(&mut self) {
        self.terminate();

        if let Some(thread) = self.thread {
            unsafe { CloseHandle(thread.as_ptr()) };
        }
    }
}
