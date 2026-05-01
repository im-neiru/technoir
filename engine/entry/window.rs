use core::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use raw_window_handle as rwh;

use crate::wallpaper::WindowsDriver;

pub struct Entry {
    driver: WindowsDriver,
    wgpu_instance: wgpu::Instance,
    hinstance: NonNull<c_void>,
}

impl Entry {
    pub fn new() -> Self {
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

        Self {
            driver: WindowsDriver::new(),
            wgpu_instance,
            hinstance,
        }
    }
}

#[derive(Debug)]
struct DisplayHandle;

impl rwh::HasDisplayHandle for DisplayHandle {
    fn display_handle(&self) -> Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
        Ok(rwh::DisplayHandle::windows())
    }
}
