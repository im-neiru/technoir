#[cfg(target_family = "windows")]
mod desktop_handles;
#[cfg(target_family = "windows")]
mod desktop_watcher;

mod read_mapped_file;

#[cfg(target_family = "windows")]
pub(crate) use desktop_handles::{DesktopHandles, get_desktop_handles};

#[cfg(target_family = "windows")]
pub use desktop_watcher::DesktopWatcher;

pub(crate) use read_mapped_file::ReadMappedFile;
