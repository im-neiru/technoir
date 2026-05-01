use engine::wallpaper::{PrepareContext, ShaderSource, TextureIndex, WallpaperProvider};

pub struct CosmicBubble {
    bg_idx: TextureIndex,
    noise_idx: TextureIndex,
}

impl WallpaperProvider for CosmicBubble {
    fn create(prepare_context: &mut PrepareContext) -> Self {
        let (bg_idx, _tex_bg) = prepare_context.load_system_wallpaper_as_texture();
        let (noise_idx, _tex_noise) = prepare_context.create_grain_noise_texture();

        let (shader_idx, shader) = prepare_context.load_built_in_shader(ShaderSource::CosmicBubble);

        Self { bg_idx, noise_idx }
    }
}
