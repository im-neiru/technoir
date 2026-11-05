use core::{ffi::c_void, ptr::NonNull};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Monitor {
    pub(super) handle: MonitorHandle,
    pub(super) name: Box<str>,
    pub(super) bounds: MonitorBounds,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct MonitorBounds {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct MonitorHandle(pub(super) MonitorHandleValue);

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub(super) enum MonitorHandleValue {
    #[cfg(target_family = "windows")]
    Win32(NonNull<c_void>),
}
