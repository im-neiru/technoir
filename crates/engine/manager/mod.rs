#[cfg(target_family = "windows")]
mod windows;

#[cfg(target_family = "windows")]
pub use windows::Manager;

pub(crate) use windows::enter_loop;
