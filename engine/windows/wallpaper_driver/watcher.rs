use core::{
    ptr,
    sync::atomic::{AtomicU8, AtomicUsize, Ordering, fence},
};

use windows_sys::Win32::{
    Foundation::HWND,
    UI::{
        Accessibility::{HWINEVENTHOOK, SetWinEventHook, UnhookWinEvent},
        WindowsAndMessaging::{
            EVENT_OBJECT_LOCATIONCHANGE, EVENT_SYSTEM_FOREGROUND, WINEVENT_OUTOFCONTEXT,
            WINEVENT_SKIPOWNPROCESS,
        },
    },
};

static USER_COUNT: AtomicU8 = AtomicU8::new(0);
static HOOK: AtomicUsize = AtomicUsize::new(0);

pub struct WatcherGuard;

pub fn start_watching() -> Option<WatcherGuard> {
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
        .compare_exchange(0, hook.addr(), Ordering::Release, Ordering::Relaxed)
        .is_err()
    {
        unsafe { UnhookWinEvent(hook) };
    }

    if USER_COUNT.fetch_add(1, Ordering::Relaxed) == u8::MAX {
        std::process::abort();
    }

    Some(WatcherGuard)
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
    if hwnd.is_null() {
        return;
    }

    // todo send
}
