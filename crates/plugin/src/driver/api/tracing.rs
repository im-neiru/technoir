use std::collections::VecDeque;

use self::bindings::technoir::plugin::tracing::{Field, Level};
use wasmtime::component::{HasSelf, Linker};

pub(in crate::driver) mod bindings {
    wasmtime::component::bindgen!({
        path: "../../wit",
        world: "api",
    });
}

pub use bindings::technoir::plugin::tracing::{
    Field as TraceField, Level as TraceLevel, Value as TraceValue,
};

pub const INITIAL_RECORD_CAPACITY: usize = 256;
pub const MAX_RECORD_COUNT: usize = 4096;

#[derive(Clone, Debug)]
pub struct TraceRecord {
    pub level: Option<TraceLevel>,
    pub target: Option<String>,
    pub fields: Vec<TraceField>,
    pub kind: TraceRecordKind,
}

#[derive(Clone, Debug)]
pub enum TraceRecordKind {
    Event {
        message: String,
        span: Option<u64>,
    },
    SpanStart {
        name: String,
        id: u64,
        parent: Option<u64>,
    },
    SpanEnd {
        id: u64,
    },
}

#[derive(Debug)]
pub struct TracingBuffer {
    records: VecDeque<TraceRecord>,
    next_span_id: u64,
}

impl Default for TracingBuffer {
    fn default() -> Self {
        Self {
            records: VecDeque::with_capacity(INITIAL_RECORD_CAPACITY),
            next_span_id: 1,
        }
    }
}

impl TracingBuffer {
    pub fn records(&self) -> &VecDeque<TraceRecord> {
        &self.records
    }

    fn record(&mut self, record: TraceRecord) {
        if self.records.len() == MAX_RECORD_COUNT {
            self.records.pop_front();
        }
        self.records.push_back(record);
    }
}

impl bindings::technoir::plugin::tracing::Host for super::super::InstanceState {
    fn enabled(&mut self, _level: Level, _target: Option<String>) -> bool {
        true
    }

    fn event(
        &mut self,
        level: Level,
        target: Option<String>,
        message: String,
        span: Option<u64>,
        fields: Vec<Field>,
    ) {
        self.api.tracing.record(TraceRecord {
            level: Some(level),
            target,
            fields,
            kind: TraceRecordKind::Event { message, span },
        });
    }

    fn span_start(
        &mut self,
        level: Level,
        target: Option<String>,
        name: String,
        parent: Option<u64>,
        fields: Vec<Field>,
    ) -> Option<u64> {
        let id = self.api.tracing.next_span_id;
        self.api.tracing.next_span_id = self.api.tracing.next_span_id.checked_add(1)?;
        self.api.tracing.record(TraceRecord {
            level: Some(level),
            target,
            fields,
            kind: TraceRecordKind::SpanStart { name, id, parent },
        });
        Some(id)
    }

    fn span_end(&mut self, id: u64) {
        self.api.tracing.record(TraceRecord {
            level: None,
            target: None,
            fields: Vec::new(),
            kind: TraceRecordKind::SpanEnd { id },
        });
    }
}

pub(in crate::driver) fn add_to_linker(
    linker: &mut Linker<super::super::InstanceState>,
) -> wasmtime::Result<()> {
    bindings::technoir::plugin::tracing::add_to_linker::<_, HasSelf<_>>(linker, |state| state)
}
