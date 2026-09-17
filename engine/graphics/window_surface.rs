use core::num::NonZeroU16;

use wgpu::{
    CurrentSurfaceTexture, Surface, SurfaceConfiguration, SurfaceTexture, TextureView,
    TextureViewDescriptor,
};

pub(crate) struct WindowSurface {
    pub(crate) surface: Surface<'static>,
    pub(crate) config: SurfaceConfiguration,
    current_frame: Option<SurfaceTexture>,
}

impl WindowSurface {
    #[inline]
    pub(crate) fn resize(
        &mut self,
        context: &super::Context,
        width: NonZeroU16,
        height: NonZeroU16,
    ) {
        self.config.width = width.get() as u32;
        self.config.height = height.get() as u32;
        self.surface.configure(&context.device, &self.config);
    }

    #[inline(always)]
    pub(crate) fn acquire_view(&mut self, context: &super::Context) -> Option<TextureView> {
        let frame = self.try_acquire(context)?;
        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        self.current_frame = Some(frame);

        Some(view)
    }

    #[inline(always)]
    fn try_acquire(&mut self, context: &super::Context) -> Option<SurfaceTexture> {
        match self.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(txt) | CurrentSurfaceTexture::Suboptimal(txt) => {
                Some(txt)
            }

            CurrentSurfaceTexture::Timeout
            | CurrentSurfaceTexture::Outdated
            | CurrentSurfaceTexture::Lost => {
                self.surface.configure(&context.device, &self.config);
                match self.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(txt)
                    | CurrentSurfaceTexture::Suboptimal(txt) => Some(txt),
                    _ => None,
                }
            }

            CurrentSurfaceTexture::Occluded => None,

            CurrentSurfaceTexture::Validation => None,
        }
    }

    #[inline(always)]
    pub(crate) fn present(&mut self, context: &super::Context) {
        if let Some(frame) = self.current_frame.take() {
            context.queue.present(frame);
        }
    }
}
