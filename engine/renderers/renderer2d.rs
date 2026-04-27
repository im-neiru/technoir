use std::num::NonZero;

use vello::{Renderer as VelloRenderer, RendererOptions, wgpu};

use super::Renderer;

pub struct Renderer2d {
    pub(super) renderer: Renderer,
    vello: VelloRenderer,
    target_texture: wgpu::Texture,
    pub(super) target_view: wgpu::TextureView,
    format: wgpu::TextureFormat,
}

impl Renderer2d {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
    ) -> Self {
        let renderer = Renderer::new_inner(
            instance,
            surface,
            width,
            height,
            wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::COPY_DST,
        )
        .await;

        let vello = VelloRenderer::new(
            &renderer.device,
            RendererOptions {
                use_cpu: false,
                antialiasing_support: vello::AaSupport::all(),
                num_init_threads: NonZero::new(1),
                pipeline_cache: None,
            },
        )
        .expect("vello init failed");

        let format = renderer.config.format;

        let (target_texture, target_view) =
            Self::create_target(&renderer.device, width, height, format);

        Self {
            renderer,
            vello,
            target_texture,
            target_view,
            format,
        }
    }

    pub fn render_scene(&mut self, scene: &vello::Scene, base_color: vello::peniko::Color) {
        let frame = self.renderer.begin_frame();

        self.vello
            .render_to_texture(
                &self.renderer.device,
                &self.renderer.queue,
                scene,
                &self.target_view,
                &vello::RenderParams {
                    base_color,
                    width: self.renderer.config.width,
                    height: self.renderer.config.height,
                    antialiasing_method: vello::AaConfig::Area,
                },
            )
            .unwrap();

        let mut encoder = self
            .renderer
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        encoder.copy_texture_to_texture(
            self.target_texture.as_image_copy(),
            frame.texture.as_image_copy(),
            wgpu::Extent3d {
                width: self.renderer.config.width,
                height: self.renderer.config.height,
                depth_or_array_layers: 1,
            },
        );

        self.renderer.queue.submit(Some(encoder.finish()));
        Renderer::present(frame);
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);
        let (target_texture, target_view) =
            Self::create_target(&self.renderer.device, width, height, self.format);

        self.target_texture = target_texture;
        self.target_view = target_view;
    }
}

impl Renderer2d {
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
            usage: wgpu::TextureUsages::STORAGE_BINDING
                | wgpu::TextureUsages::COPY_SRC
                | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }
}
