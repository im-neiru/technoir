use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

use crate::{errors::GraphicsInitError, gpu_context::GpuContext};

pub struct UiRenderer {
    ctx: GpuContext,
}

impl UiRenderer {
    #[inline]
    pub async fn new(
        display_handle: Box<dyn wgpu::wgt::WgpuHasDisplayHandle>,
        window_handle: impl HasDisplayHandle + HasWindowHandle,
    ) -> Result<Self, GraphicsInitError> {
        let (ctx, _surface) = GpuContext::get_or_init(display_handle, window_handle).await?;

        Ok(Self { ctx })
    }
}
