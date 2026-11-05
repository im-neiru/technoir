use core::iter;

use wgpu::{
    CommandEncoderDescriptor, CurrentSurfaceTexture, RenderPassColorAttachment,
    RenderPassDescriptor, TextureViewDescriptor,
};

use crate::{TargetSurface, gpu_context::GpuContext};

pub struct WallpaperRenderer {
    ctx: GpuContext,
}

impl WallpaperRenderer {
    #[inline]
    pub fn new() -> Self {
        let ctx = GpuContext::get();

        Self { ctx }
    }

    pub fn render(&self, surface: &TargetSurface) {
        let (CurrentSurfaceTexture::Suboptimal(frame) | CurrentSurfaceTexture::Success(frame)) =
            surface.surface.get_current_texture()
        else {
            return;
        };

        let view = frame.texture.create_view(&TextureViewDescriptor::default());

        let mut encoder = self
            .ctx
            .device
            .create_command_encoder(&CommandEncoderDescriptor {
                label: Some("clear"),
            });

        {
            let _pass = encoder.begin_render_pass(&RenderPassDescriptor {
                label: Some("clear"),
                color_attachments: &[Some(RenderPassColorAttachment {
                    view: &view,
                    depth_slice: None,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.04,
                            g: 0.04,
                            b: 0.08,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });
        }

        self.ctx.queue.submit(iter::once(encoder.finish()));
        self.ctx.queue.present(frame);
    }
}

impl Default for WallpaperRenderer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
