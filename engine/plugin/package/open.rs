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
        let pipelines_path = manifest.plugin.pipelines;
        let instance_name = manifest.plugin.id.replace('.', "_");

        let mut archive = ZipArchive::new(Cursor::new(mapped.as_bytes())).ok()?;

        #[inline]
        fn read_file<R: std::io::Read + std::io::Seek>(
            archive: &mut ZipArchive<R>,
            path: &str,
        ) -> Option<Vec<u8>> {
            let mut file = archive.by_name(path).ok()?;
            let mut buf = Vec::with_capacity(file.size() as usize);
            file.read_to_end(&mut buf).ok()?;
            Some(buf)
        }

        let code = read_file(&mut archive, &entry_point)?;

        let lua = Lua::new();

        let init: Function = lua
            .load(code.as_bytes())
            .set_name(instance_name)
            .eval()
            .ok()?;

        let pipelines_buf = read_file(&mut archive, &pipelines_path)?;
        let pipelines: super::Pipelines = cbor2::from_slice(&pipelines_buf).ok()?;

        let mut shaders = RapidHashMap::with_capacity(pipelines.0.len());
        let mut shader_cache: RapidHashMap<String, String> =
            RapidHashMap::with_capacity(pipelines.0.len());

        for (name, pipeline) in pipelines.iter() {
            let shader = if let Some(shader) = shader_cache.get(&pipeline.shader) {
                shader.clone()
            } else {
                let buf = read_file(&mut archive, &pipeline.shader)?;
                let shader = String::from_utf8_lossy(&buf).into_owned();

                shader_cache.insert(pipeline.shader.clone(), shader.clone());

                shader
            };

            shaders.insert(name.to_string(), shader);
        }

        Some(PluginFactory {
            lua,
            init,
            kind: manifest.plugin.plugin_type,
            shaders,
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
