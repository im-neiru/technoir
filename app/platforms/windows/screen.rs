use core::{
    ffi::c_void,
    mem,
    ptr::{self, NonNull},
};

use windows_sys::{
    Win32::{
        Devices::Display::*,
        Foundation::{ERROR_CLASS_ALREADY_EXISTS, GetLastError, RECT, TRUE},
        Graphics::Gdi::*,
        UI::WindowsAndMessaging::*,
    },
    core::{BOOL, PCWSTR},
    w,
};

use common::ScreenBounds;

use super::event_loop::window_proc;

#[derive(Debug)]
pub struct Screen {
    hmonitor: HMONITOR,
    name: String,
    bounds: ScreenBounds,
    target: Option<WallpaperTarget>,
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
                hmonitor,
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

    pub fn spawn_target(&mut self, hinstance: NonNull<c_void>, parent: NonNull<c_void>) {
        self.target = Some(WallpaperTarget::new(
            &self.name,
            &self.bounds,
            hinstance,
            parent,
        ));
    }
}

#[derive(Debug)]
pub struct WallpaperTarget {
    hwnd: NonNull<c_void>,
    hinstance: NonNull<c_void>,
    classname: [u16; 96],
}

impl WallpaperTarget {
    fn new(
        screen_name: &str,
        bounds: &ScreenBounds,
        hinstance: NonNull<c_void>,
        parent: NonNull<c_void>,
    ) -> Self {
        unsafe {
            let classname = Self::build_classname(screen_name);

            let wnd_class = WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(window_proc),
                cbClsExtra: 0,
                cbWndExtra: 0,
                hInstance: hinstance.as_ptr(),
                hIcon: ptr::null_mut(),
                hCursor: LoadCursorW(ptr::null_mut(), IDC_ARROW),
                hbrBackground: ptr::null_mut(),
                lpszMenuName: ptr::null_mut(),
                lpszClassName: classname.as_ptr(),
            };

            if RegisterClassW(&wnd_class) == 0 {
                let err = GetLastError();
                if err != ERROR_CLASS_ALREADY_EXISTS {
                    panic!("Failed to register window class: {}", err);
                }
            }

            let Some(hwnd) = NonNull::new(CreateWindowExW(
                WS_EX_LAYERED | WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE,
                classname.as_ptr(),
                classname.as_ptr(),
                WS_POPUP | WS_VISIBLE,
                0,
                0,
                100,
                100,
                parent.as_ptr(),
                ptr::null_mut(),
                hinstance.as_ptr(),
                ptr::null_mut(),
            )) else {
                let err = GetLastError();
                panic!("Failed to create window: {}", err);
            };

            SetLayeredWindowAttributes(hwnd.as_ptr(), 0, 255, LWA_ALPHA);

            SetParent(hwnd.as_ptr(), parent.as_ptr());

            SetWindowPos(
                hwnd.as_ptr(),
                HWND_TOP,
                bounds.x,
                bounds.y,
                bounds.width as i32,
                bounds.height as i32,
                SWP_SHOWWINDOW | SWP_NOACTIVATE,
            );

            Self {
                hwnd,
                hinstance,
                classname,
            }
        }
    }

    fn build_classname(screen_name: &str) -> [u16; 96] {
        let base = "TechNoirTarget ";

        let mut buf = [0u16; 96];
        let mut i = 0;

        for ch in base.encode_utf16() {
            if i >= buf.len() - 1 {
                break;
            }
            buf[i] = ch;
            i += 1;
        }

        for ch in screen_name.encode_utf16() {
            if i >= buf.len() - 1 {
                break;
            }
            buf[i] = ch;
            i += 1;
        }

        buf[i] = 0;

        buf
    }
}

impl Drop for WallpaperTarget {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.hwnd.as_ptr());
            UnregisterClassW(self.classname.as_ptr(), self.hinstance.as_ptr());
        }
    }
}
