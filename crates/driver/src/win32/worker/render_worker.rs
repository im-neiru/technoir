use core::{
    ffi::c_void,
    mem::MaybeUninit,
    num::{NonZeroU8, NonZeroUsize},
    ptr::{self, NonNull},
    sync::atomic::{AtomicBool, Ordering::Relaxed},
};

use indexmap::IndexMap;
use renderer::{TargetSurface, WallpaperRenderer};
use rustc_hash::FxBuildHasher;
use triomphe::Arc;
use windows_sys::Win32::{
    Foundation::{CloseHandle, GetLastError, WAIT_OBJECT_0},
    System::Threading::{
        CREATE_SUSPENDED, CreateThread, INFINITE, ResumeThread, Sleep, WaitForSingleObject,
    },
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
    state: Arc<State>,
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

        let renderer = WallpaperRenderer::new();

        let state = Arc::new(State {
            renderer,
            targets,
            is_shutdown: AtomicBool::new(false),
        });

        let Some(thread) = NonNull::new(unsafe {
            CreateThread(
                ptr::null(),
                RENDER_STACK,
                Some(thread_proc),
                Arc::into_raw(state.clone()).cast(),
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
            state,
        })
    }

    #[inline]
    pub(super) fn resume(&self) {
        unsafe { ResumeThread(self.thread.as_ptr()) };
    }

    #[inline]
    pub(super) fn shutdown(self) {
        let _ = self;
    }
}

impl Drop for RenderWorker {
    fn drop(&mut self) {
        self.state.is_shutdown.store(true, Relaxed);

        unsafe {
            ResumeThread(self.thread.as_ptr());

            let wait_result = WaitForSingleObject(self.thread.as_ptr(), INFINITE);

            debug_assert_eq!(wait_result, WAIT_OBJECT_0);

            CloseHandle(self.thread.as_ptr());
        }
    }
}

struct State {
    renderer: WallpaperRenderer,
    targets: IndexMap<NonZeroUsize, TargetSurface, FxBuildHasher>,
    is_shutdown: AtomicBool,
}

unsafe extern "system" fn thread_proc(ptr: *mut c_void) -> u32 {
    let Some(state) = (unsafe { ptr.cast::<State>().as_mut() }) else {
        return 0;
    };

    loop {
        for (_, target) in state.targets.iter() {
            state.renderer.render(target);
        }

        unsafe { Sleep(1) };

        if state.is_shutdown.load(Relaxed) {
            break;
        }
    }

    0
}
