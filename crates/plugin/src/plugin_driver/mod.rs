mod ffi;
mod package;

#[cfg(test)]
mod tests;

use std::path::Path;

use triomphe::Arc;
use wasmtime::{Config, Engine, Instance, Linker, Module, Store};

use self::ffi::HostState;
pub use self::package::PluginPackage;

#[derive(Clone)]
pub struct PluginDriver {
    engine: Arc<Engine>,
}

pub struct PluginInstance {
    store: Store<HostState>,
    instance: Instance,
}

impl PluginDriver {
    pub fn new() -> wasmtime::Result<Self> {
        let engine = Engine::new(&Config::new())?;
        Ok(Self {
            engine: Arc::new(engine),
        })
    }

    pub fn load(&self, package: &PluginPackage) -> wasmtime::Result<PluginInstance> {
        let module = Module::from_file(&self.engine, package.wasm_module_path())?;
        let mut linker = Linker::<HostState>::new(&self.engine);
        ffi::add_to_linker(&mut linker)?;

        let mut store = Store::new(&self.engine, HostState::default());
        let instance = linker.instantiate(&mut store, &module)?;

        Ok(PluginInstance { store, instance })
    }

    pub fn load_directory(
        &self,
        extracted_package_directory: impl AsRef<Path>,
    ) -> wasmtime::Result<PluginInstance> {
        self.load(&PluginPackage::from_extracted_directory(
            extracted_package_directory,
        ))
    }
}

impl PluginInstance {
    pub fn call_unit_export(&mut self, export: &str) -> wasmtime::Result<()> {
        let function = self
            .instance
            .get_typed_func::<(), ()>(&mut self.store, export)?;
        function.call(&mut self.store, ())
    }

    pub fn logs(&self) -> &[String] {
        &self.store.data().logs
    }
}
