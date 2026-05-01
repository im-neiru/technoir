mod screen_bounds;

#[cfg(target_family = "windows")]
mod windows;

pub use screen_bounds::ScreenBounds;

#[cfg(target_family = "windows")]
pub use windows::Screen;
