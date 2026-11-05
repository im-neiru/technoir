use core::{
    mem,
    ptr::{self, NonNull},
};

use windows_sys::{
    Win32::{
        Devices::Display::*,
        Foundation::{RECT, TRUE},
        Graphics::Gdi::*,
    },
    core::BOOL,
};

use crate::monitor::{Monitor, MonitorBounds, MonitorHandle, MonitorHandleValue};

impl Monitor {
    pub fn get_monitors() -> Box<[Self]> {
        let mut monitor: Vec<Self> = Vec::new();

        unsafe {
            EnumDisplayMonitors(
                ptr::null_mut(),
                ptr::null(),
                Some(monitor_enum_proc),
                &mut monitor as *mut Vec<Self> as isize,
            );
        }

        monitor.into_boxed_slice()
    }

    #[inline(always)]
    pub const fn name(&self) -> &str {
        &self.name
    }

    #[inline(always)]
    pub const fn bounds(&self) -> MonitorBounds {
        self.bounds
    }
}

unsafe extern "system" fn monitor_enum_proc(
    hmonitor: HMONITOR,
    _hdc: HDC,
    _rect: *mut RECT,
    lparam: isize,
) -> BOOL {
    let Some(handle) = NonNull::new(hmonitor) else {
        return TRUE;
    };

    let handle = MonitorHandle(MonitorHandleValue::Win32(handle));
    let monitors = unsafe { &mut *(lparam as *mut Vec<Monitor>) };

    let Some(info) = query_monitor_info(hmonitor) else {
        return TRUE;
    };

    let rect = info.monitorInfo.rcMonitor;

    monitors.push(Monitor {
        handle,
        name: monitor_name(&info.szDevice).into_boxed_str(),
        bounds: MonitorBounds {
            x: rect.left,
            y: rect.top,
            width: (rect.right - rect.left).unsigned_abs(),
            height: (rect.bottom - rect.top).unsigned_abs(),
        },
    });

    TRUE
}

#[inline]
fn query_monitor_info(hmonitor: HMONITOR) -> Option<MONITORINFOEXW> {
    let mut info: MONITORINFOEXW = unsafe { mem::zeroed() };
    info.monitorInfo.cbSize = mem::size_of::<MONITORINFOEXW>() as u32;

    if unsafe { GetMonitorInfoW(hmonitor, &mut info as *mut _ as _) } == 0 {
        return None;
    }

    Some(info)
}

fn monitor_name(gdi_device_name: &GdiDeviceName) -> String {
    if let Some(name) = friendly_name(gdi_device_name) {
        return name;
    }

    if let Some(name) = display_device_string(gdi_device_name) {
        return name;
    }

    utf16_until_nul(gdi_device_name).unwrap_or_default()
}

fn friendly_name(gdi_device_name: &GdiDeviceName) -> Option<String> {
    let paths = active_display_paths()?;

    for path in &paths {
        let Some(source_name) = source_gdi_name(path) else {
            continue;
        };

        if until_nul(&source_name) != until_nul(gdi_device_name) {
            continue;
        }

        if let Some(name) = target_friendly_name(path) {
            return Some(name);
        }
    }

    None
}

fn active_display_paths() -> Option<Vec<DISPLAYCONFIG_PATH_INFO>> {
    let mut path_count = 0;
    let mut mode_count = 0;

    unsafe {
        if GetDisplayConfigBufferSizes(QDC_ONLY_ACTIVE_PATHS, &mut path_count, &mut mode_count) != 0
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

        paths.truncate(path_count as usize);

        Some(paths)
    }
}

fn source_gdi_name(path: &DISPLAYCONFIG_PATH_INFO) -> Option<GdiDeviceName> {
    let mut source_info: DISPLAYCONFIG_SOURCE_DEVICE_NAME = unsafe { mem::zeroed() };

    source_info.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_SOURCE_NAME;
    source_info.header.size = mem::size_of::<DISPLAYCONFIG_SOURCE_DEVICE_NAME>() as u32;
    source_info.header.adapterId = path.sourceInfo.adapterId;
    source_info.header.id = path.sourceInfo.id;

    if unsafe { DisplayConfigGetDeviceInfo(&mut source_info.header) } != 0 {
        return None;
    }

    Some(source_info.viewGdiDeviceName)
}

fn target_friendly_name(path: &DISPLAYCONFIG_PATH_INFO) -> Option<String> {
    let mut target_info: DISPLAYCONFIG_TARGET_DEVICE_NAME = unsafe { mem::zeroed() };

    target_info.header.r#type = DISPLAYCONFIG_DEVICE_INFO_GET_TARGET_NAME;
    target_info.header.size = mem::size_of::<DISPLAYCONFIG_TARGET_DEVICE_NAME>() as u32;
    target_info.header.adapterId = path.targetInfo.adapterId;
    target_info.header.id = path.targetInfo.id;

    if unsafe { DisplayConfigGetDeviceInfo(&mut target_info.header) } != 0 {
        return None;
    }

    utf16_until_nul(&target_info.monitorFriendlyDeviceName)
}

fn display_device_string(gdi_device_name: &GdiDeviceName) -> Option<String> {
    let mut terminated_name = *gdi_device_name;

    terminated_name[GDI_DEVICE_NAME_CAPACITY - 1] = 0;

    let mut device_info: DISPLAY_DEVICEW = unsafe { mem::zeroed() };

    device_info.cb = mem::size_of::<DISPLAY_DEVICEW>() as u32;

    if unsafe { EnumDisplayDevicesW(terminated_name.as_ptr(), 0, &mut device_info, 0) } == 0 {
        return None;
    }

    utf16_until_nul(&device_info.DeviceString)
}

const GDI_DEVICE_NAME_CAPACITY: usize = 32;
type GdiDeviceName = [u16; GDI_DEVICE_NAME_CAPACITY];

#[inline]
fn until_nul(buffer: &[u16]) -> &[u16] {
    match buffer.iter().position(|&code_unit| code_unit == 0) {
        Some(nul_index) => &buffer[..nul_index],
        None => buffer,
    }
}

#[inline]
fn utf16_until_nul(buffer: &[u16]) -> Option<String> {
    let text = until_nul(buffer);

    if text.is_empty() {
        return None;
    }

    Some(String::from_utf16_lossy(text))
}
