use crate::gpu_context::GpuContext;

pub struct WallpaperRenderer {
    ctx: GpuContext,
}

impl WallpaperRenderer {
    #[inline]
    pub fn new() -> Self {
        let ctx = GpuContext::get();

        Self { ctx }
    }
}

impl Default for WallpaperRenderer {
    #[inline]
    fn default() -> Self {
        Self::new()
    }
}
