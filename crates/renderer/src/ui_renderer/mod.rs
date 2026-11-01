mod ui_target;

use crate::gpu_context::GpuContext;

pub use ui_target::UiTarget;

pub struct UiRenderer<'a> {
    pub(super) ctx: &'a GpuContext,
}
