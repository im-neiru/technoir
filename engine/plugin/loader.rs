use mlua::{Lua, prelude::*};
use smol::fs;

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
    pub(crate) async fn load_wallpaper(&self) {
        // for testing
        let file = fs::File::open("luau_sample/init.luau").await.unwrap();

        let module = self.load_module(file).await;
    }
}
