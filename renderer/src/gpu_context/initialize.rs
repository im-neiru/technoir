use wgpu::{
    BackendOptions, Backends, Instance, InstanceDescriptor, InstanceFlags, MemoryBudgetThresholds,
    Surface, SurfaceTargetUnsafe, wgt::WgpuHasDisplayHandle,
};

use super::GpuState;

impl GpuState {
    #[inline(always)]
    pub(super) fn new(
        display_handle: Box<dyn WgpuHasDisplayHandle>,
        surface_handle: SurfaceTargetUnsafe,
    ) -> (Self, Surface<'static>) {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            flags: InstanceFlags::empty(),
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
            backend_options: BackendOptions::default(),
            display: Some(display_handle),
        });

        let surface = create_surface(&instance, surface_handle);

        (Self { instance }, surface)
    }

    #[inline(always)]
    pub(super) fn create_surface(&self, surface_handle: SurfaceTargetUnsafe) -> Surface<'static> {
        create_surface(&self.instance, surface_handle)
    }
}

#[inline]
pub(super) fn create_surface(
    instance: &Instance,
    surface_handle: SurfaceTargetUnsafe,
) -> Surface<'static> {
    let surface = unsafe { instance.create_surface_unsafe(surface_handle).unwrap() };

    surface
}
