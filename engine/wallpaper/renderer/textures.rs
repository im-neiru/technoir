use wgpu::{
    Extent3d, TexelCopyBufferLayout, Texture, TextureDimension, TextureFormat, TextureUsages,
};

#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{SPI_GETDESKWALLPAPER, SystemParametersInfoW};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureIndex(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TextureKey {
    SystemWallpaper,
}

impl super::WallpaperRenderer {
    pub fn get_texture(&self, index: TextureIndex) -> Option<&wgpu::Texture> {
        let (_, texture) = self.textures.get_index(index.0)?;

        Some(texture)
    }

    #[cfg(target_os = "windows")]
    pub fn load_system_wallpaper_as_texture(&mut self) -> TextureIndex {
        if let Some((index, _key, _texture)) = self.textures.get_full(&TextureKey::SystemWallpaper)
        {
            return TextureIndex(index);
        }

        const CAPACITY: usize = 300;

        let mut path: Vec<u16> = Vec::with_capacity(CAPACITY);

        unsafe {
            SystemParametersInfoW(
                SPI_GETDESKWALLPAPER,
                CAPACITY as u32,
                path.as_mut_ptr() as _,
                0,
            );
        }

        let path_str = String::from_utf16_lossy(&path);

        let image = image::open(path_str).expect("Failed to open system wallpaper");

        let image = image.to_rgb8();

        let texture = self.create_texture_from_raw_bytes(
            "system_wallpaper",
            image.as_raw(),
            TextureFormat::Rgba8UnormSrgb,
            TextureUsages::TEXTURE_BINDING,
            image.width(),
            image.height(),
            image.width() * 3,
        );

        let (index, _) = self
            .textures
            .insert_full(TextureKey::SystemWallpaper, texture);

        TextureIndex(index)
    }

    #[allow(clippy::too_many_arguments)]
    fn create_texture_from_raw_bytes(
        &mut self,
        name: &str,
        bytes: &[u8],
        format: TextureFormat,
        usage: TextureUsages,
        width: u32,
        height: u32,
        bytes_per_row: u32,
    ) -> Texture {
        let extent = Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name),
            size: extent,
            format,
            usage: usage | TextureUsages::COPY_DST,
            view_formats: &[],
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
        });

        self.queue.write_texture(
            texture.as_image_copy(),
            bytes,
            TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(bytes_per_row),
                rows_per_image: Some(height),
            },
            extent,
        );

        texture
    }
}
