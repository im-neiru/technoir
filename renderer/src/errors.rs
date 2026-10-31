use miette::Diagnostic;
use thiserror::Error;

#[derive(Clone, Copy, Debug, Error, Diagnostic, PartialEq, Eq, Hash)]
pub enum GraphicsInitError {
    #[error("failed to retrieve the window handle")]
    #[diagnostic(code(graphics::init::get_window_handle))]
    GetWindowHandle,

    #[error("no compatible graphics device was found")]
    #[diagnostic(code(graphics::init::no_compatible_device))]
    NoCompatibleDevice,

    #[error("failed to create the graphics device")]
    #[diagnostic(code(graphics::init::create_device))]
    CreateDevice,

    #[error("failed to create the rendering surface")]
    #[diagnostic(code(graphics::init::create_surface))]
    CreateSurface,
}
