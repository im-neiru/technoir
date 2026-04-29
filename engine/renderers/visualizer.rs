use bytemuck::{Pod, Zeroable};
use chrono::Duration;
use glam::*;

use glyphon::{
    Attrs, Color, FontSystem, Metrics, Resolution, SwashCache, TextArea, TextAtlas, TextBounds,
    TextRenderer, Viewport,
};

use super::Renderer;
use crate::samplers::SpectrumAudioLoopback;

pub struct Visualizer {
    renderer: Renderer,

    prev_spectrum_l: [f32; SAMPLE_COUNT],
    prev_spectrum_r: [f32; SAMPLE_COUNT],

    prev_lum: f32,
    pipeline: wgpu::RenderPipeline,
    bind_group: wgpu::BindGroup,
    noise_texture: wgpu::Texture,
    noise_sampler: wgpu::Sampler,

    texture_size: Vec2,
    u_ephemerals: wgpu::Buffer,
    u_scaling: wgpu::Buffer,

    text_renderer: TextRenderer,
    atlas: TextAtlas,
    tex_viewport: Viewport,
    text_mask: wgpu::Texture,
    text_mask_view: wgpu::TextureView,
    text_buf: glyphon::Buffer,
    now: chrono::DateTime<chrono::Local>,
    font_system: glyphon::FontSystem,
    swash_cache: glyphon::SwashCache,
}

const SAMPLE_COUNT: usize = 64;
const SWIRL_STRENGTH: f32 = 5.5;
const SWIRL_SPEED: f32 = 1.0;

const BG_SHADER: &str = include_str!("./shaders/visualizer.wgsl");

impl Visualizer {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
    ) -> Self {
        let renderer = Renderer::new(instance, surface, width, height).await;

        let device = &renderer.device;

        let bg_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("BG Sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let u_ephemerals = renderer.create_buffer::<Ephemerals>(
            Some("u_ephemerals"),
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            false,
        );
        let u_scaling = renderer.create_buffer::<Ephemerals>(
            Some("u_scaling"),
            wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            false,
        );

        let text_mask = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("text_mask"),
            size: wgpu::Extent3d {
                width: 1024,
                height: 1024,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::R8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });

        let text_mask_view = text_mask.create_view(&wgpu::TextureViewDescriptor::default());
        let text_mask_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("text_mask_sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let bg_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                label: Some("BG Bind Group Layout"),
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },

                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 1,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 2,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },

                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 3,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 4,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 5,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 6,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                    wgpu::BindGroupLayoutEntry {
                        binding: 7,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Buffer {
                            ty: wgpu::BufferBindingType::Uniform,
                            has_dynamic_offset: false,
                            min_binding_size: None,
                        },
                        count: None,
                    },
                ],
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("BG Shader"),
            source: wgpu::ShaderSource::Wgsl(BG_SHADER.into()),
        });

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("BG Pipeline Layout"),
            bind_group_layouts: &[Some(&bg_bind_group_layout)],
            immediate_size: 0,
        });

        let surface_format = renderer.config.format;

        let bg_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("BG Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surface_format,
                    blend: Some(wgpu::BlendState::ALPHA_BLENDING),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState::default(),
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let (bg_texture, texture_size) =
            Self::load_bg_texture("./sample/image.jpg", device, &renderer.queue)
                .expect("Failed to load background");

        let noise_texture = Self::load_noise_texture(device, &renderer.queue).unwrap();

        let noise_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Noise Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group = {
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("BG Bind Group"),
                layout: &bg_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(
                            &bg_texture.create_view(&wgpu::TextureViewDescriptor::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&bg_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(&text_mask_view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&text_mask_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: wgpu::BindingResource::TextureView(
                            &noise_texture.create_view(&Default::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: wgpu::BindingResource::Sampler(&noise_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: u_ephemerals.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: u_scaling.as_entire_binding(),
                    },
                ],
            })
        };

        let mut font_system = FontSystem::new();

        let mut swash_cache = SwashCache::new();
        let cache = glyphon::Cache::new(device);
        let mut atlas = TextAtlas::new(
            device,
            &renderer.queue,
            &cache,
            wgpu::TextureFormat::R8Unorm,
        );

        let now = chrono::Local::now();

        let mut text_renderer = TextRenderer::new(
            &mut atlas,
            device,
            wgpu::MultisampleState {
                count: 1,
                mask: !0,
                alpha_to_coverage_enabled: false,
            },
            None,
        );

        let mut tex_viewport = glyphon::Viewport::new(device, &cache);

        tex_viewport.update(
            &renderer.queue,
            glyphon::Resolution {
                width: 1024,
                height: 1024,
            },
        );

        let mut text_buf = glyphon::Buffer::new(&mut font_system, glyphon::Metrics::new(42., 45.0));

        let human_time = now.format("%A\n%I:%M:%S %p").to_string();

        text_buf.set_text(
            &mut font_system,
            &human_time,
            &Attrs::new().family(glyphon::Family::Name("Zen Dots")),
            glyphon::Shaping::Advanced,
            Some(glyphon::cosmic_text::Align::Center),
        );

        text_buf.set_size(&mut font_system, Some(1024.0), Some(1024.0));
        text_buf.shape_until_scroll(&mut font_system, false);

        let text_height = text_buf.layout_runs().count() as f32 * text_buf.metrics().line_height;
        let top_offset = (1024.0 - text_height) / 2.0;

        text_renderer
            .prepare(
                device,
                &renderer.queue,
                &mut font_system,
                &mut atlas,
                &tex_viewport,
                [TextArea {
                    buffer: &text_buf,
                    left: 0.0,
                    top: top_offset,
                    scale: 1.0,
                    bounds: TextBounds {
                        left: 0,
                        top: 0,
                        right: 1024,
                        bottom: 1024,
                    },
                    default_color: Color::rgb(255, 255, 255),
                    custom_glyphs: &[],
                }],
                &mut swash_cache,
            )
            .unwrap();

        let mut result = Self {
            renderer,
            prev_spectrum_l: [0.0; SAMPLE_COUNT],
            prev_spectrum_r: [0.0; SAMPLE_COUNT],
            pipeline: bg_pipeline,
            bind_group,
            u_ephemerals,
            u_scaling,
            prev_lum: 0.,
            noise_texture,
            noise_sampler,
            texture_size,
            text_renderer,
            tex_viewport,
            atlas,
            text_mask,
            text_mask_view,
            text_buf,
            now,
            font_system,
            swash_cache,
        };

        result.resize(width, height);
        result
    }

    fn load_bg_texture(
        path: impl AsRef<std::path::Path>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<(wgpu::Texture, Vec2)> {
        let img = image::open(path).ok()?.to_rgba8();
        let (width, height) = img.dimensions();
        let raw = img.into_raw();

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Background Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            &raw,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width),
                rows_per_image: Some(height),
            },
            wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
        );

        Some((texture, vec2(width as f32, height as f32)))
    }

    fn load_noise_texture(device: &wgpu::Device, queue: &wgpu::Queue) -> Option<wgpu::Texture> {
        let width = 2048;
        let height = 2048;
        let mut data = vec![0u8; width * height * 4];

        for y in 0..height {
            for x in 0..width {
                let i = (y * width + x) * 4;
                data[i] = rand::random();
                data[i + 1] = rand::random();
                data[i + 2] = rand::random();
                data[i + 3] = rand::random();
            }
        }

        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Noise Texture"),
            size: wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8Unorm,
            usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
            view_formats: &[],
        });

        queue.write_texture(
            texture.as_image_copy(),
            &data,
            wgpu::TexelCopyBufferLayout {
                offset: 0,
                bytes_per_row: Some(4 * width as u32),
                rows_per_image: Some(height as u32),
            },
            wgpu::Extent3d {
                width: width as u32,
                height: height as u32,
                depth_or_array_layers: 1,
            },
        );

        Some(texture)
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);

        self.prev_spectrum_l = [0.0; SAMPLE_COUNT];
        self.prev_spectrum_r = [0.0; SAMPLE_COUNT];

        let scaling = Scaling {
            bg_scaling: self.renderer.scale_texture(self.texture_size),
            aspect_ratio: self.renderer.get_aspect_ratio(),
            _padding: 0.,
        };

        self.renderer
            .queue
            .write_buffer(&self.u_scaling, 0, bytemuck::cast_slice(&[scaling]));
    }

    pub fn render(&mut self, audio: &mut SpectrumAudioLoopback, time: f32) {
        let Some(frame) = self.renderer.begin_frame() else {
            return;
        };

        let new_now = chrono::Local::now();

        if new_now.signed_duration_since(self.now) > Duration::seconds(1) {
            self.now = new_now;

            // 1. Initialize buffer
            let mut text_buf =
                glyphon::Buffer::new(&mut self.font_system, glyphon::Metrics::new(30.0, 50.0));

            // 2. Prepare the three separate strings
            let day_string = self.now.format("%A").to_string();
            // Use uppercase for the month to match your example (e.g., MARCH)
            let date_string = self.now.format("\n%d %B %Y").to_string().to_uppercase();
            let time_string = self.now.format("\n%I:%M:%S %p").to_string();

            // 3. Create distinct attributes for each line
            let day_attrs = Attrs::new()
                .family(glyphon::Family::Name("Zen Dots"))
                .metrics(glyphon::Metrics::new(40.0, 50.0));

            // Date is slightly smaller (20.0) than the time (30.0)
            let date_attrs = Attrs::new()
                .family(glyphon::Family::Name("Zen Dots"))
                .metrics(glyphon::Metrics::new(20.0, 30.0));

            let time_attrs = Attrs::new()
                .family(glyphon::Family::Name("Zen Dots"))
                .metrics(glyphon::Metrics::new(30.0, 50.0));

            // 4. Set Rich Text with the new middle line
            text_buf.set_rich_text(
                &mut self.font_system,
                [
                    (day_string.as_str(), day_attrs),
                    (date_string.as_str(), date_attrs),
                    (time_string.as_str(), time_attrs),
                ],
                &Attrs::new().family(glyphon::Family::Name("Zen Dots")),
                glyphon::Shaping::Advanced,
                Some(glyphon::cosmic_text::Align::Center),
            );

            // 5. Setup Layout & Alignment
            text_buf.set_size(&mut self.font_system, Some(1024.0), Some(1024.0));

            for line in text_buf.lines.iter_mut() {
                line.set_align(Some(glyphon::cosmic_text::Align::Center));
            }

            text_buf.shape_until_scroll(&mut self.font_system, false);

            // 6. Calculate Vertical Centering
            // This logic remains the same; it will now count 3 lines instead of 2 automatically
            let mut total_height = 0.0;
            for run in text_buf.layout_runs() {
                total_height += run.line_height;
            }

            let top_offset = (1024.0 - total_height) / 2.0;

            // 7. Prepare the Renderer
            self.text_renderer
                .prepare(
                    &self.renderer.device,
                    &self.renderer.queue,
                    &mut self.font_system,
                    &mut self.atlas,
                    &self.tex_viewport,
                    [TextArea {
                        buffer: &text_buf,
                        left: 0.0,
                        top: top_offset,
                        scale: 1.0,
                        bounds: TextBounds {
                            left: 0,
                            top: 0,
                            right: 1024,
                            bottom: 1024,
                        },
                        default_color: Color::rgb(255, 255, 255),
                        custom_glyphs: &[],
                    }],
                    &mut self.swash_cache,
                )
                .unwrap();
        }

        let mut encoder =
            self.renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Frame Encoder"),
                });

        {
            let mut pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Text Mask Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.text_mask_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            self.text_renderer
                .render(&self.atlas, &self.tex_viewport, &mut pass)
                .unwrap();
        }

        let left = audio.left_spectrum();
        let right = audio.right_spectrum();

        let avg_left = if left.is_empty() {
            0.0
        } else {
            left.iter().copied().sum::<f32>() / left.len() as f32
        };

        let avg_right = if right.is_empty() {
            0.0
        } else {
            right.iter().copied().sum::<f32>() / right.len() as f32
        };

        let avg = avg_left.max(avg_right);
        let target_dim = (1.0 - avg * 0.6).max(0.2);

        let alpha = 0.2;

        let lum_value = self.prev_lum + (target_dim - self.prev_lum) * alpha;

        self.prev_lum = lum_value;

        let mut empherals = Ephemerals {
            lum: lum_value,
            swirl_factor: SWIRL_STRENGTH * ((time * SWIRL_SPEED).sin() * 0.3 + 1.0) * 0.8,
            time,
            _padding: 0.,
            spectrum: [[0.0; 4]; 16],
        };

        let count = left.len().min(right.len()).min(16);

        for i in 0..count {
            let l = left[i];
            let r = right[i];
            let mono = (l + r) * 0.80 - 0.05;

            empherals.spectrum[i] = [mono, mono, mono, mono];
        }

        self.renderer
            .queue
            .write_buffer(&self.u_ephemerals, 0, bytemuck::cast_slice(&[empherals]));

        {
            let view = frame
                .texture
                .create_view(&wgpu::TextureViewDescriptor::default());

            let mut rp = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Background Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::WHITE),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            rp.set_pipeline(&self.pipeline);
            rp.set_bind_group(0, &self.bind_group, &[]);
            rp.draw(0..3, 0..1);
        }

        self.renderer.queue.submit(Some(encoder.finish()));

        frame.present();
    }
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Ephemerals {
    lum: f32,
    time: f32,
    swirl_factor: f32,
    _padding: f32,
    spectrum: [[f32; 4]; 16],
}

#[repr(C, align(16))]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Scaling {
    bg_scaling: Vec2,
    aspect_ratio: f32,
    _padding: f32,
}
