use std::path::{Path, PathBuf};

const WASM_MODULE_FILE: &str = "plugin.wasm";

#[derive(Clone, Debug)]
pub struct PluginPackage {
    root: PathBuf,
    wasm_module: PathBuf,
}

impl PluginPackage {
    pub fn from_extracted_directory(directory: impl AsRef<Path>) -> Self {
        let root = directory.as_ref().to_path_buf();
        let wasm_module = root.join(WASM_MODULE_FILE);
        Self { root, wasm_module }
    }

    pub fn from_wasm_file(wasm_module: impl AsRef<Path>) -> Self {
        let wasm_module = wasm_module.as_ref().to_path_buf();
        let root = wasm_module
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_default();
        Self { root, wasm_module }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn wasm_module_path(&self) -> &Path {
        &self.wasm_module
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::PluginPackage;

    #[test]
    fn extracted_package_uses_standard_module_name() {
        let package = PluginPackage::from_extracted_directory("package");
        assert_eq!(
            package.wasm_module_path(),
            Path::new("package").join("plugin.wasm")
        );
    }
}
