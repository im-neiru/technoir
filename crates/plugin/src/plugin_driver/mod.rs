use triomphe::Arc;
use wasmtime::Engine;

pub struct PluginDriver {
    inner: Arc<Inner>,
}

struct Inner {
    engine: Engine,
}
