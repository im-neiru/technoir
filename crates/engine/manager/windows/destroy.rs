use core::mem;

use windows_sys::Win32::UI::{
    Shell::{NIM_DELETE, NOTIFYICONDATAW, Shell_NotifyIconW},
    WindowsAndMessaging::{DestroyWindow, UnregisterClassW},
};

impl Drop for super::Manager {
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
