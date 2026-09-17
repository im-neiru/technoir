// #[cfg(target_family = "windows")]
// mod windows;

mod config;
mod entry;
mod graphics;
mod manager;
mod plugin;
mod samplers;
mod utils;

pub mod wallpaper;

pub use config::Config;
pub use entry::Entry;
pub use samplers::AudioLoopback;

// #[cfg(target_family = "windows")]
// pub use windows::run;

pub use samplers::SpectrumAudioLoopback;
