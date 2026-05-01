#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureIndex(usize);

pub(crate) enum TextureKey {}

impl super::WallpaperRenderer {
    pub fn get_texture(&self, index: TextureIndex) -> Option<&wgpu::Texture> {
        let (_, texture) = self.textures.get_index(index.0)?;

        Some(texture)
    }
}
