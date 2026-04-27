use core::{
    ffi::c_void,
    mem,
    ptr::{self, NonNull},
};

use vello::wgpu;
use windows_sys::{
    Win32::{
        Devices::Display::*,
        Foundation::{RECT, TRUE},
        Graphics::Gdi::*,
        UI::WindowsAndMessaging::{GWL_USERDATA, PostMessageW, SetWindowLongPtrA},
    },
    core::BOOL,
};

use crate::{screen_bounds::ScreenBounds, windows::messages::WM_APP_TERMINATE};

use super::target::WallpaperTarget;

pub struct Screen {
    _hmonitor: HMONITOR,
    name: String,
    bounds: ScreenBounds,
    pub(super) target: Option<WallpaperTarget>,
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
            min_x = min_x.min(screen.bounds.x);
            min_y = min_y.min(screen.bounds.y);
        }

        if !screens.is_empty() {
            for screen in &mut screens {
                screen.bounds.x -= min_x;
                screen.bounds.y -= min_y;
            }
        }

        screens
    }

    pub(super) fn store_state(&mut self) {
        unsafe {
            if let Some(target) = &self.target {
                SetWindowLongPtrA(
                    target.hwnd.as_ptr(),
                    GWL_USERDATA,
                    NonNull::from(target).addr().cast_signed().get(),
                );
            }
        };
    }

    fn get_friendly_name(device_name: &[u16; 32]) -> Option<String> {
        let mut path_count = 0;
        let mut mode_count = 0;

        unsafe {
            if GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count)
                != 0
            {
                return None;
            }

            let mut paths = vec![mem::zeroed::<DISPLAYCONFIG_PATH_INFO>(); path_count as usize];
            let mut modes = vec![mem::zeroed::<DISPLAYCONFIG_MODE_INFO>(); mode_count as usize];

            if QueryDisplayConfig(
                QDC_ONLY_ACTIVE_PATHS,
                &mut path_count,
                paths.as_mut_ptr(),
                &mut mode_count,
                modes.as_mut_ptr(),
                ptr::null_mut(),
            ) != 0
            {
                return None;
            }

            for path in paths.iter().take(path_count as usize) {
                let mut source_info: DISPLAYCONFIG_SOURCE_DEVICE_NAME = mem::zeroed();
                source_info.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
                source_info.header.size = mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
                source_info.header.adapterId = path.sourceInfo.adapterId;
                source_info.header.id = path.sourceInfo.id;

                if DisplayConfigGetDeviceInfo(&mut source_info.header) == 0
                    && source_info
                        .viewGdiDeviceName
                        .iter()
                        .zip(device_name.iter())
                        .all(|(a, b)| a == b)
                {
                    let mut target_info: DISPLAYCONFIG_TARGET_DEVICE_NAME = mem::zeroed();
                    target_info.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
                    target_info.header.size =
                        mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
                    target_info.header.adapterId = path.targetInfo.adapterId;
                    target_info.header.id = path.targetInfo.id;

                    if DisplayConfigGetDeviceInfo(&mut target_info.header) == 0 {
                        return Some(
                            String::from_utf16_lossy(&target_info.monitorFriendlyDeviceName)
                                .trim_matches('\0')
                                .to_string(),
                        );
                    }
                }
            }
        }
        None
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
            let name = if let Some(friendly_name) = Self::get_friendly_name(&info.szDevice) {
                friendly_name
            } else {
                let mut device_info: DISPLAY_DEVICEW = unsafe { mem::zeroed() };
                device_info.cb = mem::size_of::<DISPLAY_DEVICEW>() as u32;

                if unsafe { EnumDisplayDevicesW(info.szDevice.as_ptr(), 0, &mut device_info, 0) }
                    != 0
                {
                    String::from_utf16_lossy(&device_info.DeviceString)
                        .trim_matches('\0')
                        .to_string()
                } else {
                    String::from_utf16_lossy(&info.szDevice)
                        .trim_matches('\0')
                        .to_string()
                }
            };

            let rect = info.monitorInfo.rcMonitor;
            let width = (rect.right - rect.left).unsigned_abs();
            let height = (rect.bottom - rect.top).unsigned_abs();

            screens.push(Screen {
                _hmonitor: hmonitor,
                name,
                bounds: ScreenBounds {
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

    pub fn spawn_target(
        &mut self,
        hinstance: NonNull<c_void>,
        parent: NonNull<c_void>,
        wgpu_instance: &wgpu::Instance,
    ) {
        smol::block_on(async {
            self.target = Some(
                WallpaperTarget::new(&self.name, &self.bounds, hinstance, parent, wgpu_instance)
                    .await,
            );
        });
    }

    #[inline]
    pub fn send_terminate(&self) {
        if let Some(target) = &self.target {
            unsafe { PostMessageW(target.hwnd.as_ptr(), WM_APP_TERMINATE, 0, 0) };
        }
    }
}
