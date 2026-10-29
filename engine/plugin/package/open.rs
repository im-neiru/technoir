use std::{
    io::{Cursor, Read},
    path::Path,
};

use image::EncodableLayout;
use mlua::{Function, Lua};
use rapidhash::{HashMapExt, RapidHashMap};
use zip::ZipArchive;

use crate::{plugin::factory::PluginFactory, utils::ReadMappedFile};

const MANIFEST_NAME: &str = "technoir.cbor";

pub(crate) struct PluginPackage<'s> {
    mapped: ReadMappedFile,
    manifest: super::PluginManifest<'s>,
}

impl<'s> PluginPackage<'s> {
    #[inline]
    pub(crate) fn info(&self) -> &super::Plugin {
        &self.manifest.plugin
    }
}
