mod driver;
mod event_loop;
mod renderer;
mod screen;
mod wallpaper_target;

pub(crate) use driver::WallpaperDriver;
pub use renderer::{PrepareContext, ShaderSource, TextureIndex, TextureSize};
pub(crate) use screen::{Screen, ScreenBounds};
use wallpaper_target::WallpaperTarget;

pub trait WallpaperProvider {
    fn create(prepare_context: &mut PrepareContext) -> Self
    where
        Self: Sized;
}
