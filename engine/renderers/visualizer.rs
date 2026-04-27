use vello::wgpu;

use super::Renderer;
use crate::samplers::SpectrumAudioLoopback;

pub struct Visualizer {
    renderer: Renderer,

    prev_spectrum_l: [f32; BAR_COUNT],
    prev_spectrum_r: [f32; BAR_COUNT],

    bar_layout: Vec<BarLayoutItem>,

    prev_lum: f32,
    lum_buffer: wgpu::Buffer,
    bg_pipeline: wgpu::RenderPipeline,
    bg_bind_group: Option<wgpu::BindGroup>,
    bg_sampler: wgpu::Sampler,
    bg_bind_group_layout: wgpu::BindGroupLayout,
}

const BAR_COUNT: usize = 64;

#[derive(Clone, Copy)]
struct BarLayoutItem {
    x: f32,
    w: f32,
}

const BG_SHADER: &str = "
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) tex_coords: vec2<f32>,
};

struct BrightnessUniform {
    value: f32,
};

@group(0) @binding(0) var t_diffuse: texture_2d<f32>;
@group(0) @binding(1) var s_diffuse: sampler;

@group(0) @binding(2) var<uniform> brightness: BrightnessUniform;

@vertex
fn vs_main(@builtin(vertex_index) in_vertex_index: u32) -> VertexOutput {
    var out: VertexOutput;
    let x = f32(1 - i32(in_vertex_index)) * 3.0;
    let y = f32(i32(in_vertex_index & 1u) * 2 - 1) * 3.0;
    out.clip_position = vec4<f32>(x, y, 0.0, 1.0);
    out.tex_coords = vec2<f32>(x * 0.5 + 0.5, 1.0 - (y * 0.5 + 0.5));
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let color = textureSample(t_diffuse, s_diffuse, in.tex_coords);

    return vec4<f32>(color.rgb * brightness.value, color.a);
}
";

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

        let lum_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Brightness Buffer"),
            size: 16,
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
                    blend: Some(wgpu::BlendState::REPLACE),
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

        let bg_bind_group = bg_texture.as_ref().map(|tex| {
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
                        resource: lum_buffer.as_entire_binding(),
                    },
                ],
            })
        });

        let mut result = Self {
            renderer,
            prev_spectrum_l: [0.0; BAR_COUNT],
            prev_spectrum_r: [0.0; BAR_COUNT],
            bar_layout: Vec::with_capacity(BAR_COUNT),
            bg_pipeline,
            bg_bind_group,
            bg_sampler,
            bg_bind_group_layout,
            lum_buffer,
            prev_lum: 0.,
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

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer.resize(width, height);

        let w = width as f32;
        let gap = 12.0;

        let target_width = w * 0.6;
        let start_x = (w - target_width) / 2.0;
        let bar_width = (target_width - (gap * (BAR_COUNT - 1) as f32)) / BAR_COUNT as f32;

        self.bar_layout = (0..BAR_COUNT)
            .map(|i| {
                let x = start_x + i as f32 * (bar_width + gap);
                BarLayoutItem { x, w: bar_width }
            })
            .collect();

        self.prev_spectrum_l = [0.0; BAR_COUNT];
        self.prev_spectrum_r = [0.0; BAR_COUNT];
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

        let target_lum = (1.0 - avg_left.max(avg_right) * 0.2).max(0.3);

        let alpha = 0.2;

        let lum_value = self.prev_lum + (target_lum - self.prev_lum) * alpha;
        self.prev_lum = lum_value;

        self.renderer.queue.write_buffer(
            &self.lum_buffer,
            0,
            bytemuck::cast_slice(&[lum_value, 0.0, 0.0, 0.0]),
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

            if let Some(bind_group) = &self.bg_bind_group {
                rp.set_pipeline(&self.bg_pipeline);
                rp.set_bind_group(0, bind_group, &[]);
                rp.draw(0..3, 0..1);
            }
        }

        self.renderer.queue.submit(Some(encoder.finish()));

        frame.present();
    }
}
