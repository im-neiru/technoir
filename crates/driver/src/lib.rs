#[cfg(target_os = "windows")]
mod win32;

pub struct WallpaperWindow(WallpaperWindowInner);

enum WallpaperWindowInner {
    #[cfg(target_os = "windows")]
    Win32(win32::WallpaperWindow),
}

pub struct UiWindow(UiWindowInner);

enum UiWindowInner {
    #[cfg(target_os = "windows")]
    Win32(win32::UiWindow),
}
