#[cfg(target_family = "windows")]
mod windows;

#[cfg(target_family = "windows")]
pub(in crate::wallpaper) use windows::WallpaperTarget;
