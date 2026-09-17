use wgpu::{Instance, Surface, SurfaceConfiguration, SurfaceTargetUnsafe};

pub(crate) struct WindowSurface {
    pub(crate) surface: Surface<'static>,
    pub(crate) config: Option<SurfaceConfiguration>,
}

impl WindowSurface {
    #[inline]
    pub(crate) fn new(wgpu_instance: Instance, target: impl Into<SurfaceTargetUnsafe>) -> Self {
        let surface = unsafe {
            wgpu_instance
                .create_surface_unsafe(target.into())
                .expect("Failed to create wgpu::Surface")
        };

        Self {
            config: surface.get_configuration(),
            surface,
        }
    }
}
