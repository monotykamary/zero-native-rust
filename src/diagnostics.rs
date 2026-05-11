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
