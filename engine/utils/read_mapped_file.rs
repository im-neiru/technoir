#[cfg(windows)]
use core::{
    pin::Pin,
    ptr::{self, NonNull},
    slice,
    task::{Context, Poll},
};

use std::{
    io::{self, SeekFrom},
    path::Path,
};

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

pub(crate) struct MappedFileReader<'m> {
    mapping: &'m ReadMappedFile,
    cursor: NonNull<u8>,
    end: NonNull<u8>,
}

impl<'m> smol::io::AsyncRead for MappedFileReader<'m> {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        _ctx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        let this = unsafe { self.get_unchecked_mut() };

        if buf.is_empty() {
            return Poll::Ready(Ok(0));
        }

        let remaining = unsafe { this.end.byte_offset_from_unsigned(this.cursor) };

        if remaining == 0 {
            return Poll::Ready(Ok(0));
        }

        let len = remaining.min(buf.len());

        unsafe {
            let dest = NonNull::new_unchecked(buf.as_mut_ptr());

            this.cursor.copy_to_nonoverlapping(dest, len);

            this.cursor = this.cursor.byte_add(len);
        }

        Poll::Ready(Ok(len))
    }
}

const MAX_OFFSET64: u64 = isize::MAX as u64;
const SEEK_OUT_OF_BOUNDS: &str = "seek position out of bounds";

impl<'m> smol::io::AsyncSeek for MappedFileReader<'m> {
    #[inline]
    fn poll_seek(
        self: Pin<&mut Self>,
        _ctx: &mut Context<'_>,
        pos: SeekFrom,
    ) -> Poll<io::Result<u64>> {
        let this = unsafe { self.get_unchecked_mut() };

        let len = unsafe { this.end.byte_offset_from_unsigned(this.mapping.ptr) };

        match pos {
            SeekFrom::Start(offset) => {
                if offset > MAX_OFFSET64 || offset > len as u64 {
                    return Poll::Ready(Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        SEEK_OUT_OF_BOUNDS,
                    )));
                }

                this.cursor = unsafe { this.mapping.ptr.byte_add(offset as usize) };

                Poll::Ready(Ok(offset))
            }

            SeekFrom::End(offset) => {
                if offset >= 0 {
                    if offset as u64 > MAX_OFFSET64 - len as u64 {
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            SEEK_OUT_OF_BOUNDS,
                        )));
                    }

                    let offset = offset as usize;

                    this.cursor = unsafe { this.end.byte_add(offset) };

                    Poll::Ready(Ok(len as u64 + offset as u64))
                } else {
                    let offset = offset.unsigned_abs();

                    if offset > len as u64 {
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            SEEK_OUT_OF_BOUNDS,
                        )));
                    }

                    this.cursor = unsafe { this.end.byte_sub(offset as usize) };

                    Poll::Ready(Ok(len as u64 - offset))
                }
            }

            SeekFrom::Current(offset) => {
                let current = unsafe { this.cursor.byte_offset_from_unsigned(this.mapping.ptr) };

                if offset >= 0 {
                    let offset = offset as u64;
                    let remaining = len - current;

                    if offset > MAX_OFFSET64 - current as u64 || offset > remaining as u64 {
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            SEEK_OUT_OF_BOUNDS,
                        )));
                    }

                    this.cursor = unsafe { this.cursor.byte_add(offset as usize) };

                    Poll::Ready(Ok(current as u64 + offset))
                } else {
                    let offset = offset.unsigned_abs();

                    if offset > current as u64 {
                        return Poll::Ready(Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            SEEK_OUT_OF_BOUNDS,
                        )));
                    }

                    this.cursor = unsafe { this.cursor.byte_sub(offset as usize) };

                    Poll::Ready(Ok(current as u64 - offset))
                }
            }
        }
    }
}
