mod tracing;

#[derive(Default)]
pub(super) struct ApiState {
    pub(super) tracing: TracingBuffer,
}

pub(super) use self::tracing::add_to_linker;
#[cfg(test)]
pub(super) use self::tracing::bindings;
pub use self::tracing::{
    INITIAL_RECORD_CAPACITY, MAX_RECORD_COUNT, TraceField, TraceLevel, TraceRecord,
    TraceRecordKind, TraceValue, TracingBuffer,
};
