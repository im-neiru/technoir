use core::{cell::UnsafeCell, marker::PhantomData, sync::atomic::AtomicU32};

use wgpu::{Adapter, Device, Instance, Queue};

static CONTEXT: UnsafeCell<Option<GpuContext>> = UnsafeCell::new(None);
static REF_COUNT: AtomicU32 = AtomicU32::new(0);

pub(crate) struct GpuContext(PhantomData<&'static GpuContext>);

struct GpuState {
    instance: Instance,
}
