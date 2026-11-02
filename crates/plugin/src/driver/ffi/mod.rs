mod logcat;

use wasmtime::Linker;

#[derive(Default)]
pub(super) struct HostState {
    pub(super) logs: Vec<String>,
}

pub(super) fn add_to_linker(linker: &mut Linker<HostState>) -> wasmtime::Result<()> {
    logcat::add_to_linker(linker)
}
