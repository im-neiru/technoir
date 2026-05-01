mod init;
mod registry;
mod shaders;
mod textures;

use wgpu::{Device, DeviceDescriptor, Instance, Queue, RequestDeviceError};

pub use init::PreferedDeviceKey;
pub use shaders::{ShaderIndex, ShaderSource};
pub use textures::{TextureIndex, TextureSize};

pub struct WallpaperRenderer {
    pub(in crate::wallpaper) device: Device,
    pub(in crate::wallpaper) queue: Queue,
    textures: registry::Registry<textures::TextureKey, TextureIndex, wgpu::Texture>,
    shaders: registry::Registry<shaders::ShaderKey, ShaderIndex, wgpu::ShaderModule>,
}

pub struct PrepareContext<'r> {
    inner: &'r mut WallpaperRenderer,
}

pub struct CleanupContext<'r> {
    inner: &'r mut WallpaperRenderer,
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

        Ok(Self {
            device,
            queue,
            textures: registry::Registry::new(),
            shaders: registry::Registry::new(),
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
