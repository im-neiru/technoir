use std::path::{Path, PathBuf};

use mlua::prelude::*;
use rapidhash::RapidHashSet;
use smol::{fs, stream::StreamExt};

use super::package::{Plugin, PluginManifest};
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
    pub(crate) async fn enumerate_plugins(&self) -> Box<[(super::package::Plugin, Box<Path>)]> {
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

        plugins.into_boxed_slice()
    }

    // olld will remove later
    #[inline]
    pub(crate) async fn load_wallpaper(&self) {
        // for testing
        let file = fs::File::open("sandbox/wallpaper/main.luau").await.unwrap();

        let module = self.load_module(file).await;

        let instance = module.instantiate_wallpaper();

        let mut draw_ctx = super::module::WallpaperDrawContext { frame_count: 1 };

        for i in 0..8 {
            instance.draw(&module.lua, &draw_ctx);
            draw_ctx.frame_count = i;
        }
    }
}

#[inline]
async fn enumerate_root(
    root: &Path,
    seen_files: &mut RapidHashSet<PathBuf>,
    plugins: &mut Vec<(Plugin, Box<Path>)>,
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

        if let Some(plugin) = PluginManifest::peek_plugin(&path).await {
            plugins.push((plugin, path.into_boxed_path()));
        }
    }
}
