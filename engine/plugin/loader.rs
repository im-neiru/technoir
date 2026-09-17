use mlua::{Lua, prelude::*};

pub(crate) struct PluginLoader {
    lua: Lua,
    compiler: LuaCompiler,
}

impl PluginLoader {
    #[inline]
    pub(crate) fn new() -> Self {
        let lua = Lua::new();
        let compiler = LuaCompiler::new()
            .set_optimization_level(2)
            .set_debug_level(1);

        Self { lua, compiler }
    }
}
