use std::path::Path;

use super::{PluginDriver, PluginPackage};

#[test]
fn rain_logcat_is_intercepted() -> wasmtime::Result<()> {
    let module_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../_build/wasm/debug/build/im-neiru/rain/gen/gen.wasm");
    let driver = PluginDriver::new()?;
    let package = PluginPackage::from_wasm_file(module_path);
    let mut plugin = driver.load(&package)?;

    plugin.call_unit_export("prepare")?;
    plugin.call_unit_export("close")?;

    println!("{:#?}", plugin.logs());
    Ok(())
}
