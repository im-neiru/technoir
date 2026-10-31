use std::{
    any::Any,
    future::Future,
    mem::ManuallyDrop,
    panic::{AssertUnwindSafe, catch_unwind, resume_unwind},
    pin::{Pin, pin},
    task::{Context, Poll, RawWaker, RawWakerVTable, Waker},
    thread,
};

use parking_lot::{Condvar, Mutex};
use triomphe::Arc;

pub fn block_on<F>(future: F) -> F::Output
where
    F: Future + Send,
    F::Output: Send,
{
    let shared = Arc::new(Shared::new());
    let waker = waker_from_shared(Arc::clone(&shared));
    let pinned_future = pin!(future);
    let future_slot = Mutex::new(pinned_future);

    let (shared_ref, future_slot_ref, waker_ref) = (&*shared, &future_slot, &waker);
    let helper_count = total_thread_count() - 1;

    let finished_outcome = thread::scope(|scope| {
        let helper_threads: Vec<_> = (0..helper_count)
            .filter_map(|helper_index| {
                thread::Builder::new()
                    .name(format!("block-on-helper-{helper_index}"))
                    .spawn_scoped(scope, move || {
                        drive_future(shared_ref, future_slot_ref, waker_ref)
                    })
                    .ok()
            })
            .collect();

        let mut finished_outcome = drive_future(shared_ref, future_slot_ref, waker_ref);

        for helper_thread in helper_threads {
            match helper_thread.join() {
                Ok(helper_outcome) => finished_outcome = finished_outcome.or(helper_outcome),
                Err(payload) => resume_unwind(payload),
            }
        }
        finished_outcome
    });

    match finished_outcome.expect("the future finished, so one thread holds its outcome") {
        Ok(output) => output,
        Err(payload) => resume_unwind(payload),
    }
}

const MIN_TOTAL_THREADS: usize = 2;
const MAX_TOTAL_THREADS: usize = 6;

fn total_thread_count() -> usize {
    let Ok(available) = thread::available_parallelism() else {
        return MIN_TOTAL_THREADS;
    };

    ((available.get() >> 1).saturating_sub(1)).clamp(MIN_TOTAL_THREADS, MAX_TOTAL_THREADS)
}

fn drive_future<F: Future>(
    shared: &Shared,
    future_slot: &Mutex<Pin<&mut F>>,
    waker: &Waker,
) -> Option<Result<F::Output, PanicPayload>> {
    let mut context = Context::from_waker(waker);

    while shared.wait_for_wake_or_finish() {
        let poll_outcome = catch_unwind(AssertUnwindSafe(|| {
            future_slot.lock().as_mut().poll(&mut context)
        }));

        match poll_outcome {
            Ok(Poll::Pending) => {}
            Ok(Poll::Ready(output)) => {
                shared.finish();

                return Some(Ok(output));
            }
            Err(payload) => {
                shared.finish();

                return Some(Err(payload));
            }
        }
    }
    None
}

struct Shared {
    state: Mutex<PollState>,
    state_changed: Condvar,
}

struct PollState {
    needs_poll: bool,
    finished: bool,
}

impl Shared {
    #[inline(always)]
    fn new() -> Self {
        Self {
            state: Mutex::new(PollState {
                needs_poll: true,
                finished: false,
            }),
            state_changed: Condvar::new(),
        }
    }

    #[inline(always)]
    fn request_poll(&self) {
        self.state.lock().needs_poll = true;

        self.state_changed.notify_one();
    }

    #[inline(always)]
    fn finish(&self) {
        self.state.lock().finished = true;
        self.state_changed.notify_all();
    }

    #[inline(always)]
    fn wait_for_wake_or_finish(&self) -> bool {
        let mut state = self.state.lock();
        self.state_changed
            .wait_while(&mut state, |state| !state.needs_poll && !state.finished);

        if state.finished {
            return false;
        }
        state.needs_poll = false;
        true
    }
}

static SHARED_WAKER_VTABLE: RawWakerVTable = RawWakerVTable::new(
    clone_shared_waker,
    wake_shared_waker,
    wake_shared_waker_by_ref,
    drop_shared_waker,
);

#[inline(always)]
fn waker_from_shared(shared: Arc<Shared>) -> Waker {
    let data = Arc::into_raw(shared).cast::<()>();

    unsafe { Waker::from_raw(RawWaker::new(data, &SHARED_WAKER_VTABLE)) }
}

#[inline(always)]
unsafe fn clone_shared_waker(data: *const ()) -> RawWaker {
    let borrowed_shared = unsafe { borrow_shared(data) };
    let cloned_shared = Arc::clone(&borrowed_shared);

    RawWaker::new(
        Arc::into_raw(cloned_shared).cast::<()>(),
        &SHARED_WAKER_VTABLE,
    )
}

#[inline(always)]
unsafe fn wake_shared_waker(data: *const ()) {
    let shared = unsafe { Arc::from_raw(data.cast::<Shared>()) };

    shared.request_poll();
}

#[inline(always)]
unsafe fn wake_shared_waker_by_ref(data: *const ()) {
    let borrowed_shared = unsafe { borrow_shared(data) };

    borrowed_shared.request_poll();
}

#[inline(always)]
unsafe fn drop_shared_waker(data: *const ()) {
    drop(unsafe { Arc::from_raw(data.cast::<Shared>()) });
}

type PanicPayload = Box<dyn Any + Send + 'static>;

#[inline(always)]
unsafe fn borrow_shared(data: *const ()) -> ManuallyDrop<Arc<Shared>> {
    ManuallyDrop::new(unsafe { Arc::from_raw(data.cast::<Shared>()) })
}
