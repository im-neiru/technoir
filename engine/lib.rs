#[cfg(target_family = "windows")]
mod windows;

mod config;
mod renderers;
mod samplers;
mod screen_bounds;
mod wallpaper;

pub use config::Config;
pub use renderers::{Renderer, Visualizer};
pub use samplers::AudioLoopback;
pub use screen_bounds::ScreenBounds;

#[cfg(target_family = "windows")]
pub use windows::run;

pub use samplers::SpectrumAudioLoopback;
