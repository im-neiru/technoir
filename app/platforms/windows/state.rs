use core::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use vello::wgpu;
use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

use super::{desktop_handles::DesktopHandles, manager::Manager, screen::Screen};

pub struct State {
    pub(super) hinstance: NonNull<c_void>,
    pub(super) wgpu_instance: wgpu::Instance,
    pub(super) manager: Manager,
    pub(super) screens: Vec<Screen>,
    pub(super) desktop_handles: DesktopHandles,
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
        let mut screens = Screen::get_screens();
        let desktop_handles = DesktopHandles::find();

        let target_parent = desktop_handles.worker_w.unwrap_or(desktop_handles.progman);

        for s in screens.iter_mut() {
            s.spawn_target(hinstance, target_parent);
        }

        Self {
            hinstance,
            manager,
            wgpu_instance,
            screens,
            desktop_handles,
        }
    }

    pub(super) fn enter_loop(self) {
        self.desktop_handles.redraw();
        super::event_loop::enter_loop(self);
    }
}
