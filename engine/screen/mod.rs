mod info;

#[cfg(windows)]
mod windows;

pub use info::{ScreenBounds, ScreenSize};

#[cfg(windows)]
pub use windows::WallpaperTarget;

#[derive(Debug)]
pub struct Screen {
    pub name: String,
    pub physical_size: ScreenSize,
    pub virtual_bounds: ScreenBounds,
    pub target: Option<WallpaperTarget>,
}
