use core::{
    ffi::c_void,
    ptr::{self, NonNull},
};

use raw_window_handle as rwh;

use crate::{
    manager::Manager,
    plugin::{PluginLoader, package::Packager},
    wallpaper::WallpaperDriver,
};

pub struct Entry {
    pub(crate) hinstance: NonNull<c_void>,
    pub(crate) wallpaper_driver: WallpaperDriver,
    pub(crate) manager: Manager,
    pub(crate) plugin_loader: PluginLoader,
}

impl Entry {
    pub async fn new(config: crate::Config) -> Self {
        let hinstance = unsafe {
            NonNull::new(windows_sys::Win32::System::LibraryLoader::GetModuleHandleW(
                ptr::null_mut(),
            ))
        }
        .expect("Failed to retrieve module handle");

        let manager = Manager::new(hinstance, config.open_manager);
        let plugin_loader = PluginLoader::new();

        let wallpaper_driver = WallpaperDriver::new(manager.graphics(), &plugin_loader).await;

        Self {
            wallpaper_driver,
            hinstance,
            manager,
            plugin_loader,
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
