use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

use crate::gpu_context::GpuContext;

pub struct UiRenderer {
    ctx: GpuContext,
}

impl UiRenderer {
    #[inline]
    pub fn new(
        display_handle: Box<dyn wgpu::wgt::WgpuHasDisplayHandle>,
        window_handle: impl HasDisplayHandle + HasWindowHandle,
    ) -> Self {
        let (ctx, _surface) = GpuContext::get_or_init(display_handle, window_handle);

        Self { ctx }
    }
}
