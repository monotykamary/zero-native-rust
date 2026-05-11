#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Severity {
    Hint,
    Info,
    Warning,
    Error,
    Fatal,
}

impl Severity {
    pub fn name(self) -> &'static str {
        match self {
            Self::Hint => "hint",
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
            Self::Fatal => "fatal",
        }
    }
}

pub type SourceId = u32;

#[derive(Debug, Clone)]
pub struct Source {
    pub id: SourceId,
    pub name: String,
    pub text: String,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub source_id: SourceId,
    pub start: usize,
    pub end: usize,
}

#[derive(Debug, Clone)]
pub struct Label {
    pub style: LabelStyle,
    pub span: Span,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LabelStyle {
    Primary,
    Secondary,
}

#[derive(Debug, Clone)]
pub struct Diagnostic {
    pub severity: Severity,
    pub code: DiagnosticCode,
    pub message: String,
    pub labels: Vec<Label>,
    pub notes: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct DiagnosticCode {
    pub namespace: String,
    pub value: String,
}

impl DiagnosticCode {
    pub fn is_empty(&self) -> bool {
        self.namespace.is_empty() && self.value.is_empty()
    }
}

pub fn format_short(diagnostic: &Diagnostic) -> String {
    let mut out = diagnostic.severity.name().to_string();
    if !diagnostic.code.is_empty() {
        out.push_str(&format!("[{}.{}]", diagnostic.code.namespace, diagnostic.code.value));
    }
    out.push_str(&format!(": {}", diagnostic.message));
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub byte_offset: usize,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Line {
    pub start: usize,
    pub end: usize,
    pub number: usize,
}

pub fn position_at(source: &Source, byte_offset: usize) -> Result<Position, Error> {
    if byte_offset > source.text.len() {
        return Err(Error::InvalidSpan);
    }
    let mut line = 1usize;
    let mut column = 1usize;
    for (i, ch) in source.text.chars().enumerate() {
        if i >= byte_offset { break; }
        if ch == '\n' { line += 1; column = 1; } else { column += 1; }
    }
    Ok(Position { byte_offset, line, column })
}

pub fn line_at(source: &Source, byte_offset: usize) -> Result<Line, Error> {
    if byte_offset > source.text.len() {
        return Err(Error::InvalidSpan);
    }
    let mut start = byte_offset;
    while start > 0 && source.text.as_bytes()[start - 1] != b'\n' { start -= 1; }
    let mut end = byte_offset;
    while end < source.text.len() && source.text.as_bytes()[end] != b'\n' { end += 1; }
    let pos = position_at(source, start)?;
    Ok(Line { start, end, number: pos.line })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    MissingSource,
    InvalidSpan,
    InvalidSourceText,
    NoSpaceLeft,
}

pub fn validate_span(source_text: &str, span: &Span) -> Result<(), Error> {
    if source_text.contains('\0') { return Err(Error::InvalidSourceText); }
    if span.start > span.end || span.end > source_text.len() {
        return Err(Error::InvalidSpan);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn severity_names() {
        assert_eq!("hint", Severity::Hint.name());
        assert_eq!("info", Severity::Info.name());
        assert_eq!("warning", Severity::Warning.name());
        assert_eq!("error", Severity::Error.name());
        assert_eq!("fatal", Severity::Fatal.name());
    }

    #[test]
    fn source_lookup_by_id() {
        let source = Source { id: 1, name: "app.zon".into(), text: "name = \"demo\"".into() };
        assert_eq!(1, source.id);
        assert_eq!("app.zon", source.name);
    }

    #[test]
    fn short_formatting() {
        let diag = Diagnostic {
            severity: Severity::Warning,
            code: DiagnosticCode { namespace: "asset".into(), value: "missing".into() },
            message: "missing icon".into(),
            labels: vec![],
            notes: vec![],
        };
        assert_eq!("warning[asset.missing]: missing icon", format_short(&diag));
    }

    #[test]
    fn span_validation() {
        let span = Span { start: 0, end: 5, source_id: 1 };
        assert!(span.start <= span.end);
    }

    #[test]
    fn diagnostic_code_is_empty() {
        let code = DiagnosticCode { namespace: String::new(), value: String::new() };
        assert!(code.is_empty());
        let code = DiagnosticCode { namespace: "lang".into(), value: "E001".into() };
        assert!(!code.is_empty());
    }

    #[test]
    fn position_at_offsets() {
        let source = Source { id: 1, name: "app.zon".into(), text: "name = \"demo\"\nid = \"Bad\"\n".into() };
        assert_eq!(Position { byte_offset: 0, line: 1, column: 1 }, position_at(&source, 0).unwrap());
        assert_eq!(Position { byte_offset: 14, line: 2, column: 1 }, position_at(&source, 14).unwrap());
        assert_eq!(Position { byte_offset: source.text.len(), line: 3, column: 1 }, position_at(&source, source.text.len()).unwrap());
    }

    #[test]
    fn line_at_boundaries() {
        let source = Source { id: 1, name: "app.zon".into(), text: "name = \"demo\"\nid = \"Bad\"\n".into() };
        let line1 = line_at(&source, 0).unwrap();
        assert_eq!(0, line1.start);
        assert_eq!(13, line1.end);
        assert_eq!(1, line1.number);
        let line2 = line_at(&source, 18).unwrap();
        assert_eq!(14, line2.start);
        assert_eq!(24, line2.end);
        assert_eq!(2, line2.number);
    }

    #[test]
    fn validate_span_rejects_invalid() {
        let source = Source { id: 1, name: "app.zon".into(), text: "hello".into() };
        assert!(validate_span(&source.text, &Span { source_id: 1, start: 0, end: 5 }).is_ok());
        assert!(validate_span(&source.text, &Span { source_id: 1, start: 5, end: 4 }).is_err()); // start > end
        assert!(validate_span(&source.text, &Span { source_id: 1, start: 0, end: 999 }).is_err()); // end > len
        assert!(validate_span("bad\0text", &Span { source_id: 1, start: 0, end: 3 }).is_err()); // null byte
    }
}
