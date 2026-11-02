pub(in crate::driver) mod bindings {
    wasmtime::component::bindgen!({
        path: "../../wit",
        world: "ext-wallpaper",
    });
}
