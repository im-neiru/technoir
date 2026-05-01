use engine::wallpaper::{CleanupContext, PrepareContext, ShaderSource, TextureIndex, WallpaperProvider};

pub struct CosmicBubble {
    bg_idx: TextureIndex,
    noise_idx: TextureIndex,
}

impl WallpaperProvider for CosmicBubble {
    fn create(prepare_context: &mut PrepareContext) -> Self {
        let (bg_idx, _tex_bg) = prepare_context.load_system_wallpaper_as_texture();
        let (noise_idx, _tex_noise) = prepare_context.create_grain_noise_texture();

        let (_shader_idx, _shader) = prepare_context.load_built_in_shader(ShaderSource::CosmicBubble);

        Self { bg_idx, noise_idx }
    }

    fn clean_up(&self, cleanup_context: &mut CleanupContext) {
        cleanup_context.release_texture(self.bg_idx);
        cleanup_context.release_texture(self.noise_idx);
    }
}

#[unsafe(no_mangle)]
#[allow(clippy::missing_safety_doc)]
pub unsafe extern "Rust" fn create(ctx: *mut PrepareContext) -> Box<dyn WallpaperProvider> {
    let ctx = unsafe { &mut *ctx };
    let wallpaper = CosmicBubble::create(ctx);

    Box::new(wallpaper)
}
