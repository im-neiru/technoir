use mlua::{Function, Lua, Table, UserData, UserDataFields};
use rapidhash::RapidHashMap;

use crate::graphics::Context;

use super::{package::PluginType, wallpaper_instance::WallpaperInstance};

pub(crate) struct PluginFactory {
    pub(super) lua: Lua,
    pub(super) init: Function,
    pub(super) kind: PluginType,
    pub(super) shaders: RapidHashMap<String, String>,
}

impl PluginFactory {
    pub(crate) fn new_wallpaper(&self, context: &Context) -> WallpaperInstance {
        let init_ctx = MakeWallpaperContext {
            ctx: context.clone(),
        };

        let instance: Table = self
            .init
            .call((init_ctx,))
            .expect("failed to construct wallpaper");

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
    ctx: Context,
}

impl UserData for MakeWallpaperContext {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {}
}
