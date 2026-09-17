use mlua::{Lua, Table};

pub struct PluginFactory {
    pub(super) lua: Lua,
    pub(super) table: Table,
}
