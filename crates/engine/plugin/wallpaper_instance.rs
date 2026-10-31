use mlua::{Function, Lua, Table, UserData, UserDataMethods};

pub(crate) struct WallpaperInstance {
    pub(super) table: Table,
    pub(super) fn_draw: Function,
}

impl WallpaperInstance {
    pub(crate) fn draw(&self, lua: &Lua, ctx: &DrawWallpaperContext) {
        lua.scope(|scope| {
            let ctx_handle = scope.create_userdata_ref(ctx)?;

            self.fn_draw.call::<()>((&self.table, ctx_handle))
        })
        .expect("failed to execute lua draw method");
    }
}

#[derive(Clone)]
pub struct DrawWallpaperContext {
    pub frame_count: u64,
}

impl UserData for DrawWallpaperContext {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("test", |_, this, ()| {
            println!("[Rust] ctx:test() called! Frame: {}", this.frame_count);
            Ok(())
        });
    }
}
