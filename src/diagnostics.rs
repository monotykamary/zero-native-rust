#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity { Hint, Info, Warning, Error, Fatal }

impl Severity {
    pub fn name(self) -> &'static str {
        match self { Self::Hint => "hint", Self::Info => "info", Self::Warning => "warning", Self::Error => "error", Self::Fatal => "fatal" }
    }
}

pub type SourceId = u32;

#[derive(Debug, Clone)]
pub struct Source { pub id: SourceId, pub name: String, pub text: String }

#[derive(Debug, Clone, Copy)]
pub struct Span { pub source_id: SourceId, pub start: usize, pub end: usize }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelStyle { Primary, Secondary }

#[derive(Debug, Clone)]
pub struct Label { pub style: LabelStyle, pub span: Span, pub message: String }

#[derive(Debug, Clone)]
pub struct Note { pub message: String }

#[derive(Debug, Clone)]
pub struct Suggestion { pub message: String, pub replacement: String, pub span: Option<Span> }

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: DiagnosticCode,
    pub message: String,
    pub labels: Vec<Label>,
    pub notes: Vec<Note>,
    pub suggestions: Vec<Suggestion>,
}

#[derive(Debug, Clone)]
pub struct DiagnosticCode { pub namespace: String, pub value: String }

impl DiagnosticCode {
    pub fn is_empty(&self) -> bool { self.namespace.is_empty() && self.value.is_empty() }
}

pub struct SourceMap { pub sources: Vec<Source> }

impl SourceMap {
    pub fn find(&self, id: SourceId) -> Option<&Source> { self.sources.iter().find(|s| s.id == id) }
}

pub fn code(namespace: &str, value: &str) -> DiagnosticCode {
    DiagnosticCode { namespace: namespace.to_string(), value: value.to_string() }
}

pub fn primary(span: Span, message: &str) -> Label {
    Label { style: LabelStyle::Primary, span, message: message.to_string() }
}

pub fn secondary(span: Span, message: &str) -> Label {
    Label { style: LabelStyle::Secondary, span, message: message.to_string() }
}

pub fn note(message: &str) -> Note { Note { message: message.to_string() } }

pub fn suggestion(message: &str, replacement: &str, span: Option<Span>) -> Suggestion {
    Suggestion { message: message.to_string(), replacement: replacement.to_string(), span }
}

pub fn format_short(diagnostic: &Diagnostic) -> String {
    let mut out = diagnostic.severity.name().to_string();
    if !diagnostic.code.is_empty() { out.push_str(&format!("[{}.{}]", diagnostic.code.namespace, diagnostic.code.value)); }
    out.push_str(&format!(": {}", diagnostic.message));
    out
}

pub fn format_text(source_map: &SourceMap, diagnostic: &Diagnostic) -> String {
    let mut out = format_short(diagnostic);
    for label in &diagnostic.labels {
        if let Some(source) = source_map.find(label.span.source_id) {
            let pos = position_at(source, label.span.start).unwrap_or(Position { byte_offset: 0, line: 0, column: 0 });
            out.push_str(&format!("\n  --> {}:{}:{}", source.name, pos.line, pos.column));
            if !label.message.is_empty() { out.push_str(&format!(": {}", label.message)); }
        }
    }
    for note in &diagnostic.notes { out.push_str(&format!("\n  note: {}", note.message)); }
    for sug in &diagnostic.suggestions { out.push_str(&format!("\n  help: {} -> `{}`", sug.message, sug.replacement)); }
    out
}

pub fn format_json_line(diagnostic: &Diagnostic) -> String {
    let severity = diagnostic.severity.name();
    let code = if diagnostic.code.is_empty() { "null".to_string() } else { format!("\"{}.{}\"", diagnostic.code.namespace, diagnostic.code.value) };
    let message = crate::json::write_json_string(&diagnostic.message);
    format!("{{\"severity\":\"{}\",\"code\":{},\"message\":{}}}", severity, code, message)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position { pub byte_offset: usize, pub line: usize, pub column: usize }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line { pub start: usize, pub end: usize, pub number: usize }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error { MissingSource, InvalidSpan, InvalidSourceText, NoSpaceLeft }

pub fn position_at(source: &Source, byte_offset: usize) -> Result<Position, Error> {
    if byte_offset > source.text.len() { return Err(Error::InvalidSpan); }
    let mut line = 1usize;
    let mut column = 1usize;
    for (i, ch) in source.text.chars().enumerate() {
        if i >= byte_offset { break; }
        if ch == '\n' { line += 1; column = 1; } else { column += 1; }
    }
    Ok(Position { byte_offset, line, column })
}

pub fn line_at(source: &Source, byte_offset: usize) -> Result<Line, Error> {
    if byte_offset > source.text.len() { return Err(Error::InvalidSpan); }
    let mut start = byte_offset;
    while start > 0 && source.text.as_bytes()[start - 1] != b'\n' { start -= 1; }
    let mut end = byte_offset;
    while end < source.text.len() && source.text.as_bytes()[end] != b'\n' { end += 1; }
    let pos = position_at(source, start)?;
    Ok(Line { start, end, number: pos.line })
}

pub fn validate_span(source_text: &str, span: &Span) -> Result<(), Error> {
    if source_text.contains('\0') { return Err(Error::InvalidSourceText); }
    if span.start > span.end || span.end > source_text.len() { return Err(Error::InvalidSpan); }
    Ok(())
}

pub fn validate_diagnostic(source_map: &SourceMap, diagnostic: &Diagnostic) -> Result<(), Error> {
    for label in &diagnostic.labels {
        let source = source_map.find(label.span.source_id).ok_or(Error::MissingSource)?;
        validate_span(&source.text, &label.span)?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_names() {
        assert_eq!("hint", Severity::Hint.name());
        assert_eq!("error", Severity::Error.name());
    }

    #[test]
    fn short_formatting() {
        let diag = Diagnostic {
            severity: Severity::Warning,
            code: DiagnosticCode { namespace: "asset".into(), value: "missing".into() },
            message: "missing icon".into(), labels: vec![], notes: vec![], suggestions: vec![],
        };
        assert_eq!("warning[asset.missing]: missing icon", format_short(&diag));
    }

    #[test]
    fn text_formatting_with_labels() {
        let source = Source { id: 1, name: "app.zon".into(), text: "name = \"demo\"".into() };
        let diag = Diagnostic {
            severity: Severity::Error, code: code("lang", "E001"),
            message: "bad syntax".into(),
            labels: vec![primary(Span { source_id: 1, start: 0, end: 4 }, "here")],
            notes: vec![note("try quotes")], suggestions: vec![suggestion("use quotes", "\"value\"", None)],
        };
        let map = SourceMap { sources: vec![source] };
        let text = format_text(&map, &diag);
        assert!(text.contains("bad syntax"));
        assert!(text.contains("note:"));
        assert!(text.contains("help:"));
    }

    #[test]
    fn json_line_formatting() {
        let diag = Diagnostic { severity: Severity::Error, code: code("lang", "E001"), message: "bad".into(), labels: vec![], notes: vec![], suggestions: vec![] };
        let json = format_json_line(&diag);
        assert!(json.contains("\"severity\":\"error\""));
    }

    #[test]
    fn diagnostic_code_is_empty() {
        assert!(DiagnosticCode { namespace: String::new(), value: String::new() }.is_empty());
        assert!(!DiagnosticCode { namespace: "lang".into(), value: "E001".into() }.is_empty());
    }

    #[test]
    fn position_and_line() {
        let source = Source { id: 1, name: "app.zon".into(), text: "name = \"demo\"\nid = \"Bad\"\n".into() };
        assert_eq!(Position { byte_offset: 0, line: 1, column: 1 }, position_at(&source, 0).unwrap());
        assert_eq!(Position { byte_offset: 14, line: 2, column: 1 }, position_at(&source, 14).unwrap());
    }

    #[test]
    fn validate_diagnostic_ok() {
        let source = Source { id: 1, name: "app.zig".into(), text: "hello".into() };
        let diag = Diagnostic { severity: Severity::Error, code: code("test", "E1"), message: "test".into(), labels: vec![primary(Span { source_id: 1, start: 0, end: 5 }, "here")], notes: vec![], suggestions: vec![] };
        let map = SourceMap { sources: vec![source] };
        assert!(validate_diagnostic(&map, &diag).is_ok());
    }

    #[test]
    fn validate_diagnostic_missing_source() {
        let diag = Diagnostic { severity: Severity::Error, code: code("test", "E1"), message: "test".into(), labels: vec![primary(Span { source_id: 99, start: 0, end: 5 }, "here")], notes: vec![], suggestions: vec![] };
        let map = SourceMap { sources: vec![] };
        assert!(validate_diagnostic(&map, &diag).is_err());
    }

    #[test]
    fn suggestion_and_note_constructors() {
        let n = note("try this");
        assert_eq!("try this", n.message);
        let s = suggestion("fix", "\"value\"", None);
        assert_eq!("fix", s.message);
        assert_eq!("\"value\"", s.replacement);
    }
}
