use core::{
    ffi::c_void,
    mem,
    ptr::{self, NonNull},
};
use std::num::NonZeroU16;

use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
};
use windows_sys::{
    Win32::{
        Foundation::{ERROR_CLASS_ALREADY_EXISTS, GetLastError},
        UI::{
            Shell::{
                NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NOTIFYICONDATAW,
                Shell_NotifyIconW,
            },
            WindowsAndMessaging::*,
        },
    },
    core::PCWSTR,
    w,
};

use crate::graphics::Context;

impl super::Manager {
    pub(crate) fn new(hinstance: NonNull<c_void>, visible: bool) -> Self {
        let hwnd = unsafe { Self::new_manager_win(hinstance, visible) };

        let primary_target = wgpu::SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: Some(RawDisplayHandle::Windows(WindowsDisplayHandle::new())),
            raw_window_handle: RawWindowHandle::Win32({
                let mut h = Win32WindowHandle::new(hwnd.addr().cast_signed());

                h.hinstance = Some(hinstance.addr().cast_signed());

                h
            }),
        };

        let (width, height) = unsafe {
            let mut rect = mem::zeroed();

            GetClientRect(hwnd.as_ptr() as _, &mut rect);

            (
                NonZeroU16::new_unchecked((rect.right - rect.left).max(1) as u16),
                NonZeroU16::new_unchecked((rect.bottom - rect.top).max(1) as u16),
            )
        };

        let (ctx, surface) =
            smol::block_on(Context::new_with_primary(primary_target, width, height));

        Self {
            hwnd,
            hinstance,
            is_open: visible,
            ui: ui::ManagerUi::new(),
            ctx,
            surface,
        }
    }

    pub(super) fn resize(&mut self) {
        let (width, height) = unsafe {
            let mut rect = mem::zeroed();

            GetClientRect(self.hwnd.as_ptr() as _, &mut rect);

            (
                NonZeroU16::new_unchecked((rect.right - rect.left).max(1) as u16),
                NonZeroU16::new_unchecked((rect.bottom - rect.top).max(1) as u16),
            )
        };

        self.surface.resize(&self.ctx, width, height);
    }

    pub(super) const SANDBOX_WIN_NAME: PCWSTR = w!("TechNoir");

    unsafe fn new_manager_win(hinstance: NonNull<c_void>, visible: bool) -> NonNull<c_void> {
        let wc = unsafe {
            let cursor = LoadCursorW(ptr::null_mut(), IDC_ARROW);

            let app_icon = LoadImageW(
                hinstance.as_ptr(),
                1 as PCWSTR,
                IMAGE_ICON,
                0,
                0,
                LR_DEFAULTSIZE | LR_SHARED,
            ) as HICON;

            WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(super::event_loop::window_proc),
                hInstance: hinstance.as_ptr(),
                lpszClassName: Self::SANDBOX_WIN_NAME,
                hCursor: cursor,
                hIcon: app_icon,
                ..mem::zeroed()
            }
        };

        if unsafe { RegisterClassW(&wc) } == 0 {
            let err = unsafe { GetLastError() };
            if err != ERROR_CLASS_ALREADY_EXISTS {
                panic!("Failed to register window class: {}", err);
            }
        }

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

    pub(super) fn store_state(&self, state_ptr: NonNull<crate::Entry>) {
        unsafe {
            SetWindowLongPtrW(
                self.hwnd.as_ptr(),
                GWLP_USERDATA,
                state_ptr.addr().cast_signed().get(),
            );

            self.init_tray(state_ptr.as_ref().hinstance);
        };
    }

    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn init_tray(&self, hinstance: NonNull<c_void>) {
        let mut nid: NOTIFYICONDATAW = mem::zeroed();

        nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = self.hwnd.as_ptr();
        nid.uID = 1;

        nid.uFlags = NIF_MESSAGE | NIF_TIP | NIF_ICON;
        nid.uCallbackMessage = super::WM_USER_TRAY;

        let h_icon = LoadImageW(
            hinstance.as_ptr(),
            1 as PCWSTR,
            IMAGE_ICON,
            0,
            0,
            LR_DEFAULTSIZE | LR_SHARED,
        ) as HICON;

        nid.hIcon = h_icon;

        let tip = "Open Manager\0".encode_utf16().collect::<Vec<u16>>();
        nid.szTip[..tip.len()].copy_from_slice(&tip);

        unsafe { Shell_NotifyIconW(NIM_ADD, &nid) };
    }

    pub(super) fn show_tray_menu(&mut self) {
        unsafe {
            let menu = CreatePopupMenu();
            AppendMenuW(menu, MF_STRING, 1, w!("Open Manager"));
            AppendMenuW(menu, MF_STRING, 2, w!("Terminate"));
            let mut pt = std::mem::zeroed();
            GetCursorPos(&mut pt);
            SetForegroundWindow(self.hwnd.as_ptr());
            let cmd = TrackPopupMenu(
                menu,
                TPM_RETURNCMD,
                pt.x,
                pt.y,
                0,
                self.hwnd.as_ptr(),
                std::ptr::null(),
            );
            match cmd {
                1 => self.show(),
                2 => self.terminate_program(),
                _ => {}
            }
            DestroyMenu(menu);
        }
    }
}
