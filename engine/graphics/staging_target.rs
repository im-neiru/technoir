use core::num::NonZeroU16;

use wgpu::{
    Extent3d, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
    TextureView, TextureViewDescriptor,
};

pub struct TargetSurface {
    texture: Texture,
    view: TextureView,
    format: TextureFormat,
}

impl super::Context {
    #[inline]
    pub fn create_target_surface(
        &self,
        width: NonZeroU16,
        height: NonZeroU16,
        format: TextureFormat,
    ) -> TargetSurface {
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

        TargetSurface {
            texture,
            view,
            format,
        }
    }
}

impl TargetSurface {
    #[inline]
    pub(crate) fn acquire_view(&self) -> &TextureView {
        &self.view
    }

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
    }
}
