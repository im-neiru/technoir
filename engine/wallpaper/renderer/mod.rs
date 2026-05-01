mod init;

use wgpu::{Device, DeviceDescriptor, Instance, Queue, RequestDeviceError};

pub use init::PreferedDeviceKey;

pub struct WallpaperRenderer {
    pub(in crate::wallpaper) device: Device,
    pub(in crate::wallpaper) queue: Queue,
}

impl WallpaperRenderer {
    pub(crate) async fn new(
        instance: &Instance,
        preferred: Option<PreferedDeviceKey>,
        screens: &mut [super::Screen],
    ) -> Result<Self, WallpaperRendererCreateError> {
        let instance = instance.clone();

        let Some(adapter) = Self::select_adapter(&instance, preferred, screens).await else {
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

        for s in screens {
            if let Some(target) = s.target.as_mut() {
                target.configure(&device, &adapter);
            }
        }

        Ok(Self { device, queue })
    }
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum WallpaperRendererCreateError {
    #[error("No adapter found")]
    NoAdapterFound,
    #[error("Failed to request device")]
    RequestDevice(#[from] RequestDeviceError),
}
