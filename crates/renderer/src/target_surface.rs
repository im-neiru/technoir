use core::num::{NonZeroU8, NonZeroU16};

use wgpu::{Surface, SurfaceConfiguration, rwh::RawWindowHandle};

use crate::{UiRenderer, errors::GraphicsInitError, gpu_context::GpuContext};

pub struct TargetSurface {
    ctx: GpuContext,
    pub(super) surface: Surface<'static>,
    pub(super) config: SurfaceConfiguration,
}

impl TargetSurface {
    #[inline]
    pub async fn new(
        display_handle: Box<dyn wgpu::wgt::WgpuHasDisplayHandle>,
        window_handle: RawWindowHandle,
        width: NonZeroU16,
        height: NonZeroU16,
        desired_maximum_frame_latency: NonZeroU8,
    ) -> Result<Self, GraphicsInitError> {
        let (ctx, surface, config) = GpuContext::get_or_init(
            display_handle,
            window_handle,
            width,
            height,
            desired_maximum_frame_latency,
        )
        .await?;

        Ok(Self {
            ctx,
            surface,
            config,
        })
    }

    #[inline(always)]
    pub const fn width(&self) -> u32 {
        self.config.width
    }

    #[inline(always)]
    pub const fn height(&self) -> u32 {
        self.config.height
    }

    #[inline(always)]
    pub fn renderer(&self) -> UiRenderer {
        UiRenderer { ctx: &self.ctx }
    }
}
