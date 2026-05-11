use crate::trace;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TraceMode {
    Events,
    Runtime,
    All,
}

impl TraceMode {
    pub fn includes(self, other: Self) -> bool {
        matches!((self, other), (TraceMode::All, _) | (_, TraceMode::Events))
    }
}

pub fn parse_trace_mode(raw: &str) -> Option<TraceMode> {
    match raw {
        "events" => Some(TraceMode::Events),
        "runtime" => Some(TraceMode::Runtime),
        "all" => Some(TraceMode::All),
        _ => None,
    }
}

pub fn diagnose(_sink: &mut dyn trace::Sink) {
    // Debug diagnostics would walk the runtime state
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn trace_mode_parsing_and_matching() {
        assert_eq!(Some(TraceMode::Events), parse_trace_mode("events"));
        assert!(TraceMode::All.includes(TraceMode::Runtime));
        assert!(!TraceMode::Events.includes(TraceMode::Runtime));
    }
}
