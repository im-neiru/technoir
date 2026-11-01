use core::{
    ffi::c_void,
    ptr::{self, NonNull, null_mut},
};
use std::sync::OnceLock;

use windows_sys::{
    Win32::{Graphics::Gdi::*, System::Threading::Sleep, UI::WindowsAndMessaging::*},
    w,
};

static HANDLES: OnceLock<DesktopHandles> = OnceLock::new();

type Handle = NonNull<c_void>;

#[allow(unused)]
pub(crate) struct DesktopHandles {
    pub(super) progman: Handle,
    pub(super) shelldll_defview: Option<Handle>,
    pub(super) icons_hwnd: Option<Handle>,
    pub(super) worker_w: Option<Handle>,
}

pub(crate) fn get_desktop_handles() -> &'static DesktopHandles {
    HANDLES.get_or_init(DesktopHandles::find)
}

impl DesktopHandles {
    fn find() -> Self {
        unsafe {
            let progman =
                NonNull::new(FindWindowW(w!("Progman"), null_mut())).expect("Progman not found");

            let mut shelldll_defview = None;
            let mut icons_hwnd = None;
            let mut worker_w = None;

            SendMessageTimeoutW(
                progman.as_ptr(),
                0x052C,
                0,
                0,
                SMTO_NORMAL,
                1000,
                null_mut(),
            );

            Sleep(500);

            let mut child = GetWindow(progman.as_ptr(), GW_CHILD);

            while !child.is_null() {
                let mut class_name = [0u16; 256];
                let len = GetClassNameW(child, class_name.as_mut_ptr(), 256);
                let name = String::from_utf16_lossy(&class_name[..len as usize]);

                if name == "SHELLDLL_DefView" {
                    shelldll_defview = NonNull::new(child);

                    icons_hwnd = NonNull::new(FindWindowExW(
                        child,
                        null_mut(),
                        w!("SysListView32"),
                        null_mut(),
                    ));
                } else if name == "WorkerW" {
                    worker_w = NonNull::new(child);
                }

                child = GetWindow(child, GW_HWNDNEXT);
            }

            Self {
                progman,
                shelldll_defview,
                icons_hwnd,
                worker_w,
            }
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
    pub(crate) fn get_target_parent(&self) -> NonNull<c_void> {
        self.worker_w.unwrap_or(self.progman)
    }

    #[inline]
    pub(super) fn is_desktop_handle(&self, hwnd: NonNull<c_void>) -> bool {
        hwnd == self.progman || Some(hwnd) == self.shelldll_defview
    }
}

unsafe impl Send for DesktopHandles {}
unsafe impl Sync for DesktopHandles {}
