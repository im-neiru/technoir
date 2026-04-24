use core::ptr::{self, NonNull};

use windows_sys::Win32::System::LibraryLoader::GetModuleHandleW;

use super::manager::Manager;

pub struct State {
    pub(super) manager: Manager,
}

impl State {
    pub(super) fn new(config: &crate::config::Config) -> Self {
        let hinstance = unsafe { NonNull::new(GetModuleHandleW(ptr::null_mut())) }
            .expect("Failed to retrieve module handle");

        let manager = Manager::new(hinstance, config.open_manager);

        Self { manager }
    }

    pub(super) fn enter_loop(self) {
        super::event_loop::enter_loop(self);
    }
}
