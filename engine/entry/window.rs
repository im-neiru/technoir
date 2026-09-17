use core::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use raw_window_handle as rwh;

use crate::{manager::Manager, wallpaper::WallpaperDriver};

pub struct Entry {
    pub(crate) hinstance: NonNull<c_void>,
    pub(crate) wallpaper_driver: WallpaperDriver,
    pub(crate) manager: Manager,
}

impl Entry {
    pub async fn new(config: crate::Config) -> Self {
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

        let wallpaper_driver = WallpaperDriver::new(wgpu_instance.clone()).await;
        let manager = Manager::new(hinstance, config.open_manager);

        Self {
            wallpaper_driver,
            hinstance,
            manager,
        }
    }

    pub fn run(mut self) {
        self.wallpaper_driver.run();
        crate::manager::enter_loop(self);
    }
}

#[derive(Debug)]
struct DisplayHandle;

impl rwh::HasDisplayHandle for DisplayHandle {
    fn display_handle(&self) -> Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
        Ok(rwh::DisplayHandle::windows())
    }
}
