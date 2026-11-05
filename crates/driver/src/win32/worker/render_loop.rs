use core::{
    ffi::c_void,
    mem::MaybeUninit,
    ptr::{self, NonNull},
};

use triomphe::Arc;
use windows_sys::Win32::{
    Foundation::GetLastError,
    System::Threading::{CREATE_SUSPENDED, CreateThread},
};

use crate::errors::WindowsOsError;

pub struct RenderWorker {
    thread: NonNull<c_void>,
    thread_id: u32,
}

const RENDER_STACK: usize = 4 * 1024 * 1024; // 4MB

impl RenderWorker {
    pub fn new() -> Result<Self, WindowsOsError> {
        let mut thread_id = MaybeUninit::uninit();

        let state = Arc::new(State {});

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
            return Err(WindowsOsError::FailedToCreateRenderThread {
                error_code: unsafe { GetLastError() },
            });
        };

        Ok(Self {
            thread,
            thread_id: unsafe { thread_id.assume_init() },
        })
    }
}

struct State {}

unsafe extern "system" fn thread_proc(ptr: *mut c_void) -> u32 {
    let Some(state) = (unsafe { ptr.cast::<State>().as_mut() }) else {
        return 0;
    };

    0
}
