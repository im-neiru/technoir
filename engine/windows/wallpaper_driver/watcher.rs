use core::{
    ffi::c_void,
    mem,
    num::NonZeroUsize,
    ptr::{self, NonNull},
    sync::atomic::{AtomicU8, AtomicUsize, Ordering, fence},
};

use crossbeam_queue::SegQueue;
use windows_sys::Win32::{
    Foundation::{FALSE, HWND, TRUE},
    Graphics::Gdi::{GetMonitorInfoW, MONITOR_DEFAULTTONEAREST, MONITORINFO, MonitorFromWindow},
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
        WindowsAndMessaging::{
            EVENT_OBJECT_LOCATIONCHANGE, EVENT_SYSTEM_FOREGROUND, GetWindowRect, IsIconic,
            IsWindowVisible, WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
        },
    },
};

static USER_COUNT: AtomicU8 = AtomicU8::new(0);
static HOOK: AtomicUsize = AtomicUsize::new(0);
static LAST_HWND: AtomicUsize = AtomicUsize::new(0);
static QUEUE: SegQueue<WatcherEntry> = SegQueue::new();

pub struct WatcherEntry {
    window: NonZeroUsize,
    monitor: NonZeroUsize,
    is_full: bool,
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
        NonNull::new(SetWinEventHook(
            EVENT_SYSTEM_FOREGROUND,
            EVENT_OBJECT_LOCATIONCHANGE,
            ptr::null_mut(),
            Some(desktop_callback),
            0,
            0,
            WINEVENT_OUTOFCONTEXT | WINEVENT_SKIPOWNPROCESS,
        ))?
    };

    if HOOK
        .compare_exchange(0, hook.addr().get(), Ordering::Release, Ordering::Relaxed)
        .is_err()
    {
        unsafe { UnhookWinEvent(hook.as_ptr()) };
    }

    USER_COUNT.fetch_add(1, Ordering::Relaxed);

    Some(WatcherGuard)
}

impl WatcherGuard {
    #[inline]
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

        if !hook.is_null() {
            unsafe { UnhookWinEvent(hook) };
        }
    }
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
    let Some(hwnd) = NonNull::new(hwnd) else {
        return;
    };

    if IsWindowVisible(hwnd.as_ptr()) != TRUE || IsIconic(hwnd.as_ptr()) != TRUE {
        return;
    }

    let mut rect = core::mem::zeroed();
    if GetWindowRect(hwnd.as_ptr(), &mut rect) == FALSE {
        return;
    }

    let hmonitor = MonitorFromWindow(hwnd.as_ptr(), MONITOR_DEFAULTTONEAREST);
    let mut monitor_info: MONITORINFO = core::mem::zeroed();
    monitor_info.cbSize = core::mem::size_of::<MONITORINFO>() as u32;

    if GetMonitorInfoW(hmonitor, &mut monitor_info) == FALSE {
        return;
    }

    let work = monitor_info.rcWork;

    let is_full = rect.left <= work.left
        && rect.top <= work.top
        && rect.right >= work.right
        && rect.bottom >= work.bottom;

    let addr = hwnd.addr().get();

    if LAST_HWND.swap(addr, Ordering::Relaxed) != addr {
        QUEUE.push(WatcherEntry {
            window: hwnd.addr(),
            monitor: mem::transmute(hmonitor),
            is_full,
        });
    }
}

impl WatcherEntry {
    #[inline]
    pub fn monitor_handle(&self) -> NonNull<c_void> {
        unsafe { mem::transmute(self.monitor) }
    }

    #[inline]
    pub fn window_handle(&self) -> NonNull<c_void> {
        unsafe { mem::transmute(self.monitor) }
    }

    #[inline]
    pub fn is_full(&self) -> bool {
        self.is_full
    }
}
