use crate::security;

pub const MAX_MESSAGE_BYTES: usize = 1024 * 1024;
pub const MAX_RESPONSE_BYTES: usize = 1024 * 1024;
pub const MAX_RESULT_BYTES: usize = 1024 * 1024;
pub const MAX_ID_BYTES: usize = 64;
pub const MAX_COMMAND_BYTES: usize = 128;

const NULL_JSON: &str = "null";

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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    InvalidRequest,
    PayloadTooLarge,
}

#[derive(Debug, Clone, Default)]
pub struct Source {
    pub origin: String,
    pub window_id: u64,
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
        let bytes = raw.as_bytes();
        let mut pos = 0usize;
        skip_ws(bytes, &mut pos);
        expect_byte(bytes, &mut pos, b'{')?;

        let mut id: Option<String> = None;
        let mut command: Option<String> = None;
        let mut payload = NULL_JSON.to_string();

        skip_ws(bytes, &mut pos);
        if pos < bytes.len() && bytes[pos] == b'}' {
            return Err(ParseError::InvalidRequest);
        }

        loop {
            skip_ws(bytes, &mut pos);
            let key = parse_simple_string(raw, bytes, &mut pos)?;
            skip_ws(bytes, &mut pos);
            expect_byte(bytes, &mut pos, b':')?;
            skip_ws(bytes, &mut pos);

            match key {
                "id" => {
                    let val = parse_simple_string(raw, bytes, &mut pos)?;
                    id = Some(val.to_string());
                }
                "command" => {
                    let val = parse_simple_string(raw, bytes, &mut pos)?;
                    command = Some(val.to_string());
                }
                "payload" => {
                    let start = pos;
                    skip_json_value(bytes, &mut pos)?;
                    payload = raw[start..pos].to_string();
                }
                _ => {
                    skip_json_value(bytes, &mut pos)?;
                }
            }

            skip_ws(bytes, &mut pos);
            if pos >= bytes.len() { return Err(ParseError::InvalidRequest); }
            if bytes[pos] == b',' { pos += 1; continue; }
            if bytes[pos] == b'}' { pos += 1; break; }
            return Err(ParseError::InvalidRequest);
        }

        skip_ws(bytes, &mut pos);
        if pos != bytes.len() { return Err(ParseError::InvalidRequest); }

        let id = id.ok_or(ParseError::InvalidRequest)?;
        let command = command.ok_or(ParseError::InvalidRequest)?;
        if !valid_id(&id) || !valid_command(&command) {
            return Err(ParseError::InvalidRequest);
        }
        Ok(Self { id, command, payload })
    }
}

fn valid_id(value: &str) -> bool {
    if value.is_empty() || value.len() > MAX_ID_BYTES { return false; }
    !value.chars().any(|c| (c as u32) <= 0x1f || c == '"' || c == '\\')
}

fn valid_command(value: &str) -> bool {
    if value.is_empty() || value.len() > MAX_COMMAND_BYTES { return false; }
    !value.chars().any(|c| (c as u32) <= 0x1f || c == '"' || c == '\\' || c == '/' || c == ' ')
}

fn skip_ws(bytes: &[u8], pos: &mut usize) {
    while *pos < bytes.len() && matches!(bytes[*pos], b' ' | b'\n' | b'\r' | b'\t') {
        *pos += 1;
    }
}

fn expect_byte(bytes: &[u8], pos: &mut usize, expected: u8) -> Result<(), ParseError> {
    if *pos >= bytes.len() || bytes[*pos] != expected {
        return Err(ParseError::InvalidRequest);
    }
    *pos += 1;
    Ok(())
}

fn parse_simple_string<'a>(raw: &'a str, bytes: &[u8], pos: &mut usize) -> Result<&'a str, ParseError> {
    expect_byte(bytes, pos, b'"')?;
    let start = *pos;
    while *pos < bytes.len() {
        match bytes[*pos] {
            b'"' => {
                let end = *pos;
                *pos += 1;
                return Ok(&raw[start..end]);
            }
            b'\\' => return Err(ParseError::InvalidRequest),
            b if b <= 0x1f => return Err(ParseError::InvalidRequest),
            _ => { *pos += 1; }
        }
    }
    Err(ParseError::InvalidRequest)
}

fn skip_json_value(bytes: &[u8], pos: &mut usize) -> Result<(), ParseError> {
    if *pos >= bytes.len() { return Err(ParseError::InvalidRequest); }
    match bytes[*pos] {
        b'"' => skip_json_string(bytes, pos)?,
        b'{' => skip_json_container(bytes, pos, b'{', b'}')?,
        b'[' => skip_json_container(bytes, pos, b'[', b']')?,
        _ => skip_json_atom(bytes, pos)?,
    }
    Ok(())
}

fn skip_json_string(bytes: &[u8], pos: &mut usize) -> Result<(), ParseError> {
    expect_byte(bytes, pos, b'"')?;
    while *pos < bytes.len() {
        match bytes[*pos] {
            b'"' => { *pos += 1; return Ok(()); }
            b'\\' => { *pos += 1; if *pos >= bytes.len() { return Err(ParseError::InvalidRequest); } *pos += 1; }
            b if b <= 0x1f => { return Err(ParseError::InvalidRequest); }
            _ => { *pos += 1; }
        }
    }
    Err(ParseError::InvalidRequest)
}

fn skip_json_container(bytes: &[u8], pos: &mut usize, open: u8, close: u8) -> Result<(), ParseError> {
    expect_byte(bytes, pos, open)?;
    skip_ws(bytes, pos);
    if *pos < bytes.len() && bytes[*pos] == close { *pos += 1; return Ok(()); }
    loop {
        skip_ws(bytes, pos);
        if open == b'{' {
            skip_json_string(bytes, pos)?;
            skip_ws(bytes, pos);
            expect_byte(bytes, pos, b':')?;
            skip_ws(bytes, pos);
        }
        skip_json_value(bytes, pos)?;
        skip_ws(bytes, pos);
        if *pos >= bytes.len() { return Err(ParseError::InvalidRequest); }
        if bytes[*pos] == b',' { *pos += 1; continue; }
        if bytes[*pos] == close { *pos += 1; return Ok(()); }
        return Err(ParseError::InvalidRequest);
    }
}

fn skip_json_atom(bytes: &[u8], pos: &mut usize) -> Result<(), ParseError> {
    let start = *pos;
    while *pos < bytes.len() && !matches!(bytes[*pos], b',' | b'}' | b']' | b' ' | b'\n' | b'\r' | b'\t') {
        *pos += 1;
    }
    if *pos == start { return Err(ParseError::InvalidRequest); }
    Ok(())
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
        Self { enabled: false, permissions: Vec::new(), commands: Vec::new() }
    }
}

impl Policy {
    pub fn allows(&self, command: &str, origin: &str) -> bool {
        if !self.enabled { return false; }
        let cmd_policy = match self.commands.iter().find(|c| c.name == command) {
            Some(c) => c,
            None => return false,
        };
        let perms: Vec<&str> = self.permissions.iter().map(|s| s.as_str()).collect();
        let cmd_perms: Vec<&str> = cmd_policy.permissions.iter().map(|s| s.as_str()).collect();
        if !security::has_permissions(&perms, &cmd_perms) { return false; }
        if cmd_policy.origins.is_empty() { return true; }
        cmd_policy.origins.iter().any(|o| o == "*" || o == origin)
    }

    pub fn find(&self, command: &str) -> Option<&CommandPolicy> {
        self.commands.iter().find(|c| c.name == command)
    }
}

pub type HandlerFn = fn(context: &mut dyn std::any::Any, invocation: Invocation, output: &mut [u8]) -> Result<usize, String>;
pub type AsyncRespondFn = fn(context: &mut dyn std::any::Any, source: Source, response: &[u8]) -> Result<(), String>;
pub type AsyncHandlerFn = fn(context: &mut dyn std::any::Any, invocation: Invocation, responder: AsyncResponder) -> Result<(), String>;

#[derive(Clone)]
pub struct Handler {
    pub name: String,
    pub invoke_fn: HandlerFn,
}

#[derive(Clone)]
pub struct AsyncHandler {
    pub name: String,
    pub invoke_fn: AsyncHandlerFn,
}

#[derive(Clone)]
pub struct AsyncResponder {
    pub source: Source,
    pub respond_fn: AsyncRespondFn,
}

impl AsyncResponder {
    pub fn respond(&self, context: &mut dyn std::any::Any, response: &[u8]) -> Result<(), String> {
        (self.respond_fn)(context, self.source.clone(), response)
    }

    pub fn success(&self, context: &mut dyn std::any::Any, id: &str, result: &str) -> Result<(), String> {
        let mut buffer = vec![0u8; MAX_RESPONSE_BYTES];
        let len = write_success_response(&mut buffer, id, result);
        (self.respond_fn)(context, self.source.clone(), &buffer[..len])
    }

    pub fn fail(&self, context: &mut dyn std::any::Any, id: &str, code: ErrorCode, message: &str) -> Result<(), String> {
        let mut buffer = vec![0u8; MAX_RESPONSE_BYTES];
        let len = write_error_response(&mut buffer, id, code, message);
        (self.respond_fn)(context, self.source.clone(), &buffer[..len])
    }
}

#[derive(Clone)]
pub struct Registry {
    pub handlers: Vec<Handler>,
}

impl Default for Registry {
    fn default() -> Self { Self { handlers: Vec::new() } }
}

impl Registry {
    pub fn find(&self, command: &str) -> Option<&Handler> {
        self.handlers.iter().find(|h| h.name == command)
    }
}

#[derive(Clone, Default)]
pub struct AsyncRegistry {
    pub handlers: Vec<AsyncHandler>,
}

impl AsyncRegistry {
    pub fn find(&self, command: &str) -> Option<&AsyncHandler> {
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
    pub async_registry: AsyncRegistry,
}

impl Default for Dispatcher {
    fn default() -> Self {
        Self { policy: Policy::default(), registry: Registry::default(), async_registry: AsyncRegistry::default() }
    }
}

impl Clone for Dispatcher {
    fn clone(&self) -> Self {
        Self {
            policy: self.policy.clone(),
            registry: Registry { handlers: Vec::new() },
            async_registry: AsyncRegistry { handlers: Vec::new() },
        }
    }
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
    let value = if result.is_empty() { NULL_JSON } else { result };
    if !crate::json::is_valid_value(value) {
        return write_error_response(output, id, ErrorCode::HandlerFailed, "Bridge command returned invalid JSON");
    }
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(output);
    let _ = write!(cursor, "{{\"id\":{},\"ok\":true,\"result\":{}}}", write_json_string(id), value);
    cursor.position() as usize
}

pub fn write_error_response(output: &mut [u8], id: &str, code: ErrorCode, message: &str) -> usize {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(output);
    let _ = write!(cursor, "{{\"id\":{},\"ok\":false,\"error\":{{\"code\":{},\"message\":{}}}}}", write_json_string(id), write_json_string(code.json_name()), write_json_string(message));
    cursor.position() as usize
}

pub fn write_json_string_value(output: &mut [u8], value: &str) -> usize {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(output);
    let _ = write!(cursor, "{}", write_json_string(value));
    cursor.position() as usize
}

pub fn is_valid_json_value(raw: &str) -> bool {
    crate::json::is_valid_value(raw)
}

pub fn write_json_string(value: &str) -> String {
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
    fn parse_request_envelope() {
        let req = Request::parse(
            r#"{"id":"1","command":"native.ping","payload":{"text":"hello","count":2}}"#,
        ).unwrap();
        assert_eq!("1", req.id);
        assert_eq!("native.ping", req.command);
        assert_eq!(r#"{"text":"hello","count":2}"#, req.payload);
    }

    #[test]
    fn reject_malformed_or_oversized() {
        assert!(Request::parse("{}").is_err());
        assert!(Request::parse(r#"{"id":"","command":"native.ping"}"#).is_err());
        assert!(Request::parse(r#"{"id":"1","command":"bad command"}"#).is_err());
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
        assert_eq!(s, r#"{"id":"abc","ok":false,"error":{"code":"permission_denied","message":"Denied"}}"#);
    }

    #[test]
    fn validates_json_result_value() {
        let mut buf = [0u8; 256];
        let len = write_success_response(&mut buf, "abc", "raw \"user\" text");
        let s = std::str::from_utf8(&buf[..len]).unwrap();
        assert!(s.contains("\"handler_failed\""));
    }

    #[test]
    fn write_json_string_value_test() {
        let mut buf = [0u8; 64];
        let len = write_json_string_value(&mut buf, r#"hello "user""#);
        let s = std::str::from_utf8(&buf[..len]).unwrap();
        assert_eq!(s, r#""hello \"user\"""#);
    }

    #[test]
    fn is_valid_json_value_test() {
        assert!(is_valid_json_value(r#"{"pong":true}"#));
        assert!(is_valid_json_value(r#"{"escaped\"key":true}"#));
        assert!(is_valid_json_value(r#""hello""#));
        assert!(is_valid_json_value("null"));
        assert!(!is_valid_json_value(r#"raw "user" text"#));
        assert!(!is_valid_json_value(r#"{"partial":true"#));
    }

    #[test]
    fn policy_allows() {
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

    #[test]
    fn policy_allows_wildcard() {
        let policy = Policy {
            enabled: true,
            permissions: vec![],
            commands: vec![CommandPolicy {
                name: "native.anywhere".into(),
                permissions: vec![],
                origins: vec!["*".into()],
            }],
        };
        assert!(policy.allows("native.anywhere", "https://example.com"));
    }

    #[test]
    fn policy_denied_by_permission() {
        let policy = Policy {
            enabled: true,
            permissions: vec![],
            commands: vec![CommandPolicy {
                name: "native.secure".into(),
                permissions: vec!["filesystem".into()],
                origins: vec!["zero://app".into()],
            }],
        };
        assert!(!policy.allows("native.secure", "zero://app"));
    }

    #[test]
    fn dispatcher_denied_before_unknown() {
        let dispatcher = Dispatcher::default();
        let mut buf = [0u8; 256];
        let len = dispatcher.dispatch(
            r#"{"id":"1","command":"native.ping","payload":null}"#,
            Source::default(),
            &mut buf,
        );
        let s = std::str::from_utf8(&buf[..len]).unwrap();
        assert!(s.contains("\"permission_denied\""));
    }

    #[test]
    fn valid_id_and_command() {
        assert!(valid_id("1"));
        assert!(!valid_id(""));
        assert!(valid_command("native.ping"));
        assert!(!valid_command("bad command"));
        assert!(!valid_command("a/b"));
    }

    #[test]
    fn async_registry_find() {
        let registry = AsyncRegistry::default();
        assert!(registry.find("native.ping").is_none());
    }
}
