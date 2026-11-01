use crate::utils::ReadMappedFile;

const MANIFEST_NAME: &str = "technoir.cbor";

pub(crate) struct PluginPackage<'s> {
    mapped: ReadMappedFile,
    manifest: super::PluginManifest<'s>,
}

impl<'s> PluginPackage<'s> {
    #[inline]
    pub(crate) fn info(&self) -> &super::Plugin<'_> {
        &self.manifest.plugin
    }
}
