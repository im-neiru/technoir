mod initialize;
mod lifecycle;

pub(crate) struct GpuState {
    pub(crate) instance: wgpu::Instance,
}

pub(crate) struct GpuContext(core::marker::PhantomData<&'static GpuState>);
