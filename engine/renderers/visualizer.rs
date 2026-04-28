use vello::wgpu;

use super::Renderer;
use crate::samplers::SpectrumAudioLoopback;

pub struct Visualizer {
    renderer: Renderer,

    prev_spectrum_l: [f32; SAMPLE_COUNT],
    prev_spectrum_r: [f32; SAMPLE_COUNT],

    prev_lum: f32,
    uniforms_buffer: wgpu::Buffer,
    pipeline: wgpu::RenderPipeline,
    bind_group: Option<wgpu::BindGroup>,
    noise_texture: wgpu::Texture,
    noise_sampler: wgpu::Sampler,
}

const SAMPLE_COUNT: usize = 64;

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

        let uniforms_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Uniforms Buffer"),
            size: core::mem::size_of::<Uniforms>() as u64,
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
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
            bind_group_layouts: &[&bg_bind_group_layout],
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

        let bg_texture = Self::load_bg_texture("./sample/image.jpg", device, &renderer.queue);

        let noise_texture = Self::load_noise_texture(device, &renderer.queue).unwrap();

        let noise_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Noise Sampler"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        let bind_group = bg_texture.as_ref().map(|tex| {
            let view = tex.create_view(&wgpu::TextureViewDescriptor::default());
            device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("BG Bind Group"),
                layout: &bg_bind_group_layout,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::TextureView(&view),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::Sampler(&bg_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: wgpu::BindingResource::TextureView(
                            &noise_texture.create_view(&Default::default()),
                        ),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: wgpu::BindingResource::Sampler(&noise_sampler),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: uniforms_buffer.as_entire_binding(),
                    },
                ],
            })
        });

        let mut result = Self {
            renderer,
            prev_spectrum_l: [0.0; SAMPLE_COUNT],
            prev_spectrum_r: [0.0; SAMPLE_COUNT],
            pipeline: bg_pipeline,
            bind_group,
            uniforms_buffer,
            prev_lum: 0.,
            noise_texture,
            noise_sampler,
        };

        result.resize(width, height);
        result
    }

    fn load_bg_texture(
        path: impl AsRef<std::path::Path>,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
    ) -> Option<wgpu::Texture> {
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

        Some(texture)
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
    }

    pub fn render(&mut self, audio: &mut SpectrumAudioLoopback, time: f32) {
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
        let mut uniforms = Uniforms {
            lum: lum_value,
            time,
            pad: [0.0; 2],
            spectrum: [[0.0; 4]; 16],
        };

        let count = left.len().min(right.len()).min(16);

        for i in 0..count {
            let l = left[i];
            let r = right[i];
            let mono = (l + r) * 0.80 - 0.05;

            uniforms.spectrum[i] = [mono, mono, mono, mono];
        }

        self.renderer.queue.write_buffer(
            &self.uniforms_buffer,
            0,
            bytemuck::cast_slice(&[uniforms]),
        );

        let frame = self.renderer.begin_frame();
        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());

        let mut encoder =
            self.renderer
                .device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Frame Encoder"),
                });

        {
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

            if let Some(bind_group) = &self.bind_group {
                rp.set_pipeline(&self.pipeline);
                rp.set_bind_group(0, bind_group, &[]);
                rp.draw(0..3, 0..1);
            }
        }

        self.renderer.queue.submit(Some(encoder.finish()));

        frame.present();
    }
}

use bytemuck::{Pod, Zeroable};

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
struct Uniforms {
    pub lum: f32,
    pub time: f32,
    pub pad: [f32; 2],

    pub spectrum: [[f32; 4]; 16],
}
