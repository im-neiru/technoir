use core::{
    ffi::c_void,
    future::{Future, poll_fn},
    hint::spin_loop,
    pin::pin,
    ptr::{self, NonNull, null_mut},
    sync::atomic::{AtomicBool, AtomicPtr, Ordering},
    task::{Context, Poll, Waker},
};

use windows_sys::{
    Win32::{
        Foundation::{CloseHandle, HWND, WAIT_OBJECT_0},
        Graphics::Gdi::*,
        System::{
            SystemInformation::GetTickCount64,
            Threading::{CreateThread, INFINITE, WaitForSingleObject},
        },
        UI::WindowsAndMessaging::*,
    },
    w,
};

#[allow(unused)]
#[derive(Debug)]
pub(crate) struct DesktopHandles {
    pub(super) progman: NonNull<c_void>,
    pub(super) shelldll_defview: Option<NonNull<c_void>>,
    pub(super) icons_hwnd: Option<NonNull<c_void>>,
    pub(super) worker_w: Option<NonNull<c_void>>,
}

const CLASS_PROGMAN: *const u16 = w!("Progman");
const CLASS_SHELL_DEFVIEW: *const u16 = w!("SHELLDLL_DefView");
const CLASS_SYS_LIST_VIEW: *const u16 = w!("SysListView32");
const CLASS_WORKER_W: *const u16 = w!("WorkerW");

const SPAWN_WORKER_W_MESSAGE: u32 = 0x052C;
const DESKTOP_POLL_MS: u64 = 1000;

impl DesktopHandles {
    pub(super) fn find() -> Option<Self> {
        unsafe {
            let progman = NonNull::new(FindWindowW(CLASS_PROGMAN, null_mut()))?;

            let mut handles = DesktopPollResult {
                shelldll_defview: find_last_child_with_class(progman.as_ptr(), CLASS_SHELL_DEFVIEW),
                worker_w: find_last_child_with_class(progman.as_ptr(), CLASS_WORKER_W),
                ..Default::default()
            };

            {
                if let Some(defview) = handles.shelldll_defview {
                    handles.icons_hwnd =
                        find_last_child_with_class(defview.as_ptr(), CLASS_SYS_LIST_VIEW);
                }

                if handles.shelldll_defview.is_some()
                    && handles.icons_hwnd.is_some()
                    && handles.worker_w.is_some()
                {
                    return Some(Self {
                        progman,
                        shelldll_defview: handles.shelldll_defview,
                        icons_hwnd: handles.icons_hwnd,
                        worker_w: handles.worker_w,
                    });
                }
            }

            SendMessageTimeoutW(
                progman.as_ptr(),
                SPAWN_WORKER_W_MESSAGE,
                0,
                0,
                SMTO_NORMAL,
                1000,
                null_mut(),
            );

            let state = DesktopPollState::default();

            state.shelldll_defview.store(
                handles.shelldll_defview.map_or(null_mut(), NonNull::as_ptr),
                Ordering::Relaxed,
            );
            state.icons_hwnd.store(
                handles.icons_hwnd.map_or(null_mut(), NonNull::as_ptr),
                Ordering::Relaxed,
            );
            state.worker_w.store(
                handles.worker_w.map_or(null_mut(), NonNull::as_ptr),
                Ordering::Relaxed,
            );

            let deadline = GetTickCount64() + DESKTOP_POLL_MS;

            let context = DesktopPollContext {
                progman: progman.as_ptr(),
                deadline,
                state: &state,
            };

            let worker = CreateThread(
                ptr::null(),
                0,
                Some(desktop_poll_thread),
                (&context as *const DesktopPollContext).cast(),
                0,
                null_mut(),
            );

            if !worker.is_null() {
                let remaining = deadline
                    .saturating_sub(GetTickCount64())
                    .min(u32::MAX as u64) as u32;

                if WaitForSingleObject(worker, remaining) != WAIT_OBJECT_0 {
                    state.stop.store(true, Ordering::Release);
                    WaitForSingleObject(worker, INFINITE);
                }

                CloseHandle(worker);
            }

            handles = state.collect();

            Some(Self {
                progman,
                shelldll_defview: handles.shelldll_defview,
                icons_hwnd: handles.icons_hwnd,
                worker_w: handles.worker_w,
            })
        }
    }

    pub(crate) fn redraw(&self) {
        unsafe {
            InvalidateRect(self.progman.as_ptr(), ptr::null_mut(), 1);

            if let Some(icons_hwnd) = self.icons_hwnd {
                InvalidateRect(icons_hwnd.as_ptr(), ptr::null_mut(), 1);
            }

            if let Some(shelldll_defview) = self.shelldll_defview {
                InvalidateRect(shelldll_defview.as_ptr(), ptr::null_mut(), 1);
            }

            if let Some(worker_w) = self.worker_w {
                InvalidateRect(worker_w.as_ptr(), ptr::null_mut(), 1);
            }
        }
    }

    #[inline]
    pub(super) fn get_target_parent(&self) -> NonNull<c_void> {
        self.worker_w.unwrap_or(self.progman)
    }

    #[inline]
    pub(super) fn is_desktop_handle(&self, hwnd: NonNull<c_void>) -> bool {
        hwnd == self.progman || Some(hwnd) == self.shelldll_defview
    }
}

struct DesktopPollContext {
    progman: HWND,
    deadline: u64,
    state: *const DesktopPollState,
}

unsafe extern "system" fn desktop_poll_thread(parameter: *mut c_void) -> u32 {
    let context = unsafe { &*parameter.cast::<DesktopPollContext>() };
    let state = unsafe { &*context.state };

    poll_desktop_children(context.progman, context.deadline, state);

    0
}

struct DesktopPollState {
    shelldll_defview: AtomicPtr<c_void>,
    icons_hwnd: AtomicPtr<c_void>,
    worker_w: AtomicPtr<c_void>,
    stop: AtomicBool,
}

impl Default for DesktopPollState {
    #[inline(always)]
    fn default() -> Self {
        Self {
            shelldll_defview: AtomicPtr::new(ptr::null_mut()),
            icons_hwnd: AtomicPtr::new(ptr::null_mut()),
            worker_w: AtomicPtr::new(ptr::null_mut()),
            stop: AtomicBool::new(false),
        }
    }
}

impl DesktopPollState {
    #[inline]
    fn is_complete(&self) -> bool {
        !self.shelldll_defview.load(Ordering::Acquire).is_null()
            && !self.icons_hwnd.load(Ordering::Acquire).is_null()
            && !self.worker_w.load(Ordering::Acquire).is_null()
    }

    #[inline]
    fn collect(&self) -> DesktopPollResult {
        DesktopPollResult {
            shelldll_defview: NonNull::new(self.shelldll_defview.load(Ordering::Acquire)),
            icons_hwnd: NonNull::new(self.icons_hwnd.load(Ordering::Acquire)),
            worker_w: NonNull::new(self.worker_w.load(Ordering::Acquire)),
        }
    }
}

#[derive(Default)]
struct DesktopPollResult {
    shelldll_defview: Option<NonNull<c_void>>,
    icons_hwnd: Option<NonNull<c_void>>,
    worker_w: Option<NonNull<c_void>>,
}

fn poll_desktop_children(progman: HWND, deadline: u64, state: &DesktopPollState) {
    let mut future = pin!(poll_fn(|_| {
        if state.stop.load(Ordering::Acquire) || unsafe { GetTickCount64() } >= deadline {
            return Poll::Ready(());
        }

        unsafe {
            if state.shelldll_defview.load(Ordering::Relaxed).is_null()
                && let Some(hwnd) = find_last_child_with_class(progman, CLASS_SHELL_DEFVIEW)
            {
                state
                    .shelldll_defview
                    .store(hwnd.as_ptr(), Ordering::Release);
            }

            if state.worker_w.load(Ordering::Relaxed).is_null()
                && let Some(hwnd) = find_last_child_with_class(progman, CLASS_WORKER_W)
            {
                state.worker_w.store(hwnd.as_ptr(), Ordering::Release);
            }

            let defview = state.shelldll_defview.load(Ordering::Acquire);

            if !defview.is_null()
                && state.icons_hwnd.load(Ordering::Relaxed).is_null()
                && let Some(hwnd) = find_last_child_with_class(defview, CLASS_SYS_LIST_VIEW)
            {
                state.icons_hwnd.store(hwnd.as_ptr(), Ordering::Release);
            }
        }

        if state.is_complete()
            || state.stop.load(Ordering::Acquire)
            || unsafe { GetTickCount64() } >= deadline
        {
            Poll::Ready(())
        } else {
            Poll::Pending
        }
    }));

    let mut cx = Context::from_waker(Waker::noop());

    loop {
        match future.as_mut().poll(&mut cx) {
            Poll::Ready(()) => break,
            Poll::Pending => spin_loop(),
        }
    }
}

unsafe fn find_last_child_with_class(
    parent: HWND,
    class_name: *const u16,
) -> Option<NonNull<c_void>> {
    let mut last_match = None;
    let mut previous_child = null_mut();

    loop {
        let next_child = unsafe { FindWindowExW(parent, previous_child, class_name, null_mut()) };

        let Some(next_match) = NonNull::new(next_child) else {
            return last_match;
        };

        last_match = Some(next_match);
        previous_child = next_child;
    }
}
