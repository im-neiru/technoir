mod event_loop;

use core::{
    ffi::c_void,
    mem,
    ptr::{self, NonNull},
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

use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
};

use super::messages::{WM_APP_TERMINATE, WM_USER_TRAY};

use vello::wgpu;

pub(crate) use event_loop::enter_loop;

pub struct Manager {
    hwnd: NonNull<c_void>,
    hinstance: NonNull<c_void>,
    is_open: bool,
    graphics: engine::Renderer,
    ui: ui::ManagerUi,
}

impl Manager {
    pub(super) async fn new(
        hinstance: NonNull<c_void>,
        visible: bool,
        wgpu_instance: &wgpu::Instance,
    ) -> Self {
        let hwnd = unsafe { Self::new_manager_win(hinstance, visible) };
        let wgpu_surface = unsafe {
            wgpu_instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: RawDisplayHandle::Windows(WindowsDisplayHandle::new()),

                    raw_window_handle: RawWindowHandle::Win32(Win32WindowHandle::new(
                        hwnd.addr().cast_signed(),
                    )),
                })
                .expect("Failed to create wgpu::Surface")
        };

        let (width, height) = {
            let mut rect = unsafe { mem::zeroed() };
            unsafe { GetClientRect(hwnd.as_ptr() as _, &mut rect) };
            (
                (rect.right - rect.left).max(1) as u32,
                (rect.bottom - rect.top).max(1) as u32,
            )
        };

        let graphics = engine::Renderer::new(wgpu_instance, wgpu_surface, width, height).await;

        Self {
            hwnd,
            hinstance,
            is_open: visible,
            graphics,
            ui: ui::ManagerUi::new(),
        }
    }

    pub(super) fn render(&mut self) {
        let (scene, base_color) = self.ui.render();
        self.graphics.render_scene(scene, base_color);
    }

    pub(super) fn resize(&mut self, width: u32, height: u32) {
        self.graphics.resize(width, height);
    }
}

impl Manager {
    const SANDBOX_WIN_NAME: PCWSTR = w!("TechNoir");

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
                lpfnWndProc: Some(event_loop::window_proc),
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

    pub(super) fn store_state(&self, state_ptr: NonNull<super::state::State>) {
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
        nid.uCallbackMessage = WM_USER_TRAY;

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

    pub(super) fn show(&mut self) {
        if !self.is_open {
            unsafe { ShowWindow(self.hwnd.as_ptr(), SW_SHOW) };
            self.is_open = true;
        }
    }

    pub(super) fn hide(&mut self) {
        if self.is_open {
            unsafe { ShowWindow(self.hwnd.as_ptr(), SW_HIDE) };
            self.is_open = false;
        }
    }

    pub(super) fn terminate_program(&mut self) {
        unsafe { PostMessageW(self.hwnd.as_ptr(), WM_APP_TERMINATE, 0, 0) };
    }
}

impl Drop for Manager {
    fn drop(&mut self) {
        unsafe {
            let mut nid: NOTIFYICONDATAW = mem::zeroed();
            nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
            nid.hWnd = self.hwnd.as_ptr();
            nid.uID = 1;
            Shell_NotifyIconW(NIM_DELETE, &nid);

            DestroyWindow(self.hwnd.as_ptr());
            UnregisterClassW(Self::SANDBOX_WIN_NAME, self.hinstance.as_ptr());
        }
    }
}
