use core::pin::Pin;

use smol::io;

use crate::errors::StagingBufferError;

pub(crate) trait BinWrite {
    type Error;

    async fn write<S>(
        &self,
        staging_buf: &mut StagingBuffer,
        stream: Pin<&mut S>,
    ) -> Result<(), Self::Error>
    where
        S: io::AsyncWrite + io::AsyncSeek;
}

pub(crate) struct StagingBuffer(Vec<u8>);

impl StagingBuffer {
    #[inline]
    pub(crate) fn new(cap: usize) -> Self {
        Self(Vec::with_capacity(cap))
    }

    #[inline]
    pub(crate) fn put_str_u8_len(
        &mut self,
        s: impl AsRef<str>,
        max: u8,
    ) -> Result<(), StagingBufferError> {
        self.0.clear();

        let bytes = s.as_ref().as_bytes();

        if bytes.len() >= max as usize {
            return Err(StagingBufferError::BytesExceedMaxLength);
        }

        self.0.push(bytes.len() as u8);
        self.0.extend_from_slice(bytes);

        Ok(())
    }

    #[inline]
    pub(crate) fn put_str_leb128(&mut self, s: impl AsRef<str>) {
        let bytes = s.as_ref().as_bytes();

        encode_leb128(bytes.len() as u64, &mut self.0);
        self.0.extend_from_slice(bytes);
    }

    #[inline]
    pub(crate) fn put_leb128(&mut self, l: u64) {
        encode_leb128(l, &mut self.0);
    }

    #[inline]
    pub(crate) fn begin(&mut self, cap: usize) {
        if let Some(additional) = cap.checked_sub(self.0.capacity()) {
            self.0.reserve(additional);
        }
    }

    #[inline]
    pub(crate) fn as_bytes(&self) -> &[u8] {
        self.0.as_slice()
    }
}

#[inline]
fn encode_leb128(mut value: u64, out: &mut Vec<u8>) {
    while value >= 0x80 {
        out.push((value as u8) | 0x80);
        value >>= 7;
    }

    out.push(value as u8);
}
