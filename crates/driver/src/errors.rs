use renderer::errors::GraphicsInitError;

#[derive(Clone, Copy, Debug)]
pub enum StartupError {
    WindowOs(WindowsOsError),
    GraphicsInit(GraphicsInitError),
}

#[derive(Clone, Copy, Debug)]
pub enum WindowsOsError {
    FailedToRetrieveDesktopHandles,
    FailedToGetModuleHandle,
    FailedToRegisterWindowClass,
    FailedToCreateWindow { error_code: u32 },
    FailedToCreateRenderThread { error_code: u32 },
}
