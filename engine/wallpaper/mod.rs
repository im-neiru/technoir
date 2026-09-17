mod driver;
mod event_loop;
mod renderer;
mod screen;
// mod wallpaper_loader;
mod wallpaper_target;

pub(crate) use driver::WallpaperDriver;
pub use renderer::{
    CleanupContext, PrepareContext, ShaderIndex, ShaderSource, TextureIndex, TextureSize,
};
pub(in crate::wallpaper) use screen::{Screen, ScreenBounds};
pub(in crate::wallpaper) use wallpaper_target::WallpaperTarget;

pub trait WallpaperProvider: Send + Sync {
    fn create(prepare_context: &mut PrepareContext) -> Self
    where
        Self: Sized;

    fn clean_up(&self, _cleanup_context: &mut CleanupContext) {}
}
