mod screen_bounds;

#[cfg(target_family = "windows")]
mod windows;

pub(in crate::wallpaper) use screen_bounds::ScreenBounds;

#[cfg(target_family = "windows")]
pub(in crate::wallpaper) use windows::Screen;
