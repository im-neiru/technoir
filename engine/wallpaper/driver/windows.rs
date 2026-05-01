use core::{ffi::c_void, ptr::NonNull};

pub struct WindowsDriver {
    thread: Option<NonNull<c_void>>,
    wgpu_instance: wgpu::Instance,
}

impl super::template::WallpaperDriver for WindowsDriver {
    fn run(&mut self) {}

    fn terminate(&mut self) {}
}
