use std::{borrow::Cow, path::Path, pin::Pin};

use semver::Version;
use smol::{
    fs,
    io::{self, AsyncWriteExt},
};

use crate::{
    errors::PluginRegistryEncodeError,
    utils::{BinWrite, StagingBuffer},
};

pub(crate) struct IndexFile<'s> {
    shallows: Vec<ShallowMeta<'s>>,
}

impl<'s> IndexFile<'s> {
    pub(crate) fn new() -> Self {
        Self {
            shallows: Vec::new(),
        }
    }

    pub(crate) async fn write(
        &self,
        path: impl AsRef<Path>,
    ) -> Result<(), PluginRegistryEncodeError> {
        let mut file = Box::pin(fs::File::open(path).await?);

        let mut staging_buf = StagingBuffer::new(4);

        // shallows count
        staging_buf.put_leb128(self.shallows.len() as u64);
        file.write(staging_buf.as_bytes()).await?;

        // shollow items
        for s in self.shallows.iter() {
            s.write(&mut staging_buf, file.as_mut()).await?;
        }

        Ok(())
    }
}

pub(super) struct ShallowMeta<'s> {
    name: Cow<'s, str>,
    id: Cow<'s, str>,
    version: Version,
}

impl<'s> BinWrite for ShallowMeta<'s> {
    type Error = PluginRegistryEncodeError;

    async fn write<S>(
        &self,
        staging_buf: &mut StagingBuffer,
        mut stream: Pin<&mut S>,
    ) -> Result<(), Self::Error>
    where
        S: io::AsyncWrite + io::AsyncSeek,
    {
        let version_str = self.version.to_string();
        let reserve = usize::max(self.name.len(), self.id.len()).max(version_str.len()) + 1;

        staging_buf.begin(reserve);

        // name
        if staging_buf.put_str_u8_len(&self.name, 128).is_err() {
            return Err(PluginRegistryEncodeError::FieldExceedMaxLength("name"));
        };

        stream.write(staging_buf.as_bytes()).await?;

        // id
        if staging_buf.put_str_u8_len(&self.id, 128).is_err() {
            return Err(PluginRegistryEncodeError::FieldExceedMaxLength("id"));
        };

        stream.write(staging_buf.as_bytes()).await?;

        // version
        staging_buf.put_str_leb128(&version_str);
        stream.write(staging_buf.as_bytes()).await?;

        Ok(())
    }
}
