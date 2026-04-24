use core::{
    ffi::c_void,
    mem,
    num::NonZero,
    ptr::{self, NonNull},
};

use windows_sys::{
    Win32::UI::{
        Shell::{NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NOTIFYICONDATAW, Shell_NotifyIconW},
        WindowsAndMessaging::*,
    },
    core::PCWSTR,
    w,
};

use raw_window_handle::{
    RawDisplayHandle, RawWindowHandle, Win32WindowHandle, WindowsDisplayHandle,
};

use super::event_loop::{WM_APP_TERMINATE, WM_USER_TRAY};

pub struct Manager {
    hwnd: NonNull<c_void>,
    is_open: bool,
    wgpu_surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
}

impl Manager {
    pub(super) async fn new(
        hinstance: NonNull<c_void>,
        visible: bool,
        wgpu_instance: &wgpu::Instance,
    ) -> Self {
        let win = unsafe { Self::new_manager_win(hinstance, visible) };

        let wgpu_surface = unsafe {
            wgpu_instance
                .create_surface_unsafe(wgpu::SurfaceTargetUnsafe::RawHandle {
                    raw_display_handle: Some(
                        RawDisplayHandle::Windows(WindowsDisplayHandle::new()),
                    ),
                    raw_window_handle: RawWindowHandle::Win32(Win32WindowHandle::new(
                        win.addr().cast_signed(),
                    )),
                })
                .expect("Failed to create wgpu::Surface")
        };

        let adapter = wgpu_instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&wgpu_surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("No compatible adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to request device");

        Self {
            hwnd: win,
            is_open: visible,
            wgpu_surface,
            device,
            queue,
        }
    }
}

impl Manager {
    const SANDBOX_WIN_NAME: PCWSTR = w!("TechNoir");

    unsafe fn new_manager_win(hinstance: NonNull<c_void>, visible: bool) -> NonNull<c_void> {
        let wc = unsafe {
            let cursor = LoadCursorW(ptr::null_mut(), IDC_ARROW);

            WNDCLASSW {
                style: CS_HREDRAW | CS_VREDRAW,
                lpfnWndProc: Some(super::event_loop::window_proc),
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

    pub(super) fn store_state(&self, state_ptr: NonNull<super::state::State>) {
        unsafe {
            SetWindowLongPtrW(
                self.hwnd.as_ptr(),
                GWLP_USERDATA,
                state_ptr.addr().cast_signed().get(),
            );

            self.init_tray();
        };
    }

    #[allow(unsafe_op_in_unsafe_fn)]
    unsafe fn init_tray(&self) {
        let mut nid: NOTIFYICONDATAW = mem::zeroed();

        nid.cbSize = mem::size_of::<NOTIFYICONDATAW>() as u32;
        nid.hWnd = self.hwnd.as_ptr();
        nid.uID = 1;

        nid.uFlags = NIF_MESSAGE | NIF_TIP | NIF_ICON;
        nid.uCallbackMessage = WM_USER_TRAY;

        nid.hIcon = LoadIconW(ptr::null_mut(), IDI_APPLICATION);

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
            DestroyWindow(self.hwnd.as_ptr());
        }
    }
}
