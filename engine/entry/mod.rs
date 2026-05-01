#[cfg(target_family = "windows")]
mod window;

#[cfg(target_family = "windows")]
pub use window::Entry;
