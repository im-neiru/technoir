use mlua::{Function, Lua, ObjectLike, Table, UserData, UserDataFields};

use super::{package::PluginType, wallpaper_instance::WallpaperInstance};

pub(crate) struct PluginFactory {
    pub(super) lua: Lua,
    pub(super) table: Table,
    pub(super) kind: PluginType,
}

impl PluginFactory {
    pub(crate) fn new_wallpaper(&self) -> WallpaperInstance {
        let init_ctx = MakeWallpaperContext {
            width: 1920,
            height: 1080,
        };

        let instance: Table = self
            .table
            .call_method("make", (init_ctx,))
            .expect("failed to run factory.make()");

        let fn_draw: Function = instance
            .get("draw")
            .expect("missing 'draw' method on Wallpaper instance");

        WallpaperInstance {
            table: instance,
            fn_draw,
        }
    }
}

#[derive(Clone)]
pub struct MakeWallpaperContext {
    pub width: u32,
    pub height: u32,
}

impl UserData for MakeWallpaperContext {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |_, this| Ok(this.width));
        fields.add_field_method_get("height", |_, this| Ok(this.height));
    }
}
