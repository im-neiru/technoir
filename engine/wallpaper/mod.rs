mod driver;
mod event_loop;
mod renderer;
mod screen;
mod wallpaper_target;

pub(crate) use driver::WallpaperDriver;
pub(crate) use screen::{Screen, ScreenBounds};

use wallpaper_target::WallpaperTarget;
