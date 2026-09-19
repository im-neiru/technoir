use std::{borrow::Cow, pin::Pin};

use semver::Version;
use smol::io::{self, AsyncWriteExt};

use crate::{
    errors::PluginRegistryEncodeError,
    utils::{BinWrite, StagingBuffer},
};

pub(super) struct IndexFile {}

pub(super) struct ShallowMeta<'s> {
    name: Cow<'s, str>,
    id: Cow<'s, str>,
    version: Version,
}

impl<'s> BinWrite for ShallowMeta<'s> {
    type Error = PluginRegistryEncodeError;

    async fn write_meta<S>(
        &self,
        staging_buf: &mut StagingBuffer,
        mut stream: Pin<&mut S>,
    ) -> Result<(), Self::Error>
    where
        S: io::AsyncWrite + io::AsyncSeek,
    {
        let reserve = usize::max(self.name.len(), self.id.len()) + 1;

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

        Ok(())
    }
}
