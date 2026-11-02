mod api;
mod ext_wallpaper;
mod instance_state;
mod package;

use std::path::Path;

use triomphe::Arc;
use wasmtime::{
    Config, Engine, Store,
    component::{Component, Linker},
};

pub use self::api::{
    INITIAL_RECORD_CAPACITY, MAX_RECORD_COUNT, TraceField, TraceLevel, TraceRecord,
    TraceRecordKind, TraceValue, TracingBuffer,
};
use self::instance_state::InstanceState;
pub use self::package::PluginPackage;

#[derive(Clone)]
pub struct PluginDriver {
    engine: Arc<Engine>,
}

pub struct PluginInstance {
    store: Store<InstanceState>,
    instance: ext_wallpaper::bindings::ExtWallpaper,
}

impl PluginDriver {
    pub fn new() -> wasmtime::Result<Self> {
        let mut config = Config::new();
        config.wasm_component_model(true);
        let engine = Engine::new(&config)?;
        Ok(Self {
            engine: Arc::new(engine),
        })
    }

    pub fn load(&self, package: &PluginPackage) -> wasmtime::Result<PluginInstance> {
        let component = Component::from_file(&self.engine, package.wasm_module_path())?;
        let mut linker = Linker::<InstanceState>::new(&self.engine);
        api::add_to_linker(&mut linker)?;

        let mut store = Store::new(&self.engine, InstanceState::default());
        let instance =
            ext_wallpaper::bindings::ExtWallpaper::instantiate(&mut store, &component, &linker)?;

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
        match export {
            "prepare" => self.instance.call_prepare(&mut self.store),
            "draw" => self.instance.call_draw(&mut self.store),
            "close" => self.instance.call_close(&mut self.store),
            _ => Err(wasmtime::format_err!(
                "unknown unit export `{export}` for plugin"
            )),
        }
    }

    pub fn records(&self) -> &std::collections::VecDeque<TraceRecord> {
        self.store.data().api.tracing.records()
    }

    pub fn tracing_buffer(&self) -> &TracingBuffer {
        &self.store.data().api.tracing
    }
}
