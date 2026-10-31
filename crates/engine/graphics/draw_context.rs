use wgpu::TextureView;

use crate::graphics::{Context, WindowSurface};

pub struct DrawContext<'ctx, 'v> {
    ctx: &'ctx Context,
    view: &'v TextureView,
}

impl<'ctx, 'v> super::Context {
    #[inline(always)]
    pub(crate) fn new(&'ctx self, view: &'v TextureView) -> DrawContext<'ctx, 'v> {
        DrawContext { ctx: self, view }
    }
}
