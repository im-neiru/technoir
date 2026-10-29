use wgpu::{
    Instance, InstanceDescriptor,
    rwh::{self, DisplayHandle, HasDisplayHandle, HasWindowHandle},
    wgt::WgpuHasDisplayHandle,
};

use super::{GpuContext, GpuState};

impl GpuState {
    #[inline(always)]
    pub(super) const fn new(handle: impl HasDisplayHandle + HasWindowHandle) -> Self {
        let display = handle.display_handle().unwrap();

        let instance = Instance::new(InstanceDescriptor::new_with_display_handle(Box::into(
            DisplayHandleWrapper(display),
        )));

        Self { instance: () }
    }
}

#[derive(Debug)]
struct DisplayHandleWrapper<'a>(DisplayHandle<'a>);

impl<'a> HasDisplayHandle for DisplayHandleWrapper<'a> {
    #[inline(always)]
    fn display_handle(&self) -> Result<DisplayHandle<'a>, rwh::HandleError> {
        Ok(self.0)
    }
}

impl<'a> Into<WgpuHasDisplayHandle> for DisplayHandleWrapper<'a> {
    fn into(self) -> WgpuHasDisplayHandle {
        todo!()
    }
}
