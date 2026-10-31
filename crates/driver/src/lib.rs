#[cfg(target_os = "windows")]
mod win32;

pub struct WallpaperWindow(Inner);

enum Inner {
    #[cfg(target_os = "windows")]
    Win32(win32::WallpaperWindow),
}
