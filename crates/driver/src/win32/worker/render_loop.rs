use core::{
    ffi::c_void,
    mem::MaybeUninit,
    num::{NonZeroU8, NonZeroUsize},
    ptr::{self, NonNull},
};

use indexmap::IndexMap;
use renderer::TargetSurface;
use rustc_hash::FxBuildHasher;
use triomphe::Arc;
use windows_sys::Win32::{
    Foundation::GetLastError,
    System::Threading::{CREATE_SUSPENDED, CreateThread},
};

use super::super::{
    WallpaperWindow,
    get_client_area::get_client_area,
    handles::{DisplayHandle, WindowHandle},
};
use crate::errors::{StartupError, WindowsOsError};

pub struct RenderWorker {
    thread: NonNull<c_void>,
    thread_id: u32,
}

const RENDER_STACK: usize = 4 * 1024 * 1024; // 4MB

impl RenderWorker {
    pub fn new(
        instance: NonNull<c_void>,
        windows: &[WallpaperWindow],
    ) -> Result<Self, StartupError> {
        let mut thread_id = MaybeUninit::uninit();

        let mut targets = IndexMap::with_capacity_and_hasher(windows.len(), FxBuildHasher);

        for w in windows {
            let target = pollster::block_on(async move {
                let [width, height] = get_client_area(w.handle);
                let window_handle = WindowHandle::new(instance, w.handle);

                TargetSurface::new(DisplayHandle, window_handle, width, height, unsafe {
                    NonZeroU8::new_unchecked(3)
                })
                .await
            })
            .map_err(StartupError::GraphicsInit)?;

            targets.insert(w.monitor, target);
        }

        let state = Arc::new(State { targets });

        let Some(thread) = NonNull::new(unsafe {
            CreateThread(
                ptr::null(),
                RENDER_STACK,
                Some(thread_proc),
                Arc::into_raw(state).cast(),
                CREATE_SUSPENDED,
                thread_id.as_mut_ptr(),
            )
        }) else {
            return Err(StartupError::WindowOs(
                WindowsOsError::FailedToCreateRenderThread {
                    error_code: unsafe { GetLastError() },
                },
            ));
        };

        Ok(Self {
            thread,
            thread_id: unsafe { thread_id.assume_init() },
        })
    }
}

struct State {
    targets: IndexMap<NonZeroUsize, TargetSurface, FxBuildHasher>,
}

unsafe extern "system" fn thread_proc(ptr: *mut c_void) -> u32 {
    let Some(state) = (unsafe { ptr.cast::<State>().as_mut() }) else {
        return 0;
    };

    0
}
