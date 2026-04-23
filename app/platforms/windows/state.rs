use core::{
    ffi::c_void,
    mem,
    num::NonZero,
    ptr::{self, NonNull},
};

use windows_sys::{
    Win32::{
        Foundation::*,
        System::LibraryLoader::GetModuleHandleW,
        UI::{
            Shell::{NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NOTIFYICONDATAW, Shell_NotifyIconW},
            WindowsAndMessaging::*,
        },
    },
    core::PCWSTR,
    w,
};

pub struct State {
    manager_win: NonNull<c_void>,
    is_manager_open: bool,
}

impl State {
    pub(super) fn new(config: &crate::config::Config) -> Self {
        let hinstance = unsafe { NonNull::new(GetModuleHandleW(ptr::null_mut())) }
            .expect("Failed to retrieve module handle");

        let manager_win = unsafe { Self::new_manager_win(hinstance, config.open_manager) };

        Self {
            manager_win,
            is_manager_open: config.open_manager,
        }
    }

    pub(super) fn enter_loop(mut self) {
        unsafe {
            let state_ptr = (&mut self) as *mut Self;

            SetWindowLongPtrW(
                self.manager_win.as_ptr(),
                GWLP_USERDATA,
                state_ptr.addr().cast_signed(),
            );

            self.init_tray();
        };

        unsafe {
            let mut msg = mem::zeroed::<MSG>();

            while GetMessageW(&mut msg, ptr::null_mut(), 0, 0) > 0 {
                TranslateMessage(&msg);
                DispatchMessageW(&msg);
            }
        }
    }
}

impl Drop for State {
    fn drop(&mut self) {
        unsafe {
            DestroyWindow(self.manager_win.as_ptr());
        }
    }
}

#[allow(unsafe_op_in_unsafe_fn)]
unsafe extern "system" fn window_proc(
    hwnd: HWND,
    msg: u32,
    wparam: WPARAM,
    lparam: LPARAM,
) -> LRESULT {
    let state = NonNull::<State>::new(GetWindowLongPtrW(hwnd, GWLP_USERDATA) as _);

    match msg {
        val if val == WM_USER + 1 => {
            if let Some(mut state_ptr) = state {
                let state = state_ptr.as_mut();

                match lparam as u32 {
                    WM_LBUTTONUP => state.open_manager(),
                    WM_RBUTTONUP => state.show_tray_menu(),
                    _ => {}
                }
            }

            0
        }

        WM_DESTROY => {
            PostQuitMessage(0);

            0
        }

        _ => DefWindowProcW(hwnd, msg, wparam, lparam),
    }
}

impl State {
    const SANDBOX_WIN_NAME: PCWSTR = w!("TechNoir");

    unsafe fn new_manager_win(hinstance: NonNull<c_void>, visible: bool) -> NonNull<c_void> {
        let wc = unsafe {
            let cursor = LoadCursorW(ptr::null_mut(), IDC_ARROW);

            WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(window_proc),
                hInstance: hinstance.as_ptr(),
                lpszClassName: Self::SANDBOX_WIN_NAME,
                hCursor: cursor,

                ..mem::zeroed()
            }
        };

        unsafe { NonZero::new(RegisterClassW(&wc)) }.expect("Failed to register window class");

        NonNull::new(unsafe {
            let mut style = WS_OVERLAPPEDWINDOW;

            if visible {
                style |= WS_VISIBLE;
            }

            CreateWindowExW(
                0,
                Self::SANDBOX_WIN_NAME,
                Self::SANDBOX_WIN_NAME,
                style,
                CW_USEDEFAULT,
                CW_USEDEFAULT,
                800,
                600,
                ptr::null_mut(),
                ptr::null_mut(),
                hinstance.as_ptr(),
                ptr::null_mut(),
            )
        })
        .expect("Failed to create window")
    }

    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn init_tray(&self) {
        let mut nid: NOTIFYICONDATAW = mem::zeroed();

        nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = self.manager_win.as_ptr();
        nid.uID = 1;

        nid.uFlags = NIF_MESSAGE | NIF_TIP | NIF_ICON;
        nid.uCallbackMessage = WM_USER + 1;

        nid.hIcon = LoadIconW(ptr::null_mut(), IDI_APPLICATION);

        let tip = "Open Manager\0".encode_utf16().collect::<Vec<u16>>();
        nid.szTip[..tip.len()].copy_from_slice(&tip);

        unsafe { Shell_NotifyIconW(NIM_ADD, &nid) };
    }

    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn show_tray_menu(&mut self) {
        let menu = CreatePopupMenu();
        AppendMenuW(menu, MF_STRING, 1, w!("Open Manager"));
        AppendMenuW(menu, MF_STRING, 2, w!("Terminate"));
        let mut pt = std::mem::zeroed();
        GetCursorPos(&mut pt);
        SetForegroundWindow(self.manager_win.as_ptr());
        let cmd = TrackPopupMenu(
            menu,
            TPM_RETURNCMD,
            pt.x,
            pt.y,
            0,
            self.manager_win.as_ptr(),
            std::ptr::null(),
        );
        match cmd {
            1 => self.open_manager(),
            2 => self.terminate_program(),
            _ => {}
        }
        DestroyMenu(menu);
    }

    fn open_manager(&mut self) {
        if self.is_manager_open {
            return; // TODO
        }
    }

    fn terminate_program(&mut self) {
        // TODO
    }
}
