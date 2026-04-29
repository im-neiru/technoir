pub struct Renderer {
    pub device: wgpu::Device,
    pub queue: wgpu::Queue,
    pub surface: wgpu::Surface<'static>,
    pub config: wgpu::SurfaceConfiguration,
}

impl Renderer {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
    ) -> Self {
        Self::new_inner(
            instance,
            surface,
            width,
            height,
            wgpu::TextureUsages::RENDER_ATTACHMENT,
        )
        .await
    }

    pub(super) async fn new_inner(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
        usage: wgpu::TextureUsages,
    ) -> Self {
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .expect("no adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .expect("no device");

        let caps = surface.get_capabilities(&adapter);

        let format = caps
            .formats
            .iter()
            .copied()
            .find(|f| *f == wgpu::TextureFormat::Rgba8Unorm)
            .unwrap_or(caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage,
            format,
            width,
            height,
            present_mode: caps.present_modes[0],
            alpha_mode: caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &config);

        Self {
            device,
            queue,
            surface,
            config,
        }
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);
    }

    #[inline]
    pub fn begin_frame(&self) -> Option<wgpu::SurfaceTexture> {
        match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => Some(surface_texture),
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => {
                self.surface.configure(&self.device, &self.config);
                Some(surface_texture)
            }
            wgpu::CurrentSurfaceTexture::Timeout
            | wgpu::CurrentSurfaceTexture::Occluded
            | wgpu::CurrentSurfaceTexture::Validation => None,
            wgpu::CurrentSurfaceTexture::Outdated => {
                self.surface.configure(&self.device, &self.config);
                None
            }
            wgpu::CurrentSurfaceTexture::Lost => {
                panic!("Surface lost")
            }
        }
    }

    #[inline]
    pub fn present(frame: wgpu::SurfaceTexture) {
        frame.present();
    }

    #[inline]
    pub fn get_aspect_ratio(&self) -> f32 {
        self.config.width as f32 / self.config.height as f32
    }

    #[inline]
    pub fn create_buffer<T: Sized>(
        &self,
        label: Option<&str>,
        usage: wgpu::BufferUsages,
        mapped_at_creation: bool,
    ) -> wgpu::Buffer {
        let size = core::mem::size_of::<T>() as u64;

        self.device.create_buffer(&wgpu::BufferDescriptor {
            label,
            size,
            usage,
            mapped_at_creation,
        })
    }

    pub fn scale_texture(&self, texture_size: glam::Vec2) -> glam::Vec2 {
        let screen_aspect = self.get_aspect_ratio();
        let tex_aspect = texture_size.x / texture_size.y;

        if screen_aspect > tex_aspect {
            glam::vec2(1.0, tex_aspect / screen_aspect)
        } else {
            glam::vec2(screen_aspect / tex_aspect, 1.0)
        }
    }
}
