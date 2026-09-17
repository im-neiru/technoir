use mlua::{Function, Lua, Table, UserData, UserDataFields, UserDataMethods, prelude::*};

use smol::io::{AsyncRead, AsyncReadExt};

use super::{loader::PluginLoader, wallpaper_instance::WallpaperInstance};

pub(super) struct Module {
    pub(super) lua: Lua,
    pub(super) code: Box<[u8]>,
}

impl Module {
    pub(super) fn instantiate_wallpaper(&self) -> WallpaperInstance {
        let init_ctx = WallpaperInitContext {
            width: 1920,
            height: 1080,
        };

        let factory: Table = self
            .lua
            .load(self.code.as_ref())
            .set_name("wallpaper")
            .eval()
            .unwrap();

        let instance: Table = factory
            .call_method("init", (init_ctx,))
            .expect("failed to run factory.init()");

        let draw: Function = instance
            .get("draw")
            .expect("missing 'draw' method on Wallpaper instance");

        WallpaperInstance {
            table: instance,
            draw,
        }
    }
}

impl PluginLoader {
    pub(super) async fn load_module(&self, mut read: impl AsyncRead + Unpin) -> Module {
        let mut source = Vec::new();

        read.read_to_end(&mut source).await.unwrap();

        let code = self.compiler.compile(&source).unwrap();

        let lua = Lua::new();

        Module {
            lua,
            code: code.into_boxed_slice(),
        }
    }
}

#[derive(Clone)]
pub struct WallpaperInitContext {
    pub width: u32,
    pub height: u32,
}

impl UserData for WallpaperInitContext {
    fn add_fields<F: UserDataFields<Self>>(fields: &mut F) {
        fields.add_field_method_get("width", |_, this| Ok(this.width));
        fields.add_field_method_get("height", |_, this| Ok(this.height));
    }
}

#[derive(Clone)]
pub struct WallpaperDrawContext {
    pub frame_count: u64,
}

impl UserData for WallpaperDrawContext {
    fn add_methods<M: UserDataMethods<Self>>(methods: &mut M) {
        methods.add_method("test", |_, this, ()| {
            println!("[Rust] ctx:test() called! Frame: {}", this.frame_count);
            Ok(())
        });
    }
}
