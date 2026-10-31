use wgpu::{Adapter, Device, Instance, Queue};

mod initialize;
mod lifecycle;

pub(crate) struct GpuState {
    pub(crate) instance: Instance,
    pub(crate) adapter: Adapter,
    pub(crate) device: Device,
    pub(crate) queue: Queue,
}

pub(crate) struct GpuContext(core::marker::PhantomData<&'static GpuState>);
