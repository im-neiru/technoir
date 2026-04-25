use core::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use vello::wgpu;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

use super::manager::Manager;
use engine::Screen;

pub struct State {
    pub(super) hinstance: NonNull<c_void>,
    pub(super) wgpu_instance: wgpu::Instance,
    pub(super) manager: Manager,
    pub(super) screens: Vec<Screen>,
}

impl State {
    pub(super) async fn new(config: &crate::config::Config) -> Self {
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

        let manager = Manager::new(hinstance, config.open_manager, &wgpu_instance).await;

        Self {
            hinstance,
            manager,
            wgpu_instance,
            screens: Screen::get_screens(),
        }
    }

    pub(super) fn enter_loop(self) {
        super::event_loop::enter_loop(self);
    }
}
