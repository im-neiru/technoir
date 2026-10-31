use wgpu::{
    Adapter, BackendOptions, Backends, Device, DeviceDescriptor, ExperimentalFeatures, Instance,
    InstanceDescriptor, InstanceFlags, MemoryBudgetThresholds, MemoryHints, PowerPreference, Queue,
    RequestAdapterOptions, Surface, SurfaceTargetUnsafe, Trace, wgt::WgpuHasDisplayHandle,
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

        let (adapter, device, queue) = utils::block_on(request_device(&instance, &surface));

        (
            Self {
                instance,
                adapter,
                device,
                queue,
            },
            surface,
        )
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
    unsafe { instance.create_surface_unsafe(surface_handle).unwrap() }
}

async fn request_device(
    instance: &Instance,
    surface: &Surface<'static>,
) -> (Adapter, Device, Queue) {
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(surface),
            apply_limit_buckets: false,
        })
        .await
        .unwrap();

    let (device, queue) = adapter
        .request_device(&DeviceDescriptor {
            label: Some("device"),
            required_features: adapter.features(),
            required_limits: adapter.limits(),
            experimental_features: ExperimentalFeatures::disabled(),
            memory_hints: MemoryHints::Performance,
            trace: Trace::Off,
        })
        .await
        .unwrap();

    (adapter, device, queue)
}
