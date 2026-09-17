use core::{
    ffi::c_void,
    ptr::{self, NonNull},
    slice,
};

#[cfg(windows)]
use std::os::windows::io::AsRawHandle;
use std::path::Path;

use smol::fs;

#[cfg(windows)]
use windows_sys::Win32::{
    Foundation::CloseHandle,
    System::Memory::{
        CreateFileMappingW, FILE_MAP_READ, MEMORY_MAPPED_VIEW_ADDRESS, MapViewOfFile,
        PAGE_READONLY, UnmapViewOfFile,
    },
};

#[cfg(windows)]
pub(crate) struct ReadMappedFile {
    file: fs::File,
    mapping: NonNull<c_void>,
    ptr: NonNull<u8>,
    len: usize,
}

#[cfg(windows)]
impl ReadMappedFile {
    #[inline]
    pub(crate) async fn open<P>(path: P) -> std::io::Result<Self>
    where
        P: AsRef<Path>,
    {
        let file = fs::File::open(path).await?;

        let len = usize::try_from(file.metadata().await?.len())
            .map_err(|_| std::io::Error::other("file is too large"))?;

        if len == 0 {
            return Err(std::io::Error::other("file is too large"));
        }

        let size = file.metadata().await?.len();

        let size_high = (size >> 32) as u32;
        let size_low = size as u32;

        let Some(mapping) = NonNull::new(unsafe {
            CreateFileMappingW(
                file.as_raw_handle() as _,
                ptr::null(),
                PAGE_READONLY,
                size_high,
                size_low,
                ptr::null(),
            )
        }) else {
            return Err(std::io::Error::last_os_error());
        };

        let Some(ptr) =
            NonNull::new(unsafe { MapViewOfFile(mapping.as_ptr(), FILE_MAP_READ, 0, 0, 0).Value })
        else {
            unsafe {
                CloseHandle(mapping.as_ptr());
            }

            return Err(std::io::Error::last_os_error());
        };

        Ok(Self {
            file,
            mapping,
            ptr: ptr.cast(),
            len,
        })
    }

    #[inline]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        if self.len == 0 {
            return &[];
        }

        // SAFETY:
        // `ptr` is a valid read-only mapping created by MapViewOfFile and
        // remains valid for the lifetime of `self`.
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
        }

        unsafe {
            CloseHandle(self.mapping.as_ptr());
        }

        let _ = &self.file;
    }
}
