use core::{
    num::NonZeroUsize,
    ptr::{self, NonNull},
    sync::atomic::{AtomicU8, AtomicUsize, Ordering, fence},
};
use std::os::raw::c_void;

use crossbeam_queue::SegQueue;
use windows_sys::Win32::{
    Foundation::{HWND, TRUE},
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
        WindowsAndMessaging::{
            EVENT_OBJECT_LOCATIONCHANGE, EVENT_SYSTEM_FOREGROUND, IsIconic, IsWindowVisible,
            WINEVENT_OUTOFCONTEXT, WINEVENT_SKIPOWNPROCESS,
        },
    },
};

static USER_COUNT: AtomicU8 = AtomicU8::new(0);
static HOOK: AtomicUsize = AtomicUsize::new(0);
static LAST_HWND: AtomicUsize = AtomicUsize::new(0);
static QUEUE: SegQueue<NonZeroUsize> = SegQueue::new();

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
    pub fn poll(&self) -> Option<NonNull<c_void>> {
        let handle = QUEUE.pop()?;

        Some(unsafe { NonNull::new_unchecked(handle.get() as *mut c_void) })
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

    let addr = hwnd.addr().get();

    if LAST_HWND.swap(addr, Ordering::Relaxed) != addr {
        QUEUE.push(hwnd.addr());
    }
}
