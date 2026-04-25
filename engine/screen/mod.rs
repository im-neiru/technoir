mod info;

pub use info::{ScreenBounds, ScreenSize};

use core::{
    ffi::c_void,
    mem,
    ptr::{self, NonNull},
};

use windows_sys::{
    Win32::{
        Foundation::{RECT, TRUE},
        Graphics::Gdi::{EnumDisplayMonitors, GetMonitorInfoW, HDC, HMONITOR, MONITORINFOEXW},
    },
    core::BOOL,
};

#[derive(Debug)]
pub struct Screen {
    pub name: String,
    pub physical_size: ScreenSize,
    pub virtual_bounds: ScreenBounds,
    pub target: Option<WallpaperTarget>,
}

#[derive(Debug)]
pub struct WallpaperTarget {
    hwnd: NonNull<c_void>,
}

impl Screen {
    pub fn get_screens() -> Vec<Screen> {
        let mut screens: Vec<Screen> = Vec::new();

        unsafe {
            EnumDisplayMonitors(
                ptr::null_mut(),
                ptr::null(),
                Some(Self::monitor_enum_proc),
                &mut screens as *mut Vec<Screen> as isize,
            );
        }

        let mut min_x = i32::MAX;
        let mut min_y = i32::MAX;

        for screen in &screens {
            min_x = min_x.min(screen.virtual_bounds.x);
            min_y = min_y.min(screen.virtual_bounds.y);
        }

        if !screens.is_empty() {
            for screen in &mut screens {
                screen.virtual_bounds.x -= min_x;
                screen.virtual_bounds.y -= min_y;
            }
        }

        screens
    }

    unsafe extern "system" fn monitor_enum_proc(
        hmonitor: HMONITOR,
        _hdc: HDC,
        _rect: *mut RECT,
        lparam: isize,
    ) -> BOOL {
        let screens = unsafe { &mut *(lparam as *mut Vec<Screen>) };
        let mut info: MONITORINFOEXW = unsafe { mem::zeroed() };

        info.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;

        if unsafe { GetMonitorInfoW(hmonitor, &mut info as *mut _ as _) } != 0 {
            let name = String::from_utf16_lossy(&info.szDevice)
                .trim_matches('\0')
                .to_string();

            let rect = info.monitorInfo.rcMonitor;
            let width = (rect.right - rect.left).unsigned_abs();
            let height = (rect.bottom - rect.top).unsigned_abs();

            screens.push(Screen {
                name,
                physical_size: ScreenSize { width, height },
                virtual_bounds: ScreenBounds {
                    x: rect.left,
                    y: rect.top,
                    width,
                    height,
                },
                target: None,
            });
        }

        TRUE
    }
}
