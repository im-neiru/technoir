#[cfg(target_family = "windows")]
mod desktop_handles;
#[cfg(target_family = "windows")]
mod desktop_watcher;

#[cfg(target_family = "windows")]
pub(crate) use desktop_handles::{DesktopHandles, get_desktop_handles};

#[cfg(target_family = "windows")]
pub use desktop_watcher::DesktopWatcher;
