use core::{ffi::c_void, ptr::NonNull};

use crate::wallpaper::Screen;

pub struct WindowsDriver {
    window: Option<NonNull<c_void>>,
    screens: Vec<Screen>,
}

impl WindowsDriver {
    pub(crate) fn new() -> Self {
        let screens = Screen::get_screens();

        Self {
            window: None,
            screens,
        }
    }
}
