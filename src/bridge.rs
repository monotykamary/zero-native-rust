use crate::security;

pub const MAX_MESSAGE_BYTES: usize = 1024 * 1024;
pub const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
pub const MAX_RESULT_BYTES: usize = 1024 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorCode {
    InvalidRequest,
    UnknownCommand,
    PermissionDenied,
    HandlerFailed,
    PayloadTooLarge,
    InternalError,
}

impl ErrorCode {
    pub fn json_name(self) -> &'static str {
        match self {
            Self::InvalidRequest => "invalid_request",
            Self::UnknownCommand => "unknown_command",
            Self::PermissionDenied => "permission_denied",
            Self::HandlerFailed => "handler_failed",
            Self::PayloadTooLarge => "payload_too_large",
            Self::InternalError => "internal_error",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Source {
    pub origin: String,
    pub window_id: u64,
}

impl Default for Source {
    fn default() -> Self {
        Self {
            origin: String::new(),
            window_id: 1,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub id: String,
    pub command: String,
    pub payload: String,
}

impl Request {
    pub fn parse(raw: &str) -> Result<Self, ParseError> {
        if raw.len() > MAX_MESSAGE_BYTES {
            return Err(ParseError::PayloadTooLarge);
        }
        let trimmed = raw.trim();
        if !trimmed.starts_with('{') || !trimmed.ends_with('}') {
            return Err(ParseError::InvalidRequest);
        }

        let mut id: Option<String> = None;
        let mut command: Option<String> = None;
        let mut payload = "null".to_string();

        // Simple recursive-descent JSON object parser
        let bytes = trimmed.as_bytes();
        let mut pos = 1; // skip opening {
        pos = skip_ws(bytes, pos);

        if pos < bytes.len() && bytes[pos] == b'}' {
            return Err(ParseError::InvalidRequest);
        }

        loop {
            pos = skip_ws(bytes, pos);
            if pos >= bytes.len() { return Err(ParseError::InvalidRequest); }
            let key = parse_json_string(bytes, &mut pos)?;
            pos = skip_ws(bytes, pos);
            if pos >= bytes.len() || bytes[pos] != b':' { return Err(ParseError::InvalidRequest); }
            pos += 1;
            pos = skip_ws(bytes, pos);
            if pos >= bytes.len() { return Err(ParseError::InvalidRequest); }

            let key_str = &trimmed[key.start..key.end];
            match key_str {
                "id" => {
                    let val = parse_json_string(bytes, &mut pos)?;
                    id = Some(trimmed[val.start..val.end].to_string());
                }
                "command" => {
                    let val = parse_json_string(bytes, &mut pos)?;
                    command = Some(trimmed[val.start..val.end].to_string());
                }
                "payload" => {
                    let start = pos;
                    skip_json_value(bytes, &mut pos)?;
                    payload = trimmed[start..pos].to_string();
                }
                _ => {
                    skip_json_value(bytes, &mut pos)?;
                }
            }

            pos = skip_ws(bytes, pos);
            if pos >= bytes.len() { return Err(ParseError::InvalidRequest); }
            if bytes[pos] == b',' { pos += 1; continue; }
            if bytes[pos] == b'}' { break; }
            return Err(ParseError::InvalidRequest);
        }

        let id = id.ok_or(ParseError::InvalidRequest)?;
        let command = command.ok_or(ParseError::InvalidRequest)?;
        if id.is_empty() || command.is_empty() {
            return Err(ParseError::InvalidRequest);
        }
        Ok(Self { id, command, payload })
    }
}

struct Span { start: usize, end: usize }

fn skip_ws(bytes: &[u8], mut pos: usize) -> usize {
    while pos < bytes.len() && matches!(bytes[pos], b' ' | b'\n' | b'\r' | b'\t') {
        pos += 1;
    }
    pos
}

fn parse_json_string(bytes: &[u8], pos: &mut usize) -> Result<Span, ParseError> {
    if *pos >= bytes.len() || bytes[*pos] != b'"' { return Err(ParseError::InvalidRequest); }
    let start = *pos + 1;
    *pos += 1;
    while *pos < bytes.len() {
        match bytes[*pos] {
            b'"' => {
                let end = *pos;
                *pos += 1;
                return Ok(Span { start, end });
            }
            b'\\' => { *pos += 2; }
            b if b <= 0x1f => { return Err(ParseError::InvalidRequest); }
            _ => { *pos += 1; }
        }
    }
    Err(ParseError::InvalidRequest)
}

fn skip_json_value(bytes: &[u8], pos: &mut usize) -> Result<(), ParseError> {
    if *pos >= bytes.len() { return Err(ParseError::InvalidRequest); }
    match bytes[*pos] {
        b'"' => { parse_json_string(bytes, pos)?; }
        b'{' => skip_container(bytes, pos, b'{', b'}')?,
        b'[' => skip_container(bytes, pos, b'[', b']')?,
        _ => {
            let start = *pos;
            while *pos < bytes.len() && !matches!(bytes[*pos], b',' | b'}' | b']' | b' ' | b'\n' | b'\r' | b'\t') {
                *pos += 1;
            }
            if *pos == start { return Err(ParseError::InvalidRequest); }
        }
    }
    Ok(())
}

fn skip_container(bytes: &[u8], pos: &mut usize, open: u8, close: u8) -> Result<(), ParseError> {
    if bytes[*pos] != open { return Err(ParseError::InvalidRequest); }
    *pos += 1;
    *pos = skip_ws(bytes, *pos);
    if *pos < bytes.len() && bytes[*pos] == close { *pos += 1; return Ok(()); }
    loop {
        *pos = skip_ws(bytes, *pos);
        if open == b'{' {
            parse_json_string(bytes, pos)?;
            *pos = skip_ws(bytes, *pos);
            if *pos >= bytes.len() || bytes[*pos] != b':' { return Err(ParseError::InvalidRequest); }
            *pos += 1;
            *pos = skip_ws(bytes, *pos);
        }
        skip_json_value(bytes, pos)?;
        *pos = skip_ws(bytes, *pos);
        if *pos >= bytes.len() { return Err(ParseError::InvalidRequest); }
        if bytes[*pos] == b',' { *pos += 1; continue; }
        if bytes[*pos] == close { *pos += 1; return Ok(()); }
        return Err(ParseError::InvalidRequest);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    InvalidRequest,
    PayloadTooLarge,
}

#[derive(Debug, Clone)]
pub struct CommandPolicy {
    pub name: String,
    pub permissions: Vec<String>,
    pub origins: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct Policy {
    pub enabled: bool,
    pub permissions: Vec<String>,
    pub commands: Vec<CommandPolicy>,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            enabled: false,
            permissions: Vec::new(),
            commands: Vec::new(),
        }
    }
}

impl Policy {
    pub fn allows(&self, command: &str, origin: &str) -> bool {
        if !self.enabled { return false; }
        let cmd_policy = match self.commands.iter().find(|c| c.name == command) {
            Some(c) => c,
            None => return false,
        };
        if !security::has_permissions(&self.permissions, &cmd_policy.permissions.iter().map(|s| s.as_str()).collect::<Vec<_>>()) {
            return false;
        }
        if cmd_policy.origins.is_empty() { return true; }
        cmd_policy.origins.iter().any(|o| o == "*" || o == origin)
    }
}

pub struct Handler {
    pub name: String,
}

pub struct Registry {
    pub handlers: Vec<Handler>,
}

impl Registry {
    pub fn find(&self, command: &str) -> Option<&Handler> {
        self.handlers.iter().find(|h| h.name == command)
    }
}

pub struct Invocation {
    pub request: Request,
    pub source: Source,
}

pub struct Dispatcher {
    pub policy: Policy,
    pub registry: Registry,
}

impl Dispatcher {
    pub fn dispatch(&self, raw: &str, source: Source, output: &mut [u8]) -> usize {
        if raw.len() > MAX_MESSAGE_BYTES {
            return write_error_response(output, "", ErrorCode::PayloadTooLarge, "Bridge request is too large");
        }
        let request = match Request::parse(raw) {
            Ok(r) => r,
            Err(_) => return write_error_response(output, "", ErrorCode::InvalidRequest, "Bridge request is malformed"),
        };
        if !self.policy.allows(&request.command, &source.origin) {
            return write_error_response(output, &request.id, ErrorCode::PermissionDenied, "Bridge command is not permitted");
        }
        let _handler = match self.registry.find(&request.command) {
            Some(h) => h,
            None => return write_error_response(output, &request.id, ErrorCode::UnknownCommand, "Bridge command is not registered"),
        };
        write_error_response(output, &request.id, ErrorCode::HandlerFailed, "not implemented")
    }
}

pub fn write_success_response(output: &mut [u8], id: &str, result: &str) -> usize {
    let msg = format!(
        "{{\"id\":{},\"ok\":true,\"result\":{}}}",
        json_string(id),
        result
    );
    let len = msg.len().min(output.len());
    output[..len].copy_from_slice(&msg.as_bytes()[..len]);
    len
}

pub fn write_error_response(output: &mut [u8], id: &str, code: ErrorCode, message: &str) -> usize {
    let msg = format!(
        "{{\"id\":{},\"ok\":false,\"error\":{{\"code\":{},\"message\":{}}}}}",
        json_string(id),
        json_string(code.json_name()),
        json_string(message)
    );
    let len = msg.len().min(output.len());
    output[..len].copy_from_slice(&msg.as_bytes()[..len]);
    len
}

pub fn json_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) <= 0x1f => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_request() {
        let req = Request::parse(
            r#"{"id":"1","command":"native.ping","payload":{"text":"hello","count":2}}"#,
        ).unwrap();
        assert_eq!(req.id, "1");
        assert_eq!(req.command, "native.ping");
    }

    #[test]
    fn reject_malformed() {
        assert!(Request::parse("{}").is_err());
        assert!(Request::parse(r#"{"id":"","command":"native.ping"}"#).is_err());
    }

    #[test]
    fn success_response() {
        let mut buf = [0u8; 256];
        let len = write_success_response(&mut buf, "abc", r#"{"pong":true}"#);
        let s = std::str::from_utf8(&buf[..len]).unwrap();
        assert_eq!(s, r#"{"id":"abc","ok":true,"result":{"pong":true}}"#);
    }

    #[test]
    fn error_response() {
        let mut buf = [0u8; 256];
        let len = write_error_response(&mut buf, "abc", ErrorCode::PermissionDenied, "Denied");
        let s = std::str::from_utf8(&buf[..len]).unwrap();
        assert!(s.contains("\"permission_denied\""));
    }

    #[test]
    fn policy_allows_matching_origin() {
        let policy = Policy {
            enabled: true,
            permissions: vec![],
            commands: vec![CommandPolicy {
                name: "native.ping".into(),
                permissions: vec![],
                origins: vec!["zero://inline".into()],
            }],
        };
        assert!(policy.allows("native.ping", "zero://inline"));
        assert!(!policy.allows("native.ping", "https://example.com"));
    }
}
