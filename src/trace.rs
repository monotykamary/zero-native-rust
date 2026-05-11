use crate::geometry::{RectF, SizeF};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Level {
    Trace,
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

impl Level {
    pub fn name(self) -> &'static str {
        match self {
            Self::Trace => "trace",
            Self::Debug => "debug",
            Self::Info => "info",
            Self::Warn => "warn",
            Self::Error => "err",
            Self::Fatal => "fatal",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Event,
    SpanBegin,
    SpanEnd,
    Counter,
    Gauge,
    Frame,
}

impl Kind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Event => "event",
            Self::SpanBegin => "span_begin",
            Self::SpanEnd => "span_end",
            Self::Counter => "counter",
            Self::Gauge => "gauge",
            Self::Frame => "frame",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Timestamp {
    pub ns: i128,
}

impl Timestamp {
    pub fn from_nanoseconds(ns: i128) -> Self {
        Self { ns }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Duration {
    pub ns: u64,
}

impl Duration {
    pub fn from_nanoseconds(ns: u64) -> Self {
        Self { ns }
    }
    pub fn from_microseconds(us: u64) -> Self {
        Self { ns: us * 1_000 }
    }
    pub fn from_milliseconds(ms: u64) -> Self {
        Self { ns: ms * 1_000_000 }
    }
    pub fn from_seconds(s: u64) -> Self {
        Self { ns: s * 1_000_000_000 }
    }
}

#[derive(Debug, Clone)]
pub enum FieldValue {
    String(String),
    Boolean(bool),
    Int(i64),
    Uint(u64),
    Float(f64),
}

#[derive(Debug, Clone)]
pub struct Field {
    pub key: String,
    pub value: FieldValue,
}

pub fn string_field(key: &str, value: &str) -> Field {
    Field {
        key: key.to_string(),
        value: FieldValue::String(value.to_string()),
    }
}

pub fn uint_field(key: &str, value: u64) -> Field {
    Field {
        key: key.to_string(),
        value: FieldValue::Uint(value),
    }
}

pub fn float_field(key: &str, value: f64) -> Field {
    Field {
        key: key.to_string(),
        value: FieldValue::Float(value),
    }
}

#[derive(Debug, Clone)]
pub struct Record {
    pub timestamp: Timestamp,
    pub level: Level,
    pub kind: Kind,
    pub name: String,
    pub message: Option<String>,
    pub fields: Vec<Field>,
    pub span_id: Option<u64>,
    pub parent_span_id: Option<u64>,
    pub duration: Option<Duration>,
}

pub fn event_record(
    timestamp: Timestamp,
    level: Level,
    name: &str,
    message: Option<&str>,
    fields: Vec<Field>,
) -> Record {
    Record {
        timestamp,
        level,
        kind: Kind::Event,
        name: name.to_string(),
        message: message.map(|s| s.to_string()),
        fields,
        span_id: None,
        parent_span_id: None,
        duration: None,
    }
}

pub trait Sink {
    fn write(&mut self, record: Record);
}

pub struct BufferSink {
    pub records: Vec<Record>,
    pub capacity: usize,
}

impl BufferSink {
    pub fn new(capacity: usize) -> Self {
        Self {
            records: Vec::with_capacity(capacity),
            capacity,
        }
    }
}

impl Sink for BufferSink {
    fn write(&mut self, record: Record) {
        if self.records.len() < self.capacity {
            self.records.push(record);
        }
    }
}
