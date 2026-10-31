mod ui_target;

use wgpu::rwh::{HasDisplayHandle, HasWindowHandle};

use crate::{errors::GraphicsInitError, gpu_context::GpuContext};

pub use ui_target::UiTarget;

pub struct UiRenderer {
    ctx: GpuContext,
}

impl UiRenderer {
    #[inline]
    pub async fn new(
        display_handle: Box<dyn wgpu::wgt::WgpuHasDisplayHandle>,
        window_handle: impl HasDisplayHandle + HasWindowHandle,
    ) -> Result<(Self, UiTarget), GraphicsInitError> {
        let (ctx, surface) = GpuContext::get_or_init(display_handle, window_handle).await?;

        Ok((Self { ctx }, UiTarget { surface }))
    }
}
