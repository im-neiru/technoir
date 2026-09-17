use mlua::Lua;
use smol::io::{AsyncRead, AsyncReadExt};

use super::loader::PluginLoader;

pub(super) struct Module {
    lua: Lua,
    code: Box<[u8]>,
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
