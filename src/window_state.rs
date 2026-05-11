use crate::platform::WindowState;

pub const MAX_SERIALIZED_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone)]
pub struct Store {
    pub state_dir: String,
    pub file_path: String,
}

impl Store {
    pub fn new(state_dir: &str, file_path: &str) -> Self {
        Self {
            state_dir: state_dir.to_string(),
            file_path: file_path.to_string(),
        }
    }

    pub fn save_window(&self, state: &WindowState) -> std::io::Result<()> {
        if state.label.is_empty() {
            return Ok(());
        }
        let _ = std::fs::create_dir_all(&self.state_dir);
        let mut windows = self.load_windows()?;
        let mut found = false;
        for w in &mut windows {
            if (!state.label.is_empty() && w.label == state.label) || (state.id != 0 && w.id == state.id) {
                *w = state.clone();
                found = true;
                break;
            }
        }
        if !found {
            windows.push(state.clone());
        }
        let text = write_windows(&windows);
        std::fs::write(&self.file_path, &text)
    }

    pub fn load_windows(&self) -> std::io::Result<Vec<WindowState>> {
        let bytes = std::fs::read(&self.file_path)?;
        let text = String::from_utf8_lossy(&bytes);
        // Parse the simple ZON-like window state format
        Ok(parse_windows(&text))
    }
}

fn parse_windows(text: &str) -> Vec<WindowState> {
    let mut windows = Vec::new();
    for line in text.lines() {
        let trimmed = line.trim();
        if !trimmed.starts_with('.') || !trimmed.contains("id =") {
            continue;
        }
        if let Some(w) = parse_window_line(trimmed) {
            windows.push(w);
        }
    }
    windows
}

fn parse_window_line(line: &str) -> Option<WindowState> {
    let id = extract_zon_int(line, "id")?;
    let label = extract_zon_string(line, "label")?;
    let title = extract_zon_string(line, "title").unwrap_or_default();
    let open = extract_zon_bool(line, "open").unwrap_or(true);
    let focused = extract_zon_bool(line, "focused").unwrap_or(false);
    let x = extract_zon_float(line, "x")?;
    let y = extract_zon_float(line, "y")?;
    let width = extract_zon_float(line, "width")?;
    let height = extract_zon_float(line, "height")?;
    let scale = extract_zon_float(line, "scale").unwrap_or(1.0);
    let maximized = extract_zon_bool(line, "maximized").unwrap_or(false);
    let fullscreen = extract_zon_bool(line, "fullscreen").unwrap_or(false);
    Some(WindowState {
        id,
        label,
        title,
        frame: crate::geometry::RectF::new(x, y, width, height),
        scale_factor: scale,
        open,
        focused,
        maximized,
        fullscreen,
    })
}

fn extract_zon_int(line: &str, field: &str) -> Option<u64> {
    let pattern = format!(".{} = ", field);
    let pos = line.find(&pattern)? + pattern.len();
    let rest = &line[pos..];
    let end = rest.find(|c: char| !c.is_ascii_digit()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}

fn extract_zon_string(line: &str, field: &str) -> Option<String> {
    let pattern = format!(".{} = ", field);
    let pos = line.find(&pattern)? + pattern.len();
    let rest = &line[pos..];
    if !rest.starts_with('"') { return None; }
    let mut result = String::new();
    let mut chars = rest[1..].chars();
    loop {
        match chars.next()? {
            '"' => break,
            '\\' => match chars.next()? {
                '"' => result.push('"'),
                'n' => result.push('\n'),
                'r' => result.push('\r'),
                't' => result.push('\t'),
                '\\' => result.push('\\'),
                c => result.push(c),
            },
            c => result.push(c),
        }
    }
    Some(result)
}

fn extract_zon_bool(line: &str, field: &str) -> Option<bool> {
    let pattern = format!(".{} = ", field);
    let pos = line.find(&pattern)? + pattern.len();
    let rest = &line[pos..];
    if rest.starts_with("true") { Some(true) }
    else if rest.starts_with("false") { Some(false) }
    else { None }
}

fn extract_zon_float(line: &str, field: &str) -> Option<f32> {
    let pattern = format!(".{} = ", field);
    let pos = line.find(&pattern)? + pattern.len();
    let rest = &line[pos..];
    let end = rest.find(|c: char| c == ',' || c == '}' || c.is_whitespace()).unwrap_or(rest.len());
    rest[..end].parse().ok()
}


pub fn write_windows(windows: &[WindowState]) -> String {
    let mut out = String::with_capacity(256);
    out.push_str(".{\n  .windows = .{\n");
    for w in windows {
        out.push_str(&format!(
            "    .{{ .id = {}, .label = {}, .title = {}, .open = {:?}, .focused = {:?}, .x = {}, .y = {}, .width = {}, .height = {}, .scale = {}, .maximized = {:?}, .fullscreen = {:?} }},\n",
            w.id,
            zon_string(&w.label),
            zon_string(&w.title),
            w.open,
            w.focused,
            w.frame.x,
            w.frame.y,
            w.frame.width,
            w.frame.height,
            w.scale_factor,
            w.maximized,
            w.fullscreen,
        ));
    }
    out.push_str("  },\n}\n");
    out
}

fn zon_string(value: &str) -> String {
    let mut out = String::with_capacity(value.len() + 2);
    out.push('"');
    for ch in value.chars() {
        match ch {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) <= 0x1f => out.push_str(&format!("\\x{:02x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::geometry::RectF;

    #[test]
    fn window_state_writes_records() {
        let windows = vec![
            WindowState {
                id: 1, label: "main".into(), title: String::new(),
                frame: RectF::new(10.0, 20.0, 800.0, 600.0),
                scale_factor: 2.0, open: true, focused: true, maximized: false, fullscreen: false,
            },
            WindowState {
                id: 2, label: "settings".into(), title: String::new(),
                frame: RectF::new(30.0, 40.0, 500.0, 400.0),
                scale_factor: 1.0, open: false, focused: false, maximized: false, fullscreen: false,
            },
        ];
        let text = write_windows(&windows);
        assert!(text.contains("main"));
        assert!(text.contains("settings"));
        assert!(text.contains("id = 1"));
        assert!(text.contains("id = 2"));
    }

    #[test]
    fn window_state_escapes_special_chars() {
        let windows = vec![
            WindowState {
                id: 1, label: "tools\"panel".into(), title: "Title with \"quotes\", slash \\, newline\n".into(),
                frame: RectF::new(10.0, 20.0, 800.0, 600.0),
                scale_factor: 1.0, open: false, focused: false, maximized: false, fullscreen: false,
            },
        ];
        let text = write_windows(&windows);
        assert!(text.contains("\\\"quotes\\\"")); // escaped quotes in output
        assert!(text.contains("\\n")); // escaped newline in output
    }

    #[test]
    fn window_state_write_multiple_records_has_ids_and_labels() {
        let windows = vec![
            WindowState {
                id: 1, label: "main".into(), title: String::new(),
                frame: RectF::new(10.0, 20.0, 800.0, 600.0),
                scale_factor: 2.0, open: true, focused: true, maximized: false, fullscreen: false,
            },
            WindowState {
                id: 2, label: "settings".into(), title: String::new(),
                frame: RectF::new(30.0, 40.0, 500.0, 400.0),
                scale_factor: 1.0, open: false, focused: false, maximized: false, fullscreen: false,
            },
        ];
        let text = write_windows(&windows);
        assert!(text.contains(".id = 1"));
        assert!(text.contains(".id = 2"));
        assert!(text.contains(".label = \"main\""));
        assert!(text.contains(".label = \"settings\""));
        assert!(text.contains(".scale = 2"));
    }
}
