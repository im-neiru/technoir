#[cfg(target_family = "windows")]
mod desktop_handles;
#[cfg(target_family = "windows")]
mod desktop_watcher;

mod bin_helper;
mod directories;
mod program_data_path;
mod read_mapped_file;

#[cfg(target_family = "windows")]
pub(crate) use desktop_handles::{DesktopHandles, get_desktop_handles};

#[cfg(target_family = "windows")]
pub use desktop_watcher::DesktopWatcher;

pub(crate) use bin_helper::*;
pub(crate) use directories::get_plugin_directory;
pub(crate) use program_data_path::program_data_path;
pub(crate) use read_mapped_file::ReadMappedFile;
