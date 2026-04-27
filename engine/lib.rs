#[cfg(target_family = "windows")]
mod windows;

mod config;
mod renderer;
mod screen_bounds;

pub use config::Config;
pub use renderer::Renderer;
pub use screen_bounds::ScreenBounds;

#[cfg(target_family = "windows")]
pub use windows::run;
