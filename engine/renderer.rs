use vello::{Scene, wgpu};

pub struct Renderer {
    wgpu_surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    renderer: vello::Renderer,

    target_texture: wgpu::Texture,
    target_view: wgpu::TextureView,
    config: wgpu::SurfaceConfiguration,
    target_format: wgpu::TextureFormat,
}

impl Renderer {
    pub async fn new(
        wgpu_instance: &wgpu::Instance,
        wgpu_surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
    ) -> Self {
        let adapter = wgpu_instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::default(),
                compatible_surface: Some(&wgpu_surface),
                force_fallback_adapter: false,
            })
            .await
            .expect("No compatible adapter");

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                experimental_features: wgpu::ExperimentalFeatures::disabled(),
                required_limits: wgpu::Limits::default(),
                memory_hints: Default::default(),
                trace: wgpu::Trace::Off,
            })
            .await
            .expect("Failed to request device");

        let surface_caps = wgpu_surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| *f == wgpu::TextureFormat::Rgba8Unorm)
            .unwrap_or(surface_caps.formats[0]);

        let renderer = vello::Renderer::new(
            &device,
            vello::RendererOptions {
                use_cpu: false,
                antialiasing_support: vello::AaSupport::all(),
                num_init_threads: core::num::NonZeroUsize::new(1),
                pipeline_cache: None,
            },
        )
        .expect("Failed to create renderer");

        let (target_texture, target_view) =
            Self::create_target(&device, width, height, surface_format);

        let config = vello::wgpu::SurfaceConfiguration {
            usage: vello::wgpu::TextureUsages::RENDER_ATTACHMENT
                | vello::wgpu::TextureUsages::COPY_DST,
            format: surface_format,
            width,
            height,
            present_mode: surface_caps.present_modes[0],
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        wgpu_surface.configure(&device, &config);

        Self {
            wgpu_surface,
            device,
            queue,
            renderer,
            target_texture,
            target_view,
            config,
            target_format: surface_format,
        }
    }

    fn create_target(
        device: &wgpu::Device,
        width: u32,
        height: u32,
        format: wgpu::TextureFormat,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Vello Storage Target"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format,
            usage: wgpu::TextureUsages::STORAGE_BINDING | wgpu::TextureUsages::COPY_SRC,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    pub fn render_scene(&mut self, scene: &Scene, base_color: vello::peniko::Color) {
        let frame = self
            .wgpu_surface
            .get_current_texture()
            .expect("Failed to get frame");

        self.renderer
            .render_to_texture(
                &self.device,
                &self.queue,
                scene,
                &self.target_view,
                &vello::RenderParams {
                    base_color,
                    width: self.config.width,
                    height: self.config.height,
                    antialiasing_method: vello::AaConfig::Area,
                },
            )
            .unwrap();

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        encoder.copy_texture_to_texture(
            self.target_texture.as_image_copy(),
            frame.texture.as_image_copy(),
            wgpu::Extent3d {
                width: self.config.width,
                height: self.config.height,
                depth_or_array_layers: 1,
            },
        );

        self.queue.submit(Some(encoder.finish()));
        frame.present();
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;

        self.wgpu_surface.configure(&self.device, &self.config);

        // recreate target
        self.target_texture.destroy();
        let (texture, view) = Self::create_target(&self.device, width, height, self.target_format);
        self.target_texture = texture;
        self.target_view = view;
    }

    #[inline]
    pub fn width(&self) -> u32 {
        self.config.width
    }

    #[inline]
    pub fn height(&self) -> u32 {
        self.config.height
    }
}
