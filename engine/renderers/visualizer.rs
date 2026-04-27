use std::sync::Arc;
use vello::peniko::{Blob, Color, ImageAlphaType, ImageData, ImageFormat};
use vello::{
    Scene,
    kurbo::{Affine, Rect},
    wgpu,
};

// Assuming these exist in your project structure
use super::Renderer2d;
use crate::samplers::SpectrumAudioLoopback;

pub struct Visualizer {
    renderer2d: Renderer2d,
    scene: Scene,
    prev_spectrum_l: [f32; BAR_COUNT],
    prev_spectrum_r: [f32; BAR_COUNT],
    background: Option<ImageData>,

    bar_layout: Vec<BarLayoutItem>,
}

const BAR_COUNT: usize = 64;

#[derive(Clone, Copy)]
struct BarLayoutItem {
    x: f32,
    w: f32,
}

impl Visualizer {
    pub async fn new(
        instance: &wgpu::Instance,
        surface: wgpu::Surface<'static>,
        width: u32,
        height: u32,
    ) -> Self {
        let renderer2d = Renderer2d::new(instance, surface, width, height).await;

        let mut vis = Self {
            renderer2d,
            scene: Scene::new(),
            prev_spectrum_l: [0.0; BAR_COUNT],
            prev_spectrum_r: [0.0; BAR_COUNT],
            background: Self::load_vello_image("./sample/image.jpg"),
            bar_layout: Vec::with_capacity(BAR_COUNT),
        };

        vis.resize(width, height);
        vis
    }

    fn load_vello_image(path: impl AsRef<std::path::Path>) -> Option<ImageData> {
        let img = image::open(path).ok()?.to_rgba8();
        let (width, height) = img.dimensions();

        let raw_data = img.into_raw();

        let data: Arc<dyn AsRef<[u8]> + Send + Sync> = Arc::new(raw_data);

        Some(ImageData {
            data: Blob::new(data),
            format: ImageFormat::Rgba8,
            alpha_type: ImageAlphaType::Alpha,
            width,
            height,
        })
    }

    pub fn resize(&mut self, width: u32, height: u32) {
        self.renderer2d.resize(width, height);

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

        for i in 0..64.min(left.len()).min(right.len()) {
            self.prev_spectrum_l[i] = self.prev_spectrum_l[i] * 0.8 + left[i] * 0.2;
            self.prev_spectrum_r[i] = self.prev_spectrum_r[i] * 0.8 + right[i] * 0.2;
        }

        self.scene.reset();

        let w = self.renderer2d.renderer.config.width as f64;
        let h = self.renderer2d.renderer.config.height as f64;
        let v_center = h / 2.0;
        let max_amp = h * 0.02;

        if let Some(bg) = &self.background {
            let scale = (w / bg.width as f64).max(h / bg.height as f64);
            let offset_x = (w - bg.width as f64 * scale) / 2.0;
            let offset_y = (h - bg.height as f64 * scale) / 2.0;

            self.scene.draw_image(
                bg,
                Affine::translate((offset_x, offset_y)).then_scale(scale),
            );
        }

        for (i, &BarLayoutItem { x, w: bar_w }) in self.bar_layout.iter().enumerate() {
            let amp_l = (self.prev_spectrum_l[i] as f64 * max_amp).max(1.5);
            let amp_r = (self.prev_spectrum_r[i] as f64 * max_amp).max(1.5);

            let tweak = amp_r.max(amp_l) as f32 / max_amp as f32;
            let x_norm = i as f32 / self.bar_layout.len() as f32;
            let hue = (x_norm * 50.0 + 190.0 + (time * 10.0) - tweak * 0.6).rem_euclid(360.0);
            let color = Self::hsl_to_rgb(hue, 0.95, 0.40 + tweak * 0.045);

            self.scene.fill(
                vello::peniko::Fill::NonZero,
                Affine::IDENTITY,
                color,
                None,
                &Rect::new(
                    x as f64,
                    v_center - amp_l,
                    (x + bar_w) as f64,
                    v_center + amp_r,
                ),
            );
        }

        self.renderer2d.render_scene(&self.scene, Color::BLACK);
    }

    fn hsl_to_rgb(h: f32, s: f32, l: f32) -> Color {
        let h = h % 360.0;
        let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
        let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
        let m = l - c * 0.5;

        let (r, g, b) = match (h / 60.0) as i32 {
            0 => (c, x, 0.0),
            1 => (x, c, 0.0),
            2 => (0.0, c, x),
            3 => (0.0, x, c),
            4 => (x, 0.0, c),
            _ => (c, 0.0, x),
        };

        Color::from_rgb8(
            ((r + m) * 255.0) as u8,
            ((g + m) * 255.0) as u8,
            ((b + m) * 255.0) as u8,
        )
    }
}
