use wgpu::{
    Adapter, BackendOptions, Backends, Device, DeviceDescriptor, ExperimentalFeatures, Instance,
    InstanceDescriptor, InstanceFlags, MemoryBudgetThresholds, MemoryHints, PowerPreference, Queue,
    RequestAdapterOptions, Surface, SurfaceTargetUnsafe, Trace, wgt::WgpuHasDisplayHandle,
};

use crate::errors::GraphicsInitError;

use super::GpuState;

impl GpuState {
    #[inline(always)]
    pub(super) async fn new(
        display_handle: Box<dyn WgpuHasDisplayHandle>,
        surface_handle: SurfaceTargetUnsafe,
    ) -> Result<(Self, Surface<'static>), GraphicsInitError> {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            flags: InstanceFlags::empty(),
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
            backend_options: BackendOptions::default(),
            display: Some(display_handle),
        });

        let surface = unsafe {
            instance
                .create_surface_unsafe(surface_handle)
                .map_err(|_| GraphicsInitError::CreateSurface)?
        };

        let (adapter, device, queue) = request_device(&instance, &surface).await?;

        Ok((
            Self {
                instance,
                adapter,
                device,
                queue,
            },
            surface,
        ))
    }

    #[inline]
    pub(super) fn create_surface(
        &self,
        surface_handle: SurfaceTargetUnsafe,
    ) -> Result<Surface<'static>, GraphicsInitError> {
        unsafe {
            self.instance
                .create_surface_unsafe(surface_handle)
                .map_err(|_| GraphicsInitError::CreateSurface)
        }
    }
}

async fn request_device(
    instance: &Instance,
    surface: &Surface<'static>,
) -> Result<(Adapter, Device, Queue), GraphicsInitError> {
    let adapter = instance
        .request_adapter(&RequestAdapterOptions {
            power_preference: PowerPreference::HighPerformance,
            force_fallback_adapter: false,
            compatible_surface: Some(surface),
            apply_limit_buckets: false,
        })
        .await
        .map_err(|_| GraphicsInitError::NoCompatibleDevice)?;

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
        .map_err(|_| GraphicsInitError::CreateDevice)?;

    Ok((adapter, device, queue))
}
