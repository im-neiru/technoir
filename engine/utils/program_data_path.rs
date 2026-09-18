use core::{ptr, slice};

use std::{ffi::OsString, path::PathBuf};

#[cfg(windows)]
use std::os::windows::ffi::OsStringExt;

#[cfg(windows)]
use windows_sys::Win32::{
    System::Com::CoTaskMemFree,
    UI::Shell::{FOLDERID_ProgramData, KF_FLAG_DEFAULT, SHGetKnownFolderPath},
};

#[cfg(windows)]
#[inline]
pub(crate) fn program_data_path() -> Option<PathBuf> {
    let mut path = ptr::null_mut();

    let result = unsafe {
        SHGetKnownFolderPath(
            &FOLDERID_ProgramData,
            KF_FLAG_DEFAULT.cast_unsigned(),
            ptr::null_mut(),
            &mut path,
        )
    };

    if result != 0 || path.is_null() {
        return None;
    }

    let path_buf = unsafe {
        let mut len = 0usize;

        while *path.add(len) != 0 {
            len += 1;
        }

        let wide = slice::from_raw_parts(path, len);
        PathBuf::from(OsString::from_wide(wide))
    };

    unsafe {
        CoTaskMemFree(path.cast());
    }

    Some(path_buf)
}
