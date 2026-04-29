use core::{
    ffi::c_void,
    mem,
    num::NonZeroUsize,
    ptr::{self, NonNull},
    sync::atomic::{AtomicU8, AtomicU64, Ordering, fence},
};

use crossbeam_queue::SegQueue;
use windows_sys::Win32::{
    Foundation::{FALSE, HWND, POINT, RECT, TRUE},
    Graphics::{
        Dwm::{DWMWA_CLOAKED, DwmGetWindowAttribute},
        Gdi::{
            GetMonitorInfoW, HMONITOR, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow,
        },
    },
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
        WindowsAndMessaging::{
            EVENT_OBJECT_LOCATIONCHANGE, EVENT_SYSTEM_FOREGROUND, GWL_EXSTYLE, GWL_STYLE,
            GetForegroundWindow, GetWindowLongW, GetWindowPlacement, GetWindowRect, IsIconic,
            IsWindowVisible, SW_SHOWMAXIMIZED, WINDOWPLACEMENT, WINEVENT_OUTOFCONTEXT,
            WINEVENT_SKIPOWNPROCESS, WS_CAPTION, WS_EX_TOOLWINDOW, WS_POPUP, WindowFromPoint,
        },
    },
};

static USER_COUNT: AtomicU8 = AtomicU8::new(0);
static HOOK: AtomicU64 = AtomicU64::new(0);
static LAST_STATE: AtomicU64 = AtomicU64::new(0);
static QUEUE: SegQueue<WatcherEntry> = SegQueue::new();

pub struct WatcherEntry {
    pub window: NonZeroUsize,
    pub monitor: NonZeroUsize,
    pub is_full: bool,
}

pub struct WatcherGuard;

pub fn start_watching() -> Option<WatcherGuard> {
    let mut current = USER_COUNT.load(Ordering::Acquire);
    while current > 0 {
        if current == u8::MAX {
            std::process::abort();
        }
        match USER_COUNT.compare_exchange_weak(
            current,
            current + 1,
            Ordering::Relaxed,
            Ordering::Acquire,
        ) {
            Ok(_) => return Some(WatcherGuard),
            Err(actual) => current = actual,
        }
    }

    let hook = unsafe {
        SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_OBJECT_LOCATIONCHANGE,
            ptr::null_mut(),
            Some(desktop_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        )
    };

    if hook.is_null() {
        return None;
    }

    if HOOK
        .compare_exchange(0, hook as u64, Ordering::Release, Ordering::Relaxed)
        .is_err()
    {
        unsafe { UnhookWinEvent(hook) };
    }

    USER_COUNT.fetch_add(1, Ordering::Relaxed);
    Some(WatcherGuard)
}

impl WatcherGuard {
    pub fn poll(&self) -> Option<WatcherEntry> {
        QUEUE.pop()
    }
}

impl Drop for WatcherGuard {
    fn drop(&mut self) {
        if USER_COUNT.fetch_sub(1, Ordering::Release) != 1 {
            return;
        }
        fence(Ordering::Acquire);

        let hook = HOOK.swap(0, Ordering::Relaxed) as HWINEVENTHOOK;

        if hook.is_null() {
            unsafe { UnhookWinEvent(hook) };
        }
    }
}

impl WatcherEntry {
    #[inline]
    pub fn monitor_handle(&self) -> NonNull<c_void> {
        unsafe { mem::transmute(self.monitor) }
    }

    #[inline]
    pub fn window_handle(&self) -> NonNull<c_void> {
        unsafe { mem::transmute(self.window) }
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.is_full
    }
}

struct MonitorState {
    target_hmonitor: HMONITOR,
    monitor_info: MONITORINFO,
    is_full: bool,
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe extern "system" fn enum_windows_proc(hwnd: HWND, lparam: isize) -> i32 {
    let state = &mut *(lparam as *mut MonitorState);

    if IsWindowVisible(hwnd) == FALSE || IsIconic(hwnd) == TRUE {
        return 1;
    }

    let style = GetWindowLongW(hwnd, GWL_STYLE) as u32;
    let ex_style = GetWindowLongW(hwnd, GWL_EXSTYLE) as u32;

    if (ex_style & WS_EX_TOOLWINDOW) != 0 {
        return 1;
    }

    let is_standard_app = (style & WS_CAPTION) != 0;
    let is_fullscreen_game = (style & WS_POPUP) != 0 && (style & WS_CAPTION) == 0;

    if !is_standard_app && !is_fullscreen_game {
        return 1;
    }

    let mut cloaked: i32 = 0;
    DwmGetWindowAttribute(
        hwnd,
        DWMWA_CLOAKED as _,
        &mut cloaked as *mut _ as *mut _,
        4,
    );
    if cloaked != 0 {
        return 1;
    }

    let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
    if hmonitor != state.target_hmonitor {
        return 1;
    }

    let mut rect: RECT = mem::zeroed();
    if GetWindowRect(hwnd, &mut rect) == FALSE {
        return 1;
    }

    let area = state.monitor_info.rcMonitor;
    let work = state.monitor_info.rcWork;
    let margin = 10;

    let fills_monitor = rect.left <= area.left + margin
        && rect.top <= area.top + margin
        && rect.right >= area.right - margin
        && rect.bottom >= area.bottom - margin;

    let fills_work = rect.left <= work.left + margin
        && rect.top <= work.top + margin
        && rect.right >= work.right - margin
        && rect.bottom >= work.bottom - margin;

    if fills_monitor || fills_work {
        state.is_full = true;
        return 0;
    }

    1
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe extern "system" fn desktop_callback(
    _hwineventhook: HWINEVENTHOOK,
    _event: u32,
    hwnd: HWND,
    _idobject: i32,
    _idchild: i32,
    _ideventthread: u32,
    _dwmseventtime: u32,
) {
    if hwnd.is_null() {
        return;
    }

    let hmonitor = MonitorFromWindow(hwnd, MONITOR_DEFAULTTONEAREST);
    let mut mi: MONITORINFO = mem::zeroed();
    mi.cbSize = mem::size_of::<MONITORINFO>() as u32;
    if GetMonitorInfoW(hmonitor, &mut mi) == FALSE {
        return;
    }

    let mut state = MonitorState {
        target_hmonitor: hmonitor,
        monitor_info: mi,
        is_full: false,
    };

    windows_sys::Win32::UI::WindowsAndMessaging::EnumWindows(
        Some(enum_windows_proc),
        &mut state as *mut _ as isize,
    );

    let m_addr = hmonitor as u64;
    let current_state = (m_addr & !1) | (state.is_full as u64);

    if LAST_STATE.swap(current_state, Ordering::SeqCst) != current_state {
        QUEUE.push(WatcherEntry {
            window: NonZeroUsize::new_unchecked(hwnd as usize),
            monitor: NonZeroUsize::new_unchecked(hmonitor as usize),
            is_full: state.is_full,
        });
    }
}
