pub struct StringStorage {
    buffer: Vec<u8>,
    index: usize,
}

impl StringStorage {
    pub fn new(buffer: Vec<u8>) -> Self {
        Self { buffer, index: 0 }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self { buffer: vec![0u8; capacity], index: 0 }
    }

    fn append(&mut self, bytes: &[u8]) -> Result<(), ()> {
        if self.index + bytes.len() > self.buffer.len() { return Err(()); }
        self.buffer[self.index..self.index + bytes.len()].copy_from_slice(bytes);
        self.index += bytes.len();
        Ok(())
    }

    fn append_byte(&mut self, byte: u8) -> Result<(), ()> {
        if self.index >= self.buffer.len() { return Err(()); }
        self.buffer[self.index] = byte;
        self.index += 1;
        Ok(())
    }

    pub fn as_slice(&self) -> &[u8] { &self.buffer[..self.index] }
}

pub fn field_value<'a>(payload: &'a str, field: &str) -> Option<&'a str> {
    let search = format!("\"{}\"", field);
    let pos = payload.find(&search)?;
    let after = &payload[pos + search.len()..];
    let colon = after.find(':')?;
    let value_start = after[colon + 1..].trim_start();
    let value_end = skip_value(value_start)?;
    Some(&value_start[..value_end])
}

pub fn string_field<'a>(payload: &'a str, field: &str, storage: &mut Vec<u8>) -> Option<&'a str> {
    let value = field_value(payload, field)?;
    if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
        return None;
    }
    Some(&value[1..value.len() - 1])
}

pub fn string_field_unescaped(payload: &str, field: &str) -> Option<String> {
    let value = field_value(payload, field)?;
    let mut storage = StringStorage::with_capacity(value.len());
    match parse_string_value(value, &mut storage) {
        Ok(s) => Some(s.to_string()),
        Err(_) => None,
    }
}

pub fn bool_field(payload: &str, field: &str) -> Option<bool> {
    let value = field_value(payload, field)?;
    if value == "true" { Some(true) }
    else if value == "false" { Some(false) }
    else { None }
}

pub fn number_field(payload: &str, field: &str) -> Option<f32> {
    let bytes = number_bytes(payload, field)?;
    bytes.parse().ok()
}

pub fn unsigned_field<T: std::str::FromStr>(payload: &str, field: &str) -> Option<T> {
    let bytes = number_bytes(payload, field)?;
    bytes.parse().ok()
}

fn number_bytes<'a>(payload: &'a str, field: &str) -> Option<&'a str> {
    let value = field_value(payload, field)?;
    if value.is_empty() { return None; }
    let end = value
        .char_indices()
        .find(|(_, c)| !c.is_ascii_digit() && *c != '.' && *c != '-')
        .map(|(i, _)| i)
        .unwrap_or(value.len());
    if end == 0 { return None; }
    Some(&value[..end])
}

fn skip_value(s: &str) -> Option<usize> {
    let bytes = s.as_bytes();
    if bytes.is_empty() { return None; }
    match bytes[0] {
        b'"' => {
            let mut i = 1;
            while i < bytes.len() {
                match bytes[i] {
                    b'"' => return Some(i + 1),
                    b'\\' => i += 2,
                    _ => i += 1,
                }
            }
            None
        }
        b'{' | b'[' => {
            let mut depth = 1i32;
            let mut i = 1;
            while i < bytes.len() && depth > 0 {
                match bytes[i] {
                    b'{' | b'[' => depth += 1,
                    b'}' | b']' => depth -= 1,
                    _ => {}
                }
                i += 1;
            }
            Some(i)
        }
        _ => {
            let end = s
                .char_indices()
                .find(|(_, c)| *c == ',' || *c == '}' || *c == ']' || c.is_whitespace())
                .map(|(i, _)| i)
                .unwrap_or(s.len());
            Some(end)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseStringError {
    InvalidJson,
    NoSpaceLeft,
    NonAsciiEscape,
}

pub fn parse_string_value<'a>(value: &'a str, storage: &'a mut StringStorage) -> Result<&'a str, ParseStringError> {
    if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
        return Err(ParseStringError::InvalidJson);
    }
    let bytes = value.as_bytes();
    let mut index = 1usize;
    let direct_start = index;
    let mut copied = false;
    let output_start = storage.index;

    while index + 1 < bytes.len() {
        let ch = bytes[index];
        if ch == b'\\' {
            if !copied {
                storage.append(&value.as_bytes()[direct_start..index]).map_err(|_| ParseStringError::NoSpaceLeft)?;
                copied = true;
            }
            index += 1;
            if index + 1 >= bytes.len() { return Err(ParseStringError::InvalidJson); }
            match bytes[index] {
                b'"' => storage.append_byte(b'"').map_err(|_| ParseStringError::NoSpaceLeft)?,
                b'\\' => storage.append_byte(b'\\').map_err(|_| ParseStringError::NoSpaceLeft)?,
                b'/' => storage.append_byte(b'/').map_err(|_| ParseStringError::NoSpaceLeft)?,
                b'b' => storage.append_byte(0x08).map_err(|_| ParseStringError::NoSpaceLeft)?,
                b'f' => storage.append_byte(0x0c).map_err(|_| ParseStringError::NoSpaceLeft)?,
                b'n' => storage.append_byte(b'\n').map_err(|_| ParseStringError::NoSpaceLeft)?,
                b'r' => storage.append_byte(b'\r').map_err(|_| ParseStringError::NoSpaceLeft)?,
                b't' => storage.append_byte(b'\t').map_err(|_| ParseStringError::NoSpaceLeft)?,
                b'u' => {
                    if index + 4 >= bytes.len() { return Err(ParseStringError::InvalidJson); }
                    let codepoint = parse_hex4(&value.as_bytes()[index + 1..index + 5])?;
                    if codepoint > 0x7f { return Err(ParseStringError::NonAsciiEscape); }
                    storage.append_byte(codepoint as u8).map_err(|_| ParseStringError::NoSpaceLeft)?;
                    index += 4;
                }
                _ => return Err(ParseStringError::InvalidJson),
            }
            index += 1;
            continue;
        }
        if ch <= 0x1f { return Err(ParseStringError::InvalidJson); }
        if copied { storage.append_byte(ch).map_err(|_| ParseStringError::NoSpaceLeft)?; }
        index += 1;
    }
    if !copied { return Ok(&value[direct_start..value.len() - 1]); }
    let start = output_start;
    let end = storage.index;
    Ok(std::str::from_utf8(&storage.buffer[start..end]).unwrap_or(""))
}

fn parse_hex4(bytes: &[u8]) -> Result<u32, ParseStringError> {
    if bytes.len() != 4 { return Err(ParseStringError::InvalidJson); }
    let mut result: u32 = 0;
    for &ch in bytes {
        result <<= 4;
        result += hex_value(ch).ok_or(ParseStringError::InvalidJson)?;
    }
    Ok(result)
}

fn hex_value(ch: u8) -> Option<u32> {
    match ch {
        b'0'..=b'9' => Some((ch - b'0') as u32),
        b'a'..=b'f' => Some((ch - b'a') as u32 + 10),
        b'A'..=b'F' => Some((ch - b'A') as u32 + 10),
        _ => None,
    }
}

pub fn write_string(value: &str, output: &mut [u8]) -> usize {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(output);
    let _ = cursor.write_all(b"\"");
    for ch in value.bytes() {
        match ch {
            b'"' => { let _ = cursor.write_all(b"\\\""); }
            b'\\' => { let _ = cursor.write_all(b"\\\\"); }
            b'\n' => { let _ = cursor.write_all(b"\\n"); }
            b'\r' => { let _ = cursor.write_all(b"\\r"); }
            b'\t' => { let _ = cursor.write_all(b"\\t"); }
            0..=8 | 11..=12 | 14..=0x1f => { let _ = write!(cursor, "\\u{:04x}", ch); }
            _ => { let _ = cursor.write_all(&[ch]); }
        }
    }
    let _ = cursor.write_all(b"\"");
    cursor.position() as usize
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

pub fn is_valid_value(raw: &str) -> bool {
    let trimmed = raw.trim();
    if trimmed.is_empty() { return false; }
    match trimmed.as_bytes()[0] {
        b'"' => {
            if !trimmed.ends_with('"') { return false; }
            let inner = &trimmed[1..trimmed.len()-1];
            let mut i = 0;
            let bytes = inner.as_bytes();
            while i < bytes.len() {
                match bytes[i] {
                    b'\\' => { i += 2; }
                    b if b <= 0x1f => { return false; }
                    _ => { i += 1; }
                }
            }
            true
        }
        b'{' => {
            if !trimmed.ends_with('}') { return false; }
            true
        }
        b'[' => {
            if !trimmed.ends_with(']') { return false; }
            true
        }
        b't' => trimmed == "true",
        b'f' => trimmed == "false",
        b'n' => trimmed == "null",
        b'0'..=b'9' | b'-' => trimmed.parse::<f64>().is_ok(),
        _ => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn string_field_extracts_value() {
        let mut storage = Vec::new();
        let input = r#"{"title":"Hello","nested":{"title":"wrong"}}"#;
        let value = string_field(input, "title", &mut storage);
        assert!(value.is_some());
        assert_eq!("Hello", value.unwrap());
    }

    #[test]
    fn string_field_unescaped_test() {
        let input = r#"{"title":"Hello \"user\"\n","nested":{"title":"wrong"}}"#;
        let value = string_field_unescaped(input, "title");
        assert!(value.is_some());
        assert_eq!("Hello \"user\"\n", value.unwrap());
    }

    #[test]
    fn validates_json_values() {
        assert!(is_valid_value("{\"ok\":true}"));
        assert!(!is_valid_value("{\"ok\":true"));
        assert!(is_valid_value("null"));
        assert!(is_valid_value("42"));
        assert!(is_valid_value("\"hello\""));
        assert!(!is_valid_value("raw \"user\" text"));
        assert!(is_valid_value("{\"escaped\\\"key\":true}"));
    }

    #[test]
    fn bool_field_extract() {
        let input = r#"{"active":true}"#;
        assert_eq!(Some(true), bool_field(input, "active"));
        assert_eq!(None, bool_field(input, "missing"));
    }

    #[test]
    fn number_field_extract() {
        let input = r#"{"count":42}"#;
        assert_eq!(Some(42.0), number_field(input, "count"));
    }

    #[test]
    fn unsigned_field_extract() {
        let input = r#"{"id":5}"#;
        assert_eq!(Some(5u64), unsigned_field(input, "id"));
    }

    #[test]
    fn write_json_string_escapes() {
        assert_eq!(r#""hello""#, write_json_string("hello"));
        assert!(write_json_string("a\"b").contains("\\\""));
        assert!(write_json_string("a\\b").contains("\\\\"));
        assert!(write_json_string("a\nb").contains("\\n"));
    }

    #[test]
    fn write_string_to_buffer() {
        let mut buf = [0u8; 64];
        let len = write_string("hello \"world\"", &mut buf);
        let s = std::str::from_utf8(&buf[..len]).unwrap();
        assert_eq!(s, r#""hello \"world\"""#);
    }

    #[test]
    fn field_value_returns_raw() {
        let input = r#"{"value":42,"name":"test"}"#;
        assert_eq!(Some("42"), field_value(input, "value"));
        assert_eq!(Some("\"test\""), field_value(input, "name"));
    }

    #[test]
    fn parse_string_value_unescapes() {
        let mut storage = StringStorage::with_capacity(128);
        let result = parse_string_value(r#""hello \"world\"""#, &mut storage).unwrap();
        assert_eq!("hello \"world\"", result);
    }
}
