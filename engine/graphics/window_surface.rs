use core::num::NonZeroU16;
use std::sync::Arc;

use smol::lock::RwLock;
use wgpu::{
    CompositeAlphaMode, CurrentSurfaceTexture, PresentMode, Surface, SurfaceColorSpace,
    SurfaceConfiguration, SurfaceTargetUnsafe, SurfaceTexture, TextureFormat, TextureUsages,
};

#[derive(Clone)]
pub(crate) struct WindowSurface(Arc<Inner>);

struct Inner {
    surface: Surface<'static>,
    config: RwLock<SurfaceConfiguration>,
}

impl super::Context {
    #[inline]
    pub(crate) fn create_window_surface(
        &self,
        target: impl Into<SurfaceTargetUnsafe>,
        width: NonZeroU16,
        height: NonZeroU16,
    ) -> WindowSurface {
        let surface = unsafe {
            self.instance
                .create_surface_unsafe(target.into())
                .expect("Failed to create wgpu::Surface")
        };

        let caps = surface.get_capabilities(&self.adapter);

        let format = caps
            .formats
            .iter()
            .copied()
            .find(|&format| {
                matches!(
                    format,
                    TextureFormat::Rgba8Unorm | TextureFormat::Bgra8Unorm
                )
            })
            .unwrap_or(caps.formats[0]);

        let present_mode = if caps.present_modes.contains(&PresentMode::Mailbox) {
            PresentMode::Mailbox
        } else if caps.present_modes.contains(&PresentMode::Immediate) {
            PresentMode::Immediate
        } else {
            PresentMode::Fifo
        };

        let alpha_mode = if caps.alpha_modes.contains(&CompositeAlphaMode::Opaque) {
            CompositeAlphaMode::Opaque
        } else {
            caps.alpha_modes[0]
        };

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.get() as u32,
            height: height.get() as u32,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: SurfaceColorSpace::Auto,
        };

        surface.configure(&self.device, &config);

        WindowSurface::new(surface, config)
    }
}

impl WindowSurface {
    #[inline(always)]
    pub(super) fn new(surface: Surface<'static>, config: SurfaceConfiguration) -> Self {
        Self(Arc::new(Inner {
            surface,
            config: RwLock::new(config),
        }))
    }

    #[inline]
    pub(crate) fn resize(&self, context: &super::Context, width: NonZeroU16, height: NonZeroU16) {
        smol::block_on(async {
            let mut config = self.0.config.write().await;

            config.width = width.get() as u32;
            config.height = height.get() as u32;

            self.0.surface.configure(&context.device, &config);
        });
    }

    #[inline(always)]
    pub(crate) fn width(&self) -> u32 {
        smol::block_on(async { self.0.config.read().await.width })
    }

    #[inline(always)]
    pub(crate) fn height(&self) -> u32 {
        smol::block_on(async { self.0.config.read().await.height })
    }

    #[inline]
    pub fn try_acquire(&self, context: &super::Context) -> Option<SurfaceTexture> {
        match self.0.surface.get_current_texture() {
            CurrentSurfaceTexture::Success(texture)
            | CurrentSurfaceTexture::Suboptimal(texture) => Some(texture),

            CurrentSurfaceTexture::Timeout
            | CurrentSurfaceTexture::Outdated
            | CurrentSurfaceTexture::Lost => smol::block_on(async {
                let config = self.0.config.read().await;

                self.0.surface.configure(&context.device, &config);

                match self.0.surface.get_current_texture() {
                    CurrentSurfaceTexture::Success(texture)
                    | CurrentSurfaceTexture::Suboptimal(texture) => Some(texture),

                    CurrentSurfaceTexture::Timeout
                    | CurrentSurfaceTexture::Outdated
                    | CurrentSurfaceTexture::Lost
                    | CurrentSurfaceTexture::Occluded
                    | CurrentSurfaceTexture::Validation => None,
                }
            }),

            CurrentSurfaceTexture::Occluded | CurrentSurfaceTexture::Validation => None,
        }
    }
}
