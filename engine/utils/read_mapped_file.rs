#[cfg(windows)]
use core::{
    ptr::{self, NonNull},
    slice,
};

use std::path::Path;

#[cfg(windows)]
use std::os::windows::ffi::OsStrExt;

#[cfg(windows)]
use windows_sys::Win32::{
    Foundation::{CloseHandle, GENERIC_READ, HANDLE, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, GetFileSizeEx, OPEN_EXISTING,
    },
    System::Memory::{
        CreateFileMappingW, FILE_MAP_READ, MEMORY_MAPPED_VIEW_ADDRESS, MapViewOfFile,
        PAGE_READONLY, UnmapViewOfFile,
    },
};

#[cfg(windows)]
pub(crate) struct ReadMappedFile {
    file: HANDLE,
    mapping: HANDLE,
    ptr: NonNull<u8>,
    len: usize,
}

#[cfg(windows)]
impl ReadMappedFile {
    #[inline]
    pub(crate) fn open<P>(path: P) -> std::io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let wide_path = path
            .as_ref()
            .as_os_str()
            .encode_wide()
            .chain(Some(0))
            .collect::<Vec<_>>();

        let file = unsafe {
            CreateFileW(
                wide_path.as_ptr(),
                GENERIC_READ,
                FILE_SHARE_READ,
                ptr::null(),
                OPEN_EXISTING,
                FILE_ATTRIBUTE_NORMAL,
                ptr::null_mut(),
            )
        };

        if file == INVALID_HANDLE_VALUE {
            return Err(std::io::Error::last_os_error());
        }

        let mut size = 0i64;

        if unsafe { GetFileSizeEx(file, &mut size) } == 0 {
            let error = std::io::Error::last_os_error();

            unsafe {
                CloseHandle(file);
            }

            return Err(error);
        }

        if size <= 0 {
            unsafe {
                CloseHandle(file);
            }

            return Err(std::io::Error::other("file is empty"));
        }

        let len = usize::try_from(size).map_err(|_| {
            unsafe {
                CloseHandle(file);
            }

            std::io::Error::other("file is too large")
        })?;

        let mapping = unsafe {
            CreateFileMappingW(
                file,
                ptr::null(),
                PAGE_READONLY,
                (size as u64 >> 32) as u32,
                size as u32,
                ptr::null(),
            )
        };

        let Some(mapping) = NonNull::new(mapping) else {
            let error = std::io::Error::last_os_error();

            unsafe {
                CloseHandle(file);
            }

            return Err(error);
        };

        let view = unsafe { MapViewOfFile(mapping.as_ptr(), FILE_MAP_READ, 0, 0, 0) };

        let Some(ptr) = NonNull::new(view.Value.cast::<u8>()) else {
            let error = std::io::Error::last_os_error();

            unsafe {
                CloseHandle(mapping.as_ptr());
                CloseHandle(file);
            }

            return Err(error);
        };

        Ok(Self {
            file,
            mapping: mapping.as_ptr(),
            ptr,
            len,
        })
    }

    #[inline]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        // SAFETY:
        // `ptr` is the base address returned by MapViewOfFile.
        // The mapping remains alive for the lifetime of `self`.
        // `len` is the size obtained from GetFileSizeEx before mapping.
        unsafe { slice::from_raw_parts(self.ptr.as_ptr(), self.len) }
    }

    #[inline]
    pub(crate) fn is_empty(&self) -> bool {
        self.len == 0
    }
}

#[cfg(windows)]
impl Drop for ReadMappedFile {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                Value: self.ptr.as_ptr().cast(),
            });

            CloseHandle(self.mapping);
            CloseHandle(self.file);
        }
    }
}
