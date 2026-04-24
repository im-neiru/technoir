use core::{
    mem,
    ptr::{self, NonNull},
};

use windows_sys::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

use super::state::State;

pub(super) fn enter_loop(mut state: State) {
    unsafe {
        let state_ptr = NonNull::new_unchecked((&mut state) as *mut State);
        state_ptr.as_ref().manager.store_state(state_ptr);
    };

    unsafe {
        let mut msg = mem::zeroed::<MSG>();

        while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
            TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

pub(super) const WM_APP_TERMINATE: u32 = WM_APP + 1;
pub(super) const WM_USER_TRAY: u32 = WM_USER + 1;

#[allow(unsafe_op_in_unsafe_fn)]
pub(super) unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let state = NonNull::<State>::new(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as _);

    match msg {
        WM_USER_TRAY => {
            if let Some(mut state_ptr) = state {
                let state = state_ptr.as_mut();

                match lparam as u32 {
                    WM_LBUTTONUP => state.manager.show(),
                    WM_RBUTTONUP => state.manager.show_tray_menu(),
                    _ => {}
                }
            }

            0
        }
        WM_APP_TERMINATE => {
            PostQuitMessage(0);
            0
        }
        WM_CLOSE => {
            if let Some(mut state) = state {
                unsafe {
                    state.as_mut().manager.hide();
                }

                0
            } else {
                DefWindowProcW(hwnd, msg, wparam, lparam)
            }
        }
        WM_DESTROY => {
            PostQuitMessage(0);

            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

#[derive(Debug)]
pub(super) struct WindowsDisplayHandle;

impl raw_window_handle::HasDisplayHandle for WindowsDisplayHandle {
    #[inline]
    fn display_handle(
        &self,
    ) -> Result<raw_window_handle::DisplayHandle<'_>, raw_window_handle::HandleError> {
        Ok(raw_window_handle::DisplayHandle::windows())
    }
}
