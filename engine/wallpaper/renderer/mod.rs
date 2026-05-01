mod init;

use wgpu::{Device, DeviceDescriptor, Instance, Queue, RequestDeviceError, Surface};

pub use init::PreferedDeviceKey;

pub struct WallpaperRenderer {
    instance: Instance,
    surfaces: Vec<Surface<'static>>,

    device: Device,
    queue: Queue,
}

impl WallpaperRenderer {
    pub(crate) async fn new(
        instance: &Instance,
        preferred: Option<PreferedDeviceKey>,
        surfaces: Vec<Surface<'static>>,
    ) -> Result<Self, WallpaperRendererCreateError> {
        let instance = instance.clone();

        let Some(adapter) = Self::select_adapter(&instance, preferred, &surfaces).await else {
            return Err(WallpaperRendererCreateError::NoAdapterFound);
        };

        let (device, queue) = adapter
            .request_device(&DeviceDescriptor {
                label: Some("wallpaper_renderer_device"),
                required_features: Default::default(),
                required_limits: wgpu::Limits::default(),
                experimental_features: Default::default(),
                memory_hints: wgpu::MemoryHints::Performance,
                trace: wgpu::Trace::Off,
            })
            .await?;

        Ok(Self {
            instance,
            surfaces,
            device,
            queue,
        })
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum WallpaperRendererCreateError {
    #[error("No adapter found")]
    NoAdapterFound,
    #[error("Failed to request device")]
    RequestDevice(#[from] RequestDeviceError),
}
