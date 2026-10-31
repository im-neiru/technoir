use core::num::{NonZeroU8, NonZeroU16};

use wgpu::{
    Adapter, BackendOptions, Backends, CompositeAlphaMode, Device, DeviceDescriptor,
    ExperimentalFeatures, Instance, InstanceDescriptor, InstanceFlags, MemoryBudgetThresholds,
    MemoryHints, PowerPreference, PresentMode, Queue, RequestAdapterOptions, Surface,
    SurfaceColorSpace, SurfaceConfiguration, SurfaceTargetUnsafe, TextureFormat, TextureUsages,
    Trace, wgt::WgpuHasDisplayHandle,
};

use crate::{TargetSurface, errors::GraphicsInitError};

use super::GpuState;

impl GpuState {
    #[inline(always)]
    pub(super) async fn new(
        display_handle: Box<dyn WgpuHasDisplayHandle>,
        surface_handle: SurfaceTargetUnsafe,
        width: NonZeroU16,
        height: NonZeroU16,
        desired_maximum_frame_latency: NonZeroU8,
    ) -> Result<(Self, Surface<'static>, SurfaceConfiguration), GraphicsInitError> {
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

        let config = configure_surface(
            &adapter,
            &device,
            &surface,
            width,
            height,
            desired_maximum_frame_latency,
        )?;

        Ok((
            Self {
                instance,
                adapter,
                device,
                queue,
            },
            surface,
            config,
        ))
    }

    #[inline]
    pub(super) fn create_surface(
        &self,
        surface_handle: SurfaceTargetUnsafe,
        width: NonZeroU16,
        height: NonZeroU16,
        desired_maximum_frame_latency: NonZeroU8,
    ) -> Result<(Surface<'static>, SurfaceConfiguration), GraphicsInitError> {
        let surface = unsafe {
            self.instance
                .create_surface_unsafe(surface_handle)
                .map_err(|_| GraphicsInitError::CreateSurface)?
        };

        let config = configure_surface(
            &self.adapter,
            &self.device,
            &surface,
            width,
            height,
            desired_maximum_frame_latency,
        )?;

        Ok((surface, config))
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

fn configure_surface(
    adapter: &Adapter,
    device: &Device,
    surface: &Surface<'static>,
    width: NonZeroU16,
    height: NonZeroU16,
    desired_maximum_frame_latency: NonZeroU8,
) -> Result<SurfaceConfiguration, GraphicsInitError> {
    let caps = surface.get_capabilities(&adapter);

    let usage = caps.usages & (TextureUsages::RENDER_ATTACHMENT | TextureUsages::STORAGE_BINDING);

    if usage.is_empty() {
        return Err(GraphicsInitError::IncompatibleSurface);
    }

    let mut format = None;

    for &fmt in caps.formats.iter() {
        if fmt == TextureFormat::Rgba8UnormSrgb {
            format = Some(fmt);

            break;
        }

        if fmt == TextureFormat::Bgra8UnormSrgb {
            format = Some(fmt);
        }

        if format.is_none() && matches!(fmt, TextureFormat::Bgra8Unorm | TextureFormat::Rgba8Unorm)
        {
            format = Some(fmt)
        }
    }

    let format = format.unwrap_or(
        *caps
            .formats
            .first()
            .ok_or(GraphicsInitError::IncompatibleSurface)?,
    );

    let mut present_mode = None;

    for &p in caps.present_modes.iter() {
        if p == PresentMode::Mailbox {
            present_mode = Some(PresentMode::Mailbox);
            break;
        }

        if p == PresentMode::Fifo {
            present_mode = Some(PresentMode::Fifo);
        }
    }

    let present_mode = present_mode.unwrap_or(
        *caps
            .present_modes
            .first()
            .ok_or(GraphicsInitError::IncompatibleSurface)?,
    );

    if !caps.alpha_modes.contains(&CompositeAlphaMode::Opaque) {
        return Err(GraphicsInitError::IncompatibleSurface);
    }

    let config = SurfaceConfiguration {
        usage,
        format,
        color_space: SurfaceColorSpace::Auto,
        width: width.get() as u32,
        height: height.get() as u32,
        present_mode,
        desired_maximum_frame_latency: desired_maximum_frame_latency.get() as u32,
        alpha_mode: CompositeAlphaMode::Opaque,
        view_formats: vec![],
    };

    surface.configure(device, &config);

    Ok(config)
}
