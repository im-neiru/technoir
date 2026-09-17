use core::num::NonZeroU16;

use raw_window_handle as rwh;
use wgpu::{
    Adapter, BackendOptions, Backends, CompositeAlphaMode, Device, DeviceDescriptor, Instance,
    InstanceDescriptor, InstanceFlags, MemoryBudgetThresholds, PresentMode, Queue,
    RequestAdapterOptions, SurfaceColorSpace, SurfaceConfiguration, SurfaceTargetUnsafe,
    TextureFormat, TextureUsages,
};

use super::WindowSurface;

#[derive(Clone)]
pub struct Context {
    pub(crate) instance: Instance,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
    pub(crate) adapter: Adapter,
}

impl Context {
    pub async fn new_with_primary(
        primary_target: impl Into<SurfaceTargetUnsafe>,
        width: NonZeroU16,
        height: NonZeroU16,
    ) -> (Self, WindowSurface) {
        let instance = Instance::new(InstanceDescriptor {
            backends: Backends::PRIMARY,
            flags: InstanceFlags::default(),
            memory_budget_thresholds: MemoryBudgetThresholds::default(),
            backend_options: BackendOptions::default(),
            display: Some(Box::new(DisplayHandle)),
        });

        let primary_surface = unsafe {
            instance
                .create_surface_unsafe(primary_target.into())
                .expect("Failed to create primary wgpu::Surface")
        };

        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                compatible_surface: Some(&primary_surface),
                ..Default::default()
            })
            .await
            .expect("No compatible GPU adapter found");

        let required_features = adapter.features();
        let required_limits = adapter.limits();

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("Graphics Device"),
                required_features,
                required_limits,
                ..Default::default()
            })
            .await
            .expect("Failed to request high-performance GPU device configuration");

        let caps = primary_surface.get_capabilities(&adapter);

        let format = caps
            .formats
            .iter()
            .copied()
            .find(|&f| f == TextureFormat::Rgba8Unorm || f == TextureFormat::Bgra8Unorm)
            .expect("Primary surface or Adapter does not support compatible Rgba8Unorm or Bgra8Unorm formats!");

        let present_mode = if caps.present_modes.contains(&PresentMode::Mailbox) {
            PresentMode::Mailbox
        } else if caps.present_modes.contains(&PresentMode::Immediate) {
            PresentMode::Immediate
        } else {
            PresentMode::Fifo
        };

        let alpha_mode = if caps.alpha_modes.contains(&CompositeAlphaMode::Opaque) {
            CompositeAlphaMode::Opaque
        } else {
            caps.alpha_modes[0]
        };

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT,
            format,
            width: width.get() as u32,
            height: height.get() as u32,
            present_mode,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
            color_space: SurfaceColorSpace::Auto,
        };

        primary_surface.configure(&device, &config);

        let context = Self {
            instance,
            device,
            queue,
            adapter,
        };

        let window_surface = WindowSurface {
            surface: primary_surface,
            config,
        };

        (context, window_surface)
    }
}

#[derive(Debug)]
struct DisplayHandle;

impl rwh::HasDisplayHandle for DisplayHandle {
    fn display_handle(&self) -> Result<rwh::DisplayHandle<'_>, rwh::HandleError> {
        Ok(rwh::DisplayHandle::windows())
    }
}
