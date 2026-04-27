pub struct Config {
    pub(crate) open_manager: bool,
}

impl Config {
    pub fn load() -> Self {
        Self {
            open_manager: false,
        }
    }
}
