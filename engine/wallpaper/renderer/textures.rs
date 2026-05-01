use wgpu::{
    Extent3d, TexelCopyBufferLayout, Texture, TextureDimension, TextureFormat, TextureUsages,
};

#[cfg(target_os = "windows")]
use windows_sys::Win32::UI::WindowsAndMessaging::{SPI_GETDESKWALLPAPER, SystemParametersInfoW};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureIndex(usize);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum TextureKey {
    SystemWallpaper,
    GrainNoise,
}

impl<'r> super::PrepareContext<'r> {
    pub fn get_texture(&self, index: TextureIndex) -> Option<&wgpu::Texture> {
        self.inner.textures.get_by_index(index.0)
    }

    #[cfg(target_os = "windows")]
    pub fn load_system_wallpaper_as_texture(&mut self) -> (TextureIndex, &Texture) {
        let device = &self.inner.device;
        let queue = &self.inner.queue;

        self.inner
            .textures
            .get_or_insert(TextureKey::SystemWallpaper, || {
                const CAPACITY: usize = 300;
                let mut path = [0u16; CAPACITY];

                unsafe {
                    SystemParametersInfoW(
                        SPI_GETDESKWALLPAPER,
                        CAPACITY as u32,
                        path.as_mut_ptr() as _,
                        0,
                    );
                }

                let len = path.iter().position(|&c| c == 0).unwrap_or(CAPACITY);
                let path_str = String::from_utf16_lossy(&path[..len]);

                let image = image::open(path_str).expect("Failed to open system wallpaper");

                let image = image.to_rgba8();

                let texture = create_texture_from_raw_bytes(
                    device,
                    queue,
                    "system_wallpaper",
                    image.as_raw(),
                    TextureFormat::Rgba8UnormSrgb,
                    TextureUsages::TEXTURE_BINDING,
                    TextureSize {
                        width: image.width(),
                        height: image.height(),
                    },
                    image.width() * 4,
                );

                Ok::<_, std::convert::Infallible>(texture)
            })
            .expect("Texture creation is infallible")
    }

    pub fn create_grain_noise_texture(&mut self) -> (TextureIndex, &Texture) {
        let device = &self.inner.device;
        let queue = &self.inner.queue;

        self.inner
            .textures
            .get_or_insert(TextureKey::GrainNoise, || {
                let width = 1024;
                let height = 1024;

                let mut bytes = vec![0u8; width as usize * height as usize];

                for y in 0..height {
                    for x in 0..width {
                        let index = (y * width + x) as usize;

                        bytes[index] = rand::random();
                    }
                }

                let texture = create_texture_from_raw_bytes(
                    device,
                    queue,
                    "grain_noise",
                    &bytes,
                    TextureFormat::R8Unorm,
                    TextureUsages::TEXTURE_BINDING,
                    TextureSize { width, height },
                    width,
                );

                Ok::<_, std::convert::Infallible>(texture)
            })
            .expect("Texture creation is infallible")
    }
}

impl<'r> super::CleanupContext<'r> {
    pub fn release_texture(&mut self, index: TextureIndex) {
        self.inner.textures.release_by_index(index.0);
    }
}

#[allow(clippy::too_many_arguments)]
fn create_texture_from_raw_bytes(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    name: &str,
    bytes: &[u8],
    format: TextureFormat,
    usage: TextureUsages,
    size: TextureSize,
    bytes_per_row: u32,
) -> Texture {
    let extent = Extent3d {
        width: size.width,
        height: size.height,
        depth_or_array_layers: 1,
    };

    let texture = device.create_texture(&wgpu::TextureDescriptor {
        label: Some(name),
        size: extent,
        format,
        usage: usage | TextureUsages::COPY_DST,
        view_formats: &[],
        mip_level_count: 1,
        sample_count: 1,
        dimension: TextureDimension::D2,
    });

    queue.write_texture(
        texture.as_image_copy(),
        bytes,
        TexelCopyBufferLayout {
            offset: 0,
            bytes_per_row: Some(bytes_per_row),
            rows_per_image: Some(size.height),
        },
        extent,
    );

    texture
}

impl From<usize> for TextureIndex {
    #[inline]
    fn from(value: usize) -> Self {
        Self(value)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct TextureSize {
    pub width: u32,
    pub height: u32,
}
