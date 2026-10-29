#[cfg(not(all(feature = "web", any(target_arch = "wasm32", target_arch = "wasm64"))))]
use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};
use wgpu::{SurfaceTargetUnsafe, wgt::WgpuHasDisplayHandle};

use crate::gpu_context::GpuContext;

pub struct Renderer {
    ctx: GpuContext,
}

impl Renderer {
    #[inline(always)]
    #[cfg(not(all(feature = "web", any(target_arch = "wasm32", target_arch = "wasm64"))))]
    pub fn new_native(
        display_handle: Box<dyn wgpu::wgt::WgpuHasDisplayHandle>,
        window: impl HasDisplayHandle + HasWindowHandle,
    ) -> Self {
        let raw_display_handle = window.display_handle().unwrap().as_raw();
        let raw_window_handle = window.window_handle().unwrap().as_raw();

        let surface_handle = SurfaceTargetUnsafe::RawHandle {
            raw_display_handle: Some(raw_display_handle),
            raw_window_handle,
        };

        Self::new(display_handle, surface_handle)
    }

    fn new(
        display_handle: Box<dyn WgpuHasDisplayHandle>,
        surface_handle: SurfaceTargetUnsafe,
    ) -> Self {
        let (ctx, _surface) = GpuContext::get_or_init(display_handle, surface_handle);

        Self { ctx }
    }
}
