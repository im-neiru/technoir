mod ui_window;
mod wallpaper_window;
mod worker;

pub(super) use ui_window::UiWindow;
pub(super) use wallpaper_window::WallpaperWindow;
pub(super) use worker::Worker;

use super::errors::StartupError;

pub fn run() -> Result<(), StartupError> {
    let worker = Worker::new().map_err(StartupError::WindowOs)?;

    let ui_window = worker.create_ui_window().map_err(StartupError::WindowOs)?;

    ui_window.show();

    worker.run();

    Ok(())
}
