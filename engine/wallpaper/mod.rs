mod driver;
mod event_loop;
mod screen;
mod wallpaper_target;

pub(crate) use driver::WallpaperDriver;
pub(in crate::wallpaper) use screen::Screen;
