use core::ptr::{self, NonNull};

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

use crate::platforms::windows::event_loop;

use super::manager::Manager;

pub struct State {
    pub(super) wgpu_instance: wgpu::Instance,
    pub(super) manager: Manager,
}

impl State {
    pub(super) fn new(config: &crate::config::Config) -> Self {
        let hinstance = unsafe { NonNull::new(GetModuleHandleW(ptr::null_mut())) }
            .expect("Failed to retrieve module handle");

        let wgpu_instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            flags: wgpu::InstanceFlags::default(),
            memory_budget_thresholds: wgpu::MemoryBudgetThresholds {
                for_resource_creation: None,
                for_device_loss: None,
            },
            backend_options: wgpu::BackendOptions::default(),
            display: Some(Box::new(event_loop::WindowsDisplayHandle)),
        });

        let manager = Manager::new(hinstance, config.open_manager, &wgpu_instance);

        Self {
            manager,
            wgpu_instance,
        }
    }

    pub(super) fn enter_loop(self) {
        super::event_loop::enter_loop(self);
    }
}
