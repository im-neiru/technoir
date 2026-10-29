use std::{io::Read, path::Path};

use semver::Version;
use zip::ZipArchive;

use crate::{
    errors::PluginInstallError,
    plugin::{
        consts::MANIFEST_NAME,
        package::{Plugin, PluginManifest},
        registry::{
            PluginRegistry,
            meta::{Entry, IndexFile},
        },
    },
    utils::ReadMappedFile,
};

impl PluginRegistry {
    #[inline]
    pub(crate) async fn install_packages(&self, paths: &[&Path]) -> Result<(), PluginInstallError> {
        let mut index_file = self.index_file();

        for path in paths.iter() {
            let manifest = Self::read_manifest(path)?;
            self.upsert_entry(&mut index_file, manifest)?;
        }

        Ok(())
    }

    fn read_manifest(path: &Path) -> Result<PluginManifest, PluginInstallError> {
        let mapped = ReadMappedFile::open(path)?;

        if mapped.is_empty() {
            return Err(PluginInstallError::InvalidPluginPackage);
        }

        let Ok(mut archive) = ZipArchive::new(mapped.stream()) else {
            return Err(PluginInstallError::InvalidPluginPackage);
        };

        let Ok(mut manifest_file) = archive.by_name(MANIFEST_NAME) else {
            return Err(PluginInstallError::InvalidPluginPackage);
        };

        let mut bytes = Vec::new();
        manifest_file.read_to_end(&mut bytes)?;

        cbor2::from_slice(&bytes).map_err(|_| PluginInstallError::InvalidPluginPackage)
    }

    fn upsert_entry<'s>(
        &self,
        index_file: &mut IndexFile<'s>,
        manifest: PluginManifest<'s>,
    ) -> Result<(), PluginInstallError> {
        let Plugin {
            name,
            id,
            version,
            api_version: _api_version,
            plugin_type,
            entry_point: _entry_point,
            description,
            author,
            license,
            pipelines: _pipelines,
            icon,
            tags,
        } = manifest.plugin;

        if let Some(existing) = index_file.entries.get(&id)
            && !should_upgrade(&existing.version, &version)
        {
            return Ok(());
        }

        self.programs.push(bytes)

        index_file.entries.insert(
            id,
            Entry {
                name,
                version,
                plugin_type,
                description,
                author,
                license,
                icon,
                tags,
                code_id: 0,        // todo
                pipelines: vec![], // todo
            },
        );

        Ok(())
    }
}

fn should_upgrade(existing: &Version, incoming: &Version) -> bool {
    if incoming <= existing {
        return false;
    }

    if existing.pre.is_empty() && !incoming.pre.is_empty() {
        return false;
    }

    true
}
