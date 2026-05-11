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

pub fn duration_between(start: Timestamp, end: Timestamp) -> Duration {
    Duration::from_nanoseconds((end.ns - start.ns).max(0) as u64)
}

#[derive(Debug, Clone, PartialEq)]
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
    Field { key: key.to_string(), value: FieldValue::String(value.to_string()) }
}

pub fn boolean_field(key: &str, value: bool) -> Field {
    Field { key: key.to_string(), value: FieldValue::Boolean(value) }
}

pub fn int_field(key: &str, value: i64) -> Field {
    Field { key: key.to_string(), value: FieldValue::Int(value) }
}

pub fn uint_field(key: &str, value: u64) -> Field {
    Field { key: key.to_string(), value: FieldValue::Uint(value) }
}

pub fn float_field(key: &str, value: f64) -> Field {
    Field { key: key.to_string(), value: FieldValue::Float(value) }
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

pub fn span_begin_record(
    timestamp: Timestamp,
    level: Level,
    span_id: u64,
    parent_span_id: Option<u64>,
    name: &str,
    fields: Vec<Field>,
) -> Record {
    Record {
        timestamp,
        level,
        kind: Kind::SpanBegin,
        name: name.to_string(),
        message: None,
        fields,
        span_id: Some(span_id),
        parent_span_id,
        duration: None,
    }
}

pub fn span_end_record(
    timestamp: Timestamp,
    level: Level,
    span_id: u64,
    parent_span_id: Option<u64>,
    name: &str,
    message: Option<&str>,
    duration: Duration,
    fields: Vec<Field>,
) -> Record {
    Record {
        timestamp,
        level,
        kind: Kind::SpanEnd,
        name: name.to_string(),
        message: message.map(|s| s.to_string()),
        fields,
        span_id: Some(span_id),
        parent_span_id,
        duration: Some(duration),
    }
}

pub fn counter_record(
    timestamp: Timestamp,
    name: &str,
    value: u64,
    fields: Vec<Field>,
) -> Record {
    Record {
        timestamp,
        level: Level::Info,
        kind: Kind::Counter,
        name: name.to_string(),
        message: None,
        fields: vec![uint_field("value", value)],
        span_id: None,
        parent_span_id: None,
        duration: None,
    }
}

pub fn gauge_record(
    timestamp: Timestamp,
    name: &str,
    value: f64,
    fields: Vec<Field>,
) -> Record {
    Record {
        timestamp,
        level: Level::Info,
        kind: Kind::Gauge,
        name: name.to_string(),
        message: None,
        fields: vec![float_field("value", value)],
        span_id: None,
        parent_span_id: None,
        duration: None,
    }
}

pub fn frame_record(
    timestamp: Timestamp,
    name: &str,
    duration: Duration,
    index: u64,
    fields: Vec<Field>,
) -> Record {
    Record {
        timestamp,
        level: Level::Info,
        kind: Kind::Frame,
        name: name.to_string(),
        message: None,
        fields: vec![uint_field("index", index)],
        span_id: None,
        parent_span_id: None,
        duration: Some(duration),
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

    pub fn written(&self) -> &[Record] {
        &self.records
    }
}

impl Sink for BufferSink {
    fn write(&mut self, record: Record) {
        if self.records.len() < self.capacity {
            self.records.push(record);
        }
    }
}

pub struct FanoutSink<'a> {
    pub sinks: Vec<&'a mut dyn Sink>,
}

impl<'a> FanoutSink<'a> {
    pub fn new(sinks: Vec<&'a mut dyn Sink>) -> Self {
        Self { sinks }
    }
}

impl<'a> Sink for FanoutSink<'a> {
    fn write(&mut self, record: Record) {
        for sink in &mut self.sinks {
            sink.write(record.clone());
        }
    }
}

pub fn format_text(record: &Record) -> String {
    let mut parts = Vec::new();
    parts.push(format!("ts={}", record.timestamp.ns));
    parts.push(format!("level={}", record.level.name()));
    parts.push(format!("kind={}", record.kind.name()));
    parts.push(format!("name=\"{}\"", escape_text_string(&record.name)));
    if let Some(ref msg) = record.message {
        parts.push(format!("message=\"{}\"", escape_text_string(msg)));
    }
    if let Some(span_id) = record.span_id {
        parts.push(format!("span_id={}", span_id));
    }
    if let Some(parent_span_id) = record.parent_span_id {
        parts.push(format!("parent_span_id={}", parent_span_id));
    }
    if let Some(ref dur) = record.duration {
        parts.push(format!("duration_ns={}", dur.ns));
    }
    for field in &record.fields {
        parts.push(format!("{}={}", field.key, format_field_value_text(&field.value)));
    }
    parts.join(" ")
}

fn format_field_value_text(value: &FieldValue) -> String {
    match value {
        FieldValue::String(s) => format!("\"{}\"", escape_text_string(s)),
        FieldValue::Boolean(b) => if *b { "true".into() } else { "false".into() },
        FieldValue::Int(i) => i.to_string(),
        FieldValue::Uint(u) => u.to_string(),
        FieldValue::Float(f) => format!("{}", f),
    }
}

fn escape_text_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c => out.push(c),
        }
    }
    out
}

pub fn format_json_line(record: &Record) -> String {
    let mut s = String::new();
    s.push_str(&format!(
        "{{\"timestamp_ns\":{},\"level\":\"{}\",\"kind\":\"{}\",\"name\":",
        record.timestamp.ns,
        record.level.name(),
        record.kind.name(),
    ));
    s.push_str(&json_string(&record.name));
    if let Some(ref msg) = record.message {
        s.push_str(",\"message\":");
        s.push_str(&json_string(msg));
    }
    if let Some(span_id) = record.span_id {
        s.push_str(&format!(",\"span_id\":{}", span_id));
    }
    if let Some(parent_span_id) = record.parent_span_id {
        s.push_str(&format!(",\"parent_span_id\":{}", parent_span_id));
    }
    if let Some(ref dur) = record.duration {
        s.push_str(&format!(",\"duration_ns\":{}", dur.ns));
    }
    s.push_str(",\"fields\":{");
    let mut first = true;
    for field in &record.fields {
        if !first { s.push(','); }
        first = false;
        s.push_str(&json_string(&field.key));
        s.push(':');
        s.push_str(&format_field_value_json(&field.value));
    }
    s.push_str("}}\n");
    s
}

fn format_field_value_json(value: &FieldValue) -> String {
    match value {
        FieldValue::String(s) => json_string(s),
        FieldValue::Boolean(b) => if *b { "true".into() } else { "false".into() },
        FieldValue::Int(i) => i.to_string(),
        FieldValue::Uint(u) => u.to_string(),
        FieldValue::Float(f) => format!("{}", f),
    }
}

fn json_string(value: &str) -> String {
    crate::bridge::json_string(value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn level_and_kind_names() {
        assert_eq!("trace", Level::Trace.name());
        assert_eq!("err", Level::Error.name());
        assert_eq!("span_begin", Kind::SpanBegin.name());
        assert_eq!("frame", Kind::Frame.name());
    }

    #[test]
    fn duration_constructors() {
        assert_eq!(1, Duration::from_nanoseconds(1).ns);
        assert_eq!(1_000, Duration::from_microseconds(1).ns);
        assert_eq!(1_000_000, Duration::from_milliseconds(1).ns);
        assert_eq!(1_000_000_000, Duration::from_seconds(1).ns);
        assert_eq!(15, duration_between(Timestamp::from_nanoseconds(10), Timestamp::from_nanoseconds(25)).ns);
        assert_eq!(0, duration_between(Timestamp::from_nanoseconds(25), Timestamp::from_nanoseconds(10)).ns);
    }

    #[test]
    fn field_constructors() {
        let fields = vec![
            string_field("phase", "layout"),
            boolean_field("dirty", true),
            int_field("delta", -3),
            uint_field("count", 42),
            float_field("ratio", 0.5),
        ];
        assert_eq!("phase", fields[0].key);
        assert_eq!(&FieldValue::String("layout".into()), &fields[0].value);
        assert_eq!(&FieldValue::Boolean(true), &fields[1].value);
        assert_eq!(&FieldValue::Int(-3), &fields[2].value);
        assert_eq!(&FieldValue::Uint(42), &fields[3].value);
        if let FieldValue::Float(f) = &fields[4].value {
            assert!((f - 0.5).abs() < 0.0001);
        } else {
            panic!("expected Float");
        }
    }

    #[test]
    fn record_constructors() {
        let fields = vec![string_field("route", "/")];
        let record = event_record(Timestamp::from_nanoseconds(100), Level::Info, "request", Some("ok"), fields);
        assert_eq!(Kind::Event, record.kind);
        assert_eq!("request", record.name);
        assert_eq!(Some("ok".to_string()), record.message);

        let begin = span_begin_record(Timestamp::from_nanoseconds(10), Level::Debug, 7, Some(3), "render", vec![]);
        assert_eq!(Kind::SpanBegin, begin.kind);
        assert_eq!(Some(7), begin.span_id);

        let end = span_end_record(
            Timestamp::from_nanoseconds(25), Level::Debug, 7, Some(3), "render", Some("done"),
            Duration::from_nanoseconds(15), vec![],
        );
        assert_eq!(Kind::SpanEnd, end.kind);
        assert_eq!(Some(Duration::from_nanoseconds(15)), end.duration);
    }

    #[test]
    fn buffer_sink_stores_records() {
        let mut sink = BufferSink::new(2);
        sink.write(event_record(Timestamp::from_nanoseconds(1), Level::Info, "one", None, vec![]));
        sink.write(event_record(Timestamp::from_nanoseconds(2), Level::Warn, "two", None, vec![]));
        // third write is silently dropped (capacity=2)
        sink.write(event_record(Timestamp::from_nanoseconds(3), Level::Error, "three", None, vec![]));
        assert_eq!(2, sink.written().len());
        assert_eq!("one", sink.written()[0].name);
        assert_eq!("two", sink.written()[1].name);
    }

    #[test]
    fn text_formatting_includes_metadata_and_fields() {
        let record = Record {
            timestamp: Timestamp::from_nanoseconds(123),
            level: Level::Debug,
            kind: Kind::SpanEnd,
            name: "render".into(),
            message: Some("done".into()),
            fields: vec![string_field("phase", "draw"), uint_field("items", 3)],
            span_id: Some(9),
            parent_span_id: Some(1),
            duration: Some(Duration::from_nanoseconds(456)),
        };
        let text = format_text(&record);
        assert!(text.contains("ts=123"));
        assert!(text.contains("level=debug"));
        assert!(text.contains("kind=span_end"));
        assert!(text.contains("name=\"render\""));
        assert!(text.contains("message=\"done\""));
        assert!(text.contains("span_id=9"));
        assert!(text.contains("parent_span_id=1"));
        assert!(text.contains("duration_ns=456"));
        assert!(text.contains("phase=\"draw\""));
        assert!(text.contains("items=3"));
    }

    #[test]
    fn json_line_formatting_escapes_strings() {
        let fields = vec![
            string_field("quote", "a\"b"),
            string_field("path", "a\\b"),
            string_field("line", "a\nb"),
            boolean_field("ok", true),
        ];
        let record = event_record(
            Timestamp::from_nanoseconds(5), Level::Info,
            "cli\nrun", Some("hi \"there\""), fields,
        );
        let json = format_json_line(&record);
        assert!(json.starts_with("{\"timestamp_ns\":5"));
        assert!(json.contains("\"name\":\"cli\\nrun\""));
        assert!(json.contains("\"message\":\"hi \\\"there\\\"\""));
        assert!(json.contains("\"quote\":\"a\\\"b\""));
        assert!(json.contains("\"path\":\"a\\\\b\""));
        assert!(json.contains("\"line\":\"a\\nb\""));
        assert!(json.contains("\"ok\":true"));
    }

    #[test]
    fn counter_gauge_frame_constructors() {
        let counter = counter_record(Timestamp::from_nanoseconds(1), "requests", 12, vec![]);
        let json = format_json_line(&counter);
        assert!(json.contains("\"kind\":\"counter\""));
        assert!(json.contains("\"value\":12"));

        let gauge = gauge_record(Timestamp::from_nanoseconds(2), "load", 0.75, vec![]);
        let json = format_json_line(&gauge);
        assert!(json.contains("\"kind\":\"gauge\""));
        assert!(json.contains("\"value\":0.75"));

        let frame = frame_record(
            Timestamp::from_nanoseconds(3), "main",
            Duration::from_nanoseconds(16_000_000), 4, vec![],
        );
        let json = format_json_line(&frame);
        assert!(json.contains("\"kind\":\"frame\""));
        assert!(json.contains("\"duration_ns\":16000000"));
        assert!(json.contains("\"index\":4"));
    }

    #[test]
    fn fanout_sink_writes_every_child() {
        let mut sink_a = BufferSink::new(2);
        let mut sink_b = BufferSink::new(2);
        {
            let mut fanout = FanoutSink::new(vec![&mut sink_a, &mut sink_b]);
            fanout.write(event_record(Timestamp::from_nanoseconds(1), Level::Info, "one", None, vec![]));
        }
        assert_eq!(1, sink_a.written().len());
        assert_eq!(1, sink_b.written().len());
    }
}
