use core::{
    cell::UnsafeCell,
    hint::spin_loop,
    marker::PhantomData,
    mem::MaybeUninit,
    ops::Deref,
    sync::atomic::{AtomicU32, Ordering, fence},
};

use wgpu::{Surface, SurfaceTargetUnsafe, wgt::WgpuHasDisplayHandle};

use super::{GpuContext, GpuState};

impl GpuContext {
    pub(crate) fn get_or_init(
        display_handle: Box<dyn WgpuHasDisplayHandle>,
        surface_handle: SurfaceTargetUnsafe,
    ) -> (Self, Surface<'static>) {
        loop {
            match SLOT_FLAG.load(Ordering::Acquire) {
                UNINITIALIZED => {
                    if SLOT_FLAG
                        .compare_exchange(
                            UNINITIALIZED,
                            INITIALIZING,
                            Ordering::Acquire,
                            Ordering::Relaxed,
                        )
                        .is_ok()
                    {
                        let surface = initialize_slot(display_handle, surface_handle);

                        return (Self(PhantomData), surface);
                    }
                }
                INITIALIZING | DESTROYING => spin_loop(),
                INITIALIZED => {
                    if try_acquire_ref() {
                        let surface = get_state().create_surface(surface_handle);

                        return (Self(PhantomData), surface);
                    }

                    spin_loop();
                }
                _ => unreachable!(),
            }
        }
    }
}

impl Clone for GpuContext {
    fn clone(&self) -> Self {
        if !try_acquire_ref() {
            unreachable!("live GpuContext has no reference");
        }

        Self(PhantomData)
    }
}

impl Deref for GpuContext {
    type Target = GpuState;

    fn deref(&self) -> &Self::Target {
        get_state()
    }
}

impl Drop for GpuContext {
    fn drop(&mut self) {
        release_ref();
    }
}

#[repr(u32)]
#[derive(Clone, Copy)]
enum SlotFlag {
    Uninitialized = 0,
    Initializing = 1,
    Initialized = 2,
    Destroying = 3,
}

const UNINITIALIZED: u32 = SlotFlag::Uninitialized as u32;
const INITIALIZING: u32 = SlotFlag::Initializing as u32;
const INITIALIZED: u32 = SlotFlag::Initialized as u32;
const DESTROYING: u32 = SlotFlag::Destroying as u32;
const MAX_REFS: u32 = u32::MAX;

static SLOT_FLAG: AtomicU32 = AtomicU32::new(UNINITIALIZED);
static REF_COUNT: AtomicU32 = AtomicU32::new(0);
static SLOT: GpuSlot = GpuSlot(UnsafeCell::new(MaybeUninit::uninit()));

struct GpuSlot(UnsafeCell<MaybeUninit<GpuState>>);

unsafe impl Sync for GpuSlot {}

struct TransitionGuard {
    settle_to: SlotFlag,
}

impl TransitionGuard {
    #[inline(always)]
    const fn new(settle_to: SlotFlag) -> Self {
        Self { settle_to }
    }
}

impl Drop for TransitionGuard {
    fn drop(&mut self) {
        SLOT_FLAG.store(self.settle_to as u32, Ordering::Release);
    }
}

fn initialize_slot(
    display_handle: Box<dyn WgpuHasDisplayHandle>,
    surface_handle: SurfaceTargetUnsafe,
) -> Surface<'static> {
    let mut transition = TransitionGuard::new(SlotFlag::Uninitialized);

    let (state, surface) = GpuState::new(display_handle, surface_handle);

    unsafe {
        (*SLOT.0.get()).write(state);
    }

    REF_COUNT.store(1, Ordering::Release);
    transition.settle_to = SlotFlag::Initialized;

    surface
}

fn try_acquire_ref() -> bool {
    match atomic_update(
        &REF_COUNT,
        Ordering::Relaxed,
        Ordering::Acquire,
        |refs| match refs {
            0 | MAX_REFS => None,
            _ => Some(refs + 1),
        },
    ) {
        Ok(_) => true,
        Err(0) => false,
        Err(MAX_REFS) => std::process::abort(),
        Err(_) => unreachable!(),
    }
}

fn release_ref() {
    let previous = atomic_update(&REF_COUNT, Ordering::Relaxed, Ordering::Release, |refs| {
        refs.checked_sub(1)
    })
    .unwrap_or_else(|_| unreachable!());

    if previous != 1 {
        return;
    }

    SLOT_FLAG
        .compare_exchange(
            INITIALIZED,
            DESTROYING,
            Ordering::Acquire,
            Ordering::Relaxed,
        )
        .unwrap_or_else(|_| unreachable!());

    let _transition = TransitionGuard::new(SlotFlag::Uninitialized);

    fence(Ordering::Acquire);

    unsafe {
        (*SLOT.0.get()).assume_init_drop();
    }
}

fn atomic_update(
    atomic: &AtomicU32,
    fetch_order: Ordering,
    set_order: Ordering,
    mut f: impl FnMut(u32) -> Option<u32>,
) -> Result<u32, u32> {
    let mut prev = atomic.load(fetch_order);

    while let Some(next) = f(prev) {
        match atomic.compare_exchange_weak(prev, next, set_order, fetch_order) {
            x @ Ok(_) => return x,
            Err(next_prev) => prev = next_prev,
        }
    }

    Err(prev)
}

#[inline(always)]
fn get_state() -> &'static GpuState {
    unsafe { (*SLOT.0.get()).assume_init_ref() }
}
