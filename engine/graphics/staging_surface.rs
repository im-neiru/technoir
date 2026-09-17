use core::num::NonZeroU16;

use wgpu::{
    Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView, TextureViewDescriptor,
};

pub struct StagingSurface {
    texture: Texture,
    pub(super) view: TextureView,
    format: TextureFormat,
    pub(super) width: u32,
    pub(super) height: u32,
}

impl super::Context {
    #[inline]
    pub fn create_target_surface(
        &self,
        width: NonZeroU16,
        height: NonZeroU16,
        format: TextureFormat,
    ) -> StagingSurface {
        let size = Extent3d {
            width: width.get() as u32,
            height: height.get() as u32,
            depth_or_array_layers: 1,
        };

        let texture = self.device.create_texture(&TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        StagingSurface {
            texture,
            view,
            format,
            width: size.width,
            height: size.height,
        }
    }
}

impl StagingSurface {
    #[inline]
    pub fn resize(&mut self, context: &super::Context, width: NonZeroU16, height: NonZeroU16) {
        let size = Extent3d {
            width: width.get() as u32,
            height: height.get() as u32,
            depth_or_array_layers: 1,
        };

        let texture = context.device.create_texture(&TextureDescriptor {
            label: None,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: self.format,
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });

        let view = texture.create_view(&TextureViewDescriptor::default());

        self.texture = texture;
        self.view = view;

        self.width = size.width;
        self.height = size.height;
    }
}
