// #[cfg(target_family = "windows")]
// mod windows;

mod config;
mod entry;
mod manager;
mod renderers;
mod samplers;
mod utils;

pub mod wallpaper;

pub use config::Config;
pub use entry::Entry;
pub use renderers::{Renderer, Visualizer};
pub use samplers::AudioLoopback;

// #[cfg(target_family = "windows")]
// pub use windows::run;

pub use samplers::SpectrumAudioLoopback;
