use wgpu::{
    BackendOptions, Backends, Instance, InstanceDescriptor, InstanceFlags, MemoryBudgetThresholds,
    rwh, wgt,
};

use super::{GpuContext, GpuState};

impl GpuState {
    #[inline(always)]
    pub(super) fn new(
        display: Box<dyn wgt::WgpuHasDisplayHandle>,
        _window: rwh::WindowHandle,
    ) -> Self {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            flags: InstanceFlags::empty(),
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
            backend_options: BackendOptions::default(),
            display: Some(display),
        });

        Self { instance }
    }
}
