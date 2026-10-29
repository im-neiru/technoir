use crate::gpu_context::GpuContext;

pub struct Renderer {
    ctx: GpuContext,
}

impl Renderer {
    #[cfg(not(all(feature = "web", any(target_arch = "wasm32", target_arch = "wasm64"))))]
    pub fn new(
        display: Box<dyn wgpu::wgt::WgpuHasDisplayHandle>,
        window: wgpu::rwh::WindowHandle,
    ) -> Self {
        let ctx = GpuContext::get_or_init(display, window);

        Self { ctx }
    }
}
