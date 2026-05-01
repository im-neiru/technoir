use core::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use raw_window_handle as rwh;

use crate::wallpaper::WallpaperDriver;

pub struct Entry {
    wallpaper_driver: WallpaperDriver,
    wgpu_instance: wgpu::Instance,
    hinstance: NonNull<c_void>,
}

impl Entry {
    pub async fn new() -> Self {
        let hinstance = unsafe {
            NonNull::new(windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(
                ptr::null_mut(),
            ))
        }
        .expect("Failed to retrieve module handle");

        let wgpu_instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds {
                for_resource_creation: None,
                for_device_loss: None,
            },
            backend_options: wgpu::BackendOptions::default(),
            display: Some(Box::new(DisplayHandle)),
        });

        let wallpaper_driver = WallpaperDriver::new(&wgpu_instance, hinstance).await;

        Self {
            wallpaper_driver,
            wgpu_instance,
            hinstance,
        }
    }

    pub fn run(mut self) {
        self.wallpaper_driver.run();
    }
}

#[derive(Debug)]
struct DisplayHandle;

impl rwh::HasDisplayHandle for DisplayHandle {
    fn display_handle(&self) -> Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
        Ok(rwh::DisplayHandle::windows())
    }
}
