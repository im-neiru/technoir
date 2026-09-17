use std::{
    io::{Cursor, Read},
    path::Path,
};
use zip::ZipArchive;

use crate::utils::ReadMappedFile;

impl super::PluginManifest {
    #[inline]
    pub async fn peek_plugin(path: &Path) -> Option<super::Plugin> {
        let mapped = ReadMappedFile::open(path).await.ok()?;

        if mapped.is_empty() {
            return None;
        }

        let mut archive = ZipArchive::new(Cursor::new(mapped.as_bytes())).ok()?;
        let mut manifest_file = archive.by_name("technoir.cbor").ok()?;

        let mut manifest = Vec::new();

        manifest_file.read_to_end(&mut manifest).ok()?;

        let manifest: Self = cbor2::from_slice(&manifest).ok()?;

        Some(manifest.plugin)
    }
}
