use std::{borrow::Cow, fs::File, io::Write, path::Path};

use mlua::prelude::*;
use smol::fs;
use zip::{CompressionMethod, ZipWriter, write::SimpleFileOptions};

use super::{manifest::PluginManifest, pipelines::Pipelines, wgsl_minifier::minify_wgsl};

pub struct Packager {
    compiler: LuaCompiler,
    zip_opt: SimpleFileOptions,
}

impl Packager {
    pub fn release_mode() -> Self {
        let compiler = LuaCompiler::new()
            .set_optimization_level(2)
            .set_type_info_level(0)
            .set_debug_level(0);

        let zip_opt = SimpleFileOptions::default()
            .compression_method(CompressionMethod::Zstd)
            .compression_level(Some(3));

        Self { compiler, zip_opt }
    }

    pub async fn pack<P>(&self, project: P, output: Option<P>)
    where
        P: AsRef<Path>,
    {
        let manifest = fs::read_to_string(project.as_ref().join("technoir.toml"))
            .await
            .unwrap();

        let mut manifest: PluginManifest = toml::from_str(&manifest).unwrap();

        let code;
        {
            let luau = fs::read(project.as_ref().join(manifest.plugin.entry_point.as_str()))
                .await
                .unwrap();

            code = self.compiler.compile(luau).unwrap();

            manifest.plugin.entry_point = String::from("main.luauc");
        }

        let pipeline_path = project.as_ref().join(manifest.plugin.pipelines.as_str());

        let pipelines = fs::read_to_string(pipeline_path).await.unwrap();
        let mut pipelines: Pipelines = toml::from_str(&pipelines).unwrap();

        let mut shaders_list = Vec::new();
        let shader_dir = project.as_ref().join("shaders");

        for info in pipelines.0.values_mut() {
            let wgsl = fs::read_to_string(shader_dir.join(info.shader.as_str()))
                .await
                .unwrap();

            let wgsl = minify_wgsl(&wgsl).unwrap();

            let index = shaders_list.len();
            shaders_list.push(wgsl.into_boxed_str());

            info.shader = format!("{index:x}.wgsl");
        }

        {
            let output: Cow<'_, Path> = output
                .as_ref()
                .map(|path| Cow::Borrowed(path.as_ref()))
                .unwrap_or_else(|| {
                    Cow::Owned(
                        project
                            .as_ref()
                            .join("build")
                            .join(manifest.plugin.id.as_str())
                            .with_extension("zip"),
                    )
                });

            if !output.parent().map(|p| p.exists()).unwrap_or(false) {
                fs::create_dir_all(output.parent().unwrap()).await.unwrap()
            }

            let pack = File::create(&output).unwrap();
            let mut zip = ZipWriter::new(pack);

            zip.start_file("main.luauc", self.zip_opt).unwrap();
            zip.write_all(&code).unwrap();

            let pipelines_bytes = cbor2::to_vec(&pipelines).unwrap();

            zip.start_file("pipelines.cbor", self.zip_opt).unwrap();
            zip.write_all(&pipelines_bytes).unwrap();

            let manifest_bytes = cbor2::to_vec(&manifest).unwrap();

            zip.start_file("manifest.cbor", self.zip_opt).unwrap();
            zip.write_all(&manifest_bytes).unwrap();

            for (index, shader) in shaders_list.into_iter().enumerate() {
                zip.start_file(format!("shaders/{index:x}.wgsl"), self.zip_opt)
                    .unwrap();

                zip.write_all(shader.as_bytes()).unwrap();
            }

            zip.finish().unwrap();
        }
    }
}
