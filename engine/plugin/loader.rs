use mlua::{Lua, prelude::*};

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
}
