mod desktop_handles;
mod event_loop;
mod screen;
mod target;

use core::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use desktop_handles::DesktopHandles;

pub use screen::Screen;
use windows_sys::Win32::{
    Foundation::CloseHandle,
    System::{
        LibraryLoader::GetModuleHandleW,
        Threading::{CreateThread, INFINITE, WaitForSingleObject},
    },
};

pub struct WallpaperDriver {
    screens: Vec<Screen>,
    desktop_handles: DesktopHandles,
    thread: Option<NonNull<c_void>>,
}

impl WallpaperDriver {
    pub fn new() -> Self {
        let screens = Screen::get_screens();
        let desktop_handles = DesktopHandles::find();

        Self {
            screens,
            desktop_handles,
            thread: None,
        }
    }

    pub fn terminate(&mut self) {
        if let Some(thread) = self.thread {
            unsafe {
                for s in &mut self.screens {
                    s.send_terminate();
                }

                println!("Terminate");
                WaitForSingleObject(thread.as_ptr(), INFINITE);
                println!("Waited");
            }

            self.thread = None;
        }
    }

    pub fn run(&mut self) {
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

    fn run_internal(&mut self) {
        self.desktop_handles.redraw();

        let hinstance = NonNull::new(unsafe { GetModuleHandleW(ptr::null_mut()) })
            .expect("Failed to retrieve module handle");
        let target = self.desktop_handles.get_target_parent();

        for s in self.screens.iter_mut() {
            s.spawn_target(hinstance, target);
        }

        unsafe { event_loop::enter_loop() }
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
