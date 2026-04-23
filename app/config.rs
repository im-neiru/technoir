pub(crate) struct Config {
    pub(crate) open_manager: bool,
}

impl Config {
    pub(crate) fn load() -> Self {
        Self { open_manager: true }
    }
}
