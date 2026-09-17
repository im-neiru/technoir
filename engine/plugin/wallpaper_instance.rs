use mlua::{Function, Lua, Table};

use crate::plugin::module::WallpaperDrawContext;

pub(crate) struct WallpaperInstance {
    pub(super) table: Table,
    pub(super) draw: Function,
}

impl WallpaperInstance {
    pub(crate) fn draw(&self, lua: &Lua, ctx: &WallpaperDrawContext) {
        lua.scope(|scope| {
            let ctx_handle = scope.create_userdata_ref(ctx)?;

            self.draw.call::<()>((&self.table, ctx_handle))
        })
        .expect("failed to execute lua draw method");
    }
}
