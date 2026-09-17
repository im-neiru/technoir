use core::num::NonZeroU16;
use wgpu::{
    CompositeAlphaMode, PresentMode, Surface, SurfaceColorSpace, SurfaceConfiguration,
    SurfaceTargetUnsafe, TextureFormat, TextureUsages,
};

pub(crate) struct WindowSurface {
    pub(crate) surface: Surface<'static>,
    pub(crate) config: SurfaceConfiguration,
}

impl super::Context {
    #[inline]
    pub(crate) fn create_surface(
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
            .find(|&f| f == TextureFormat::Rgba8Unorm || f == TextureFormat::Bgra8Unorm)
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

        WindowSurface { surface, config }
    }
}

impl WindowSurface {
    #[inline]
    pub(crate) fn resize(
        &mut self,
        context: super::Context,
        width: NonZeroU16,
        height: NonZeroU16,
    ) {
        self.config.width = width.get() as u32;
        self.config.height = height.get() as u32;

        self.surface.configure(&context.device, &self.config);
    }
}
