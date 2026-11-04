use core::{
    ffi::c_void,
    ptr::{self, NonNull, null_mut},
};

use windows_sys::{
    Win32::{
        Foundation::HWND, Graphics::Gdi::*, System::Threading::Sleep, UI::WindowsAndMessaging::*,
    },
    w,
};

#[allow(unused)]
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

impl DesktopHandles {
    pub(super) fn find() -> Option<Self> {
        unsafe {
            let progman = NonNull::new(FindWindowW(CLASS_PROGMAN, null_mut()))?;

            SendMessageTimeoutW(
                progman.as_ptr(),
                SPAWN_WORKER_W_MESSAGE,
                0,
                0,
                SMTO_NORMAL,
                1000,
                null_mut(),
            );

            let shelldll_defview =
                find_last_child_with_class(progman.as_ptr(), CLASS_SHELL_DEFVIEW);
            let worker_w = find_last_child_with_class(progman.as_ptr(), CLASS_WORKER_W);

            let icons_hwnd = match shelldll_defview {
                Some(defview) => find_last_child_with_class(defview.as_ptr(), CLASS_SYS_LIST_VIEW),
                None => None,
            };

            Some(Self {
                progman,
                shelldll_defview,
                icons_hwnd,
                worker_w,
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
