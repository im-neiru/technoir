use core::{ffi::c_void, mem::MaybeUninit, num::NonZeroU16, ptr::NonNull};

use windows_sys::Win32::{Foundation::FALSE, UI::WindowsAndMessaging::GetClientRect};

pub(super) fn get_client_area(handle: NonNull<c_void>) -> [NonZeroU16; 2] {
    let mut rect = MaybeUninit::uninit();
    let is_err = unsafe { GetClientRect(handle.as_ptr(), rect.as_mut_ptr()) } == FALSE;

    if is_err {
        return unsafe {
            [
                NonZeroU16::new_unchecked(300),
                NonZeroU16::new_unchecked(300),
            ]
        };
    }

    let rect = unsafe { rect.assume_init() };

    let width = (rect.right - rect.left).clamp(300, 32767).cast_unsigned();
    let height = (rect.bottom - rect.top).clamp(300, 32767).cast_unsigned();

    unsafe {
        [
            NonZeroU16::new_unchecked(width as u16),
            NonZeroU16::new_unchecked(height as u16),
        ]
    }
}
