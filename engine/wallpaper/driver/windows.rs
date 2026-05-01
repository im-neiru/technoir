use core::{ffi::c_void, ptr::NonNull};

use crate::wallpaper::Screen;

pub struct WallpaperDriver {
    window: Option<NonNull<c_void>>,
    screens: Vec<Screen>,
}

impl WallpaperDriver {
    pub(crate) fn new() -> Self {
        let screens = Screen::get_screens();

        Self {
            window: None,
            screens,
        }
    }
}
