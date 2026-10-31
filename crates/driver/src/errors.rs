#[derive(Clone, Copy, Debug)]
pub enum StartupError {
    WindowOs(WindowsOsError),
}

#[derive(Clone, Copy, Debug)]
pub enum WindowsOsError {
    FailedToGetModuleHandle,
    FailedToRegisterWindowClass,
    FailedToCreateWindow { error_code: u32 },
}
