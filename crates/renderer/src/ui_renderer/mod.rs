mod ui_target;

use core::num::{NonZeroU8, NonZeroU16};

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
        width: NonZeroU16,
        height: NonZeroU16,
        desired_maximum_frame_latency: NonZeroU8,
    ) -> Result<(Self, UiTarget), GraphicsInitError> {
        let (ctx, surface) = GpuContext::get_or_init(
            display_handle,
            window_handle,
            width,
            height,
            desired_maximum_frame_latency,
        )
        .await?;

        Ok((Self { ctx }, UiTarget { surface }))
    }
}
