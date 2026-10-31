use std::path::{Path, PathBuf};

use mlua::prelude::*;
use rapidhash::RapidHashSet;
use smol::{fs, stream::StreamExt};

use super::package::{Plugin, PluginManifest, PluginPackage};
use crate::utils::program_data_path;

pub(crate) struct PluginLoader {
    pub(super) compiler: LuaCompiler,
}

impl PluginLoader {
    #[inline]
    pub(crate) fn new() -> Self {
        let compiler = LuaCompiler::new()
            .set_optimization_level(2)
            .set_debug_level(1);

        Self { compiler }
    }

    #[inline]
    pub(crate) async fn enumerate_plugins(&self) -> Vec<PluginPackage> {
        let mut plugins = Vec::new();
        let mut seen_files = RapidHashSet::default();

        if let Some(paths) = std::env::var_os("TECHNOIR_PLUGINS") {
            for root in std::env::split_paths(&paths) {
                enumerate_root(&root, &mut seen_files, &mut plugins).await;
            }
        }

        if let Some(program_data) = program_data_path() {
            let root = program_data.join("Technoir").join("plugins");

            enumerate_root(&root, &mut seen_files, &mut plugins).await;
        }

        plugins
    }
}

#[inline]
async fn enumerate_root(
    root: &Path,
    seen_files: &mut RapidHashSet<PathBuf>,
    plugins: &mut Vec<PluginPackage>,
) {
    let Ok(mut entries) = fs::read_dir(root).await else {
        return;
    };

    while let Some(Ok(entry)) = entries.next().await {
        let path = entry.path();

        if !path.is_file() {
            continue;
        }

        let Some(extension) = path.extension() else {
            continue;
        };

        if !extension.eq_ignore_ascii_case("tnzip") {
            continue;
        }

        let path = match std::fs::canonicalize(&path) {
            Ok(path) => path,
            Err(_) => path,
        };

        if !seen_files.insert(path.clone()) {
            continue;
        }

        if let Some(plugin) = PluginManifest::read_package(&path).await {
            plugins.push(plugin);
        }
    }
}
