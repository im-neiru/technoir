// #[cfg(target_family = "windows")]
// mod windows;

mod config;
mod entry;
mod graphics;
mod manager;
mod utils;

pub mod plugin;
pub mod wallpaper;

pub use config::Config;
pub use entry::Entry;

// #[cfg(target_family = "windows")]
// pub use windows::run;
