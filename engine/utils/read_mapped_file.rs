#[cfg(windows)]
use core::{
    ffi::c_void,
    pin::Pin,
    ptr::{self, NonNull},
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
    Foundation::{CloseHandle, GENERIC_READ, INVALID_HANDLE_VALUE},
    Storage::FileSystem::{
        CreateFileW, FILE_ATTRIBUTE_NORMAL, FILE_SHARE_READ, GetFileSizeEx, OPEN_EXISTING,
    },
    System::Memory::{
        CreateFileMappingW, FILE_MAP_READ, MEMORY_MAPPED_VIEW_ADDRESS, MapViewOfFile,
        PAGE_READONLY, UnmapViewOfFile,
    },
};

type HandleNonNull = NonNull<c_void>;

#[cfg(windows)]
pub(crate) struct ReadMappedFile {
    slice: NonNull<[u8]>,
    file: HandleNonNull,
    mapping: HandleNonNull,
}

#[cfg(windows)]
impl ReadMappedFile {
    #[inline]
    pub(crate) fn open<P>(path: P) -> io::Result<Self>
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
            return Err(io::Error::last_os_error());
        }

        let Some(file) = NonNull::new(file) else {
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "CreateFileW returned a null handle",
            ));
        };

        let mut size = 0i64;

        if unsafe { GetFileSizeEx(file.as_ptr(), &mut size) } == 0 {
            let error = io::Error::last_os_error();

            unsafe {
                CloseHandle(file.as_ptr());
            }

            return Err(error);
        }

        if size <= 0 {
            unsafe {
                CloseHandle(file.as_ptr());
            }

            return Err(std::io::Error::other("file is empty"));
        }

        let len = usize::try_from(size).map_err(|_| {
            unsafe {
                CloseHandle(file.as_ptr());
            }

            std::io::Error::other("file is too large")
        })?;

        let mapping = unsafe {
            CreateFileMappingW(
                file.as_ptr(),
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
                CloseHandle(file.as_ptr());
            }

            return Err(error);
        };

        let view = unsafe { MapViewOfFile(mapping.as_ptr(), FILE_MAP_READ, 0, 0, 0) };

        let Some(ptr) = NonNull::new(view.Value.cast::<u8>()) else {
            let error = std::io::Error::last_os_error();

            unsafe {
                CloseHandle(mapping.as_ptr());
                CloseHandle(file.as_ptr());
            }

            return Err(error);
        };

        let slice = NonNull::slice_from_raw_parts(ptr, len);

        debug_assert_eq!(
            slice.cast::<u8>().addr(),
            ptr.addr(),
            "Reconstructed file mapping slice address ({:?}) drifted from original pointer ({:?})",
            slice.cast::<u8>(),
            ptr
        );

        Ok(Self {
            file,
            mapping,
            slice,
        })
    }

    #[inline]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        // SAFETY:
        // `ptr` is the base address returned by MapViewOfFile.
        // The mapping remains alive for the lifetime of `self`.
        // `len` is the size obtained from GetFileSizeEx before mapping.
        unsafe { self.slice.as_ref() }
    }

    #[inline]
    pub(crate) fn stream<'m>(&'m self) -> MappedFileReader<'m> {
        let ptr = self.start_ptr();

        MappedFileReader {
            mapping: self,
            cursor: ptr,
            end: unsafe { ptr.byte_add(self.len()) },
        }
    }

    #[inline(always)]
    pub(crate) fn is_empty(&self) -> bool {
        self.slice.is_empty()
    }

    #[inline(always)]
    pub(crate) fn len(&self) -> usize {
        self.slice.len()
    }

    #[inline(always)]
    fn start_ptr(&self) -> NonNull<u8> {
        self.slice.cast::<u8>()
    }
}

#[cfg(windows)]
impl Drop for ReadMappedFile {
    #[inline]
    fn drop(&mut self) {
        unsafe {
            UnmapViewOfFile(MEMORY_MAPPED_VIEW_ADDRESS {
                Value: self.slice.cast::<u8>().cast().as_ptr(),
            });

            CloseHandle(self.mapping.as_ptr());
            CloseHandle(self.file.as_ptr());
        }
    }
}

pub(crate) struct MappedFileReader<'m> {
    mapping: &'m ReadMappedFile,
    cursor: NonNull<u8>,
    end: NonNull<u8>,
}

const MAX_OFFSET64: u64 = isize::MAX as u64;
const SEEK_OUT_OF_BOUNDS: &str = "seek position out of bounds";

impl<'m> MappedFileReader<'m> {
    #[inline]
    fn read_inner(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        if buf.is_empty() {
            return Ok(0);
        }

        let remaining = unsafe { self.end.byte_offset_from_unsigned(self.cursor) };

        if remaining == 0 {
            return Ok(0);
        }

        let len = remaining.min(buf.len());

        unsafe {
            let dest = NonNull::new_unchecked(buf.as_mut_ptr());

            self.cursor.copy_to_nonoverlapping(dest, len);

            self.cursor = self.cursor.byte_add(len);
        }

        Ok(len)
    }

    #[inline]
    fn seek_inner(&mut self, pos: SeekFrom) -> io::Result<u64> {
        let len = self.mapping.len();

        match pos {
            SeekFrom::Start(offset) => {
                if offset > MAX_OFFSET64 || offset > len as u64 {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        SEEK_OUT_OF_BOUNDS,
                    ));
                }

                self.cursor = unsafe { self.mapping.start_ptr().byte_add(offset as usize) };

                Ok(offset)
            }
            SeekFrom::End(offset) => {
                if offset >= 0 {
                    if offset > 0 {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            SEEK_OUT_OF_BOUNDS,
                        ));
                    }

                    self.cursor = self.end;

                    Ok(len as u64)
                } else {
                    let offset = offset.unsigned_abs();

                    if offset > len as u64 {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidInput,
                            SEEK_OUT_OF_BOUNDS,
                        ));
                    }

                    self.cursor = unsafe { self.end.byte_sub(offset as usize) };

                    Ok(len as u64 - offset)
                }
            }
            SeekFrom::Current(offset) => self.seek_relative_inner(offset),
        }
    }

    #[inline]
    fn rewind_inner(&mut self) -> io::Result<()> {
        self.cursor = self.mapping.start_ptr();

        Ok(())
    }

    #[inline]
    fn stream_position_inner(&self) -> u64 {
        let offset = unsafe {
            self.cursor
                .byte_offset_from_unsigned(self.mapping.start_ptr())
        };

        offset as u64
    }

    #[inline]
    fn seek_relative_inner(&mut self, offset: i64) -> io::Result<u64> {
        let current = self.stream_position_inner();

        if offset >= 0 {
            let offset = offset as u64;
            let remaining = self.mapping.len() as u64 - current;

            if offset > MAX_OFFSET64 - current || offset > remaining {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    SEEK_OUT_OF_BOUNDS,
                ));
            }

            self.cursor = unsafe { self.cursor.byte_add(offset as usize) };

            Ok(current + offset)
        } else {
            let offset = offset.unsigned_abs();

            if offset > current {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    SEEK_OUT_OF_BOUNDS,
                ));
            }

            self.cursor = unsafe { self.cursor.byte_sub(offset as usize) };

            Ok(current - offset)
        }
    }
}

impl<'m> std::io::Read for MappedFileReader<'m> {
    #[inline]
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        self.read_inner(buf)
    }
}

impl<'m> std::io::Seek for MappedFileReader<'m> {
    #[inline]
    fn seek(&mut self, pos: SeekFrom) -> io::Result<u64> {
        self.seek_inner(pos)
    }

    #[inline]
    fn rewind(&mut self) -> io::Result<()> {
        self.rewind_inner()
    }

    #[inline]
    fn stream_position(&mut self) -> io::Result<u64> {
        Ok(self.stream_position_inner())
    }

    #[inline]
    fn seek_relative(&mut self, offset: i64) -> io::Result<()> {
        self.seek_relative_inner(offset)?;

        Ok(())
    }
}

impl<'m> smol::io::AsyncRead for MappedFileReader<'m> {
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        _ctx: &mut Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        Poll::Ready(unsafe { self.get_unchecked_mut().read_inner(buf) })
    }
}

impl<'m> smol::io::AsyncSeek for MappedFileReader<'m> {
    #[inline]
    fn poll_seek(
        self: Pin<&mut Self>,
        _ctx: &mut Context<'_>,
        pos: SeekFrom,
    ) -> Poll<io::Result<u64>> {
        Poll::Ready(unsafe { self.get_unchecked_mut().seek_inner(pos) })
    }
}
