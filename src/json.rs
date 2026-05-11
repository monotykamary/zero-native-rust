pub fn string_field<'a>(payload: &'a str, field: &str, storage: &mut Vec<u8>) -> Option<&'a str> {
    let value = field_value(payload, field)?;
    if value.len() < 2 || !value.starts_with('"') || !value.ends_with('"') {
        return None;
    }
    Some(&value[1..value.len() - 1])
}

pub fn bool_field(payload: &str, field: &str) -> Option<bool> {
    let value = field_value(payload, field)?;
    if value == "true" {
        Some(true)
    } else if value == "false" {
        Some(false)
    } else {
        None
    }
}

pub fn number_field(payload: &str, field: &str) -> Option<f32> {
    let bytes = number_bytes(payload, field)?;
    bytes.parse().ok()
}

pub fn unsigned_field<T: std::str::FromStr>(payload: &str, field: &str) -> Option<T> {
    let bytes = number_bytes(payload, field)?;
    bytes.parse().ok()
}

fn field_value<'a>(payload: &'a str, field: &str) -> Option<&'a str> {
    let search = format!("\"{}\"", field);
    let pos = payload.find(&search)?;
    let after = &payload[pos + search.len()..];
    let colon = after.find(':')?;
    let value_start = after[colon + 1..].trim_start();
    // Find value boundary — simplified for bridge messages
    let value_end = skip_value(value_start)?;
    Some(&value_start[..value_end])
}

fn number_bytes<'a>(payload: &'a str, field: &str) -> Option<&'a str> {
    let value = field_value(payload, field)?;
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
            let end = s[1..].find('"')? + 1;
            Some(end + 1)
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
        b'"' => trimmed.ends_with('"'),
        b'{' => trimmed.ends_with('}'),
        b'[' => trimmed.ends_with(']'),
        b't' => trimmed == "true",
        b'f' => trimmed == "false",
        b'n' => trimmed == "null",
        b'0'..=b'9' | b'-' => trimmed.parse::<f64>().is_ok(),
        _ => false,
    }
}
