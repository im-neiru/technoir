use core::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

use super::manager::Manager;
use super::wallpaper_driver::WallpaperDriver;

pub struct State {
    pub(super) hinstance: NonNull<c_void>,
    pub(super) wgpu_instance: wgpu::Instance,
    pub(super) manager: Manager,
    pub(super) driver: WallpaperDriver,
}

impl State {
    pub(super) fn new(config: &crate::config::Config) -> Self {
        let hinstance = unsafe { NonNull::new(GetModuleHandleW(ptr::null_mut())) }
            .expect("Failed to retrieve module handle");

        let wgpu_instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds {
                for_resource_creation: None,
                for_device_loss: None,
            },
            backend_options: wgpu::BackendOptions::default(),
        });

        let manager = Manager::new(hinstance, config.open_manager);

        let driver = WallpaperDriver::new(&wgpu_instance);

        Self {
            hinstance,
            manager,
            wgpu_instance,
            driver,
        }
    }

    pub(super) async fn enter_ui(mut self) {
        self.driver.run();
        self.manager.init_graphics(&self.wgpu_instance).await;
        super::manager::enter_loop(self);
    }
}
