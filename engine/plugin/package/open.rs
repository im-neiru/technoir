use std::{
    io::{Cursor, Read},
    path::Path,
};

use image::EncodableLayout;
use mlua::{Function, Lua, Table};
use zip::ZipArchive;

use crate::{plugin::factory::PluginFactory, utils::ReadMappedFile};

const MANIFEST_NAME: &str = "technoir.cbor";

pub(crate) struct PluginPackage {
    mapped: ReadMappedFile,
    manifest: super::PluginManifest,
}

impl PluginPackage {
    #[inline]
    pub(crate) fn info(&self) -> &super::Plugin {
        &self.manifest.plugin
    }

    #[inline]
    pub(crate) fn into_factory(self) -> Option<PluginFactory> {
        let Self { mapped, manifest } = self;

        let entry_point = manifest.plugin.entry_point;

        let mut archive = ZipArchive::new(Cursor::new(mapped.as_bytes())).ok()?;
        let mut file = archive.by_name(&entry_point).ok()?;

        let mut code = Vec::with_capacity(file.size() as usize);
        file.read_to_end(&mut code).ok()?;

        let instance_name = manifest.plugin.id.replace('.', "_");

        let lua = Lua::new();
        let init: Function = lua
            .load(code.as_bytes())
            .set_name(instance_name)
            .eval()
            .unwrap();

        Some(PluginFactory {
            lua,
            init,
            kind: manifest.plugin.plugin_type,
        })
    }
}

impl super::PluginManifest {
    #[inline]
    pub(crate) async fn read_package(path: &Path) -> Option<PluginPackage> {
        let mapped = ReadMappedFile::open(path).await.ok()?;

        if mapped.is_empty() {
            return None;
        }

        let manifest = {
            let mut archive = ZipArchive::new(Cursor::new(mapped.as_bytes())).ok()?;
            let mut file = archive.by_name(MANIFEST_NAME).ok()?;

            let mut bytes = Vec::new();
            file.read_to_end(&mut bytes).ok()?;

            cbor2::from_slice(&bytes).ok()?
        };

        Some(PluginPackage { mapped, manifest })
    }
}
