use image::{GenericImageView, imageops::FilterType};
use wgpu::{TexelCopyBufferLayout, Texture, TextureDimension, TextureFormat, TextureUsages};

impl super::Renderer {
    pub async fn load_texture_from_image_file<P: AsRef<std::path::Path>>(
        &self,
        name: impl AsRef<str>,
        path: P,
        monochrome: bool,
        fit: bool,
        resize_target: Option<TextureSize>,
    ) -> Result<Texture, LoadTextureImageError> {
        let mut img = {
            let bytes = smol::fs::read(path).await?;

            image::load_from_memory(&bytes)?
        };

        let (width, height) = if let Some(TextureSize { width, height }) = resize_target {
            if width == 0 || height == 0 {
                return Err(LoadTextureImageError::InvalidTextureSize);
            }

            if fit {
                img = img.resize_to_fill(width, height, FilterType::Lanczos3);
            } else {
                img = img.resize_exact(width, height, FilterType::Lanczos3);
            }

            (width, height)
        } else {
            let (width, height) = img.dimensions();

            (width, height)
        };

        let texture = self.device.create_texture(&wgpu::TextureDescriptor {
            label: Some(name.as_ref()),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            format: if monochrome {
                TextureFormat::R8Unorm
            } else {
                TextureFormat::Rgba8Unorm
            },
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
        });

        let extent = wgpu::Extent3d {
            width,
            height,
            depth_or_array_layers: 1,
        };

        if monochrome {
            let img = img.to_luma8();

            self.queue.write_texture(
                texture.as_image_copy(),
                img.as_raw(),
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width),
                    rows_per_image: Some(height),
                },
                extent,
            );
        } else {
            let img = img.to_rgba8();

            self.queue.write_texture(
                texture.as_image_copy(),
                img.as_raw(),
                TexelCopyBufferLayout {
                    offset: 0,
                    bytes_per_row: Some(width * 4),
                    rows_per_image: Some(height),
                },
                extent,
            );
        }

        Ok(texture)
    }

    pub fn scale_texture(&self, texture_size: TextureSize) -> glam::Vec2 {
        let screen_aspect = self.get_aspect_ratio();
        let tex_aspect = texture_size.width as f32 / texture_size.height as f32;

        if screen_aspect > tex_aspect {
            glam::vec2(1.0, tex_aspect / screen_aspect)
        } else {
            glam::vec2(screen_aspect / tex_aspect, 1.0)
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum LoadTextureImageError {
    #[error("Failed to read image file")]
    Io(#[from] std::io::Error),

    #[error("Failed to decode image")]
    Image(#[from] image::ImageError),

    #[error("Invalid texture size")]
    InvalidTextureSize,
}

#[derive(Debug, Clone, Copy)]
pub struct TextureSize {
    pub width: u32,
    pub height: u32,
}
