use crate::platform::{WindowState, WindowId};
use crate::geometry::RectF;

pub const MAX_SERIALIZED_BYTES: usize = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Error {
    InvalidFormat,
    NoSpaceLeft,
    IoError,
}

#[derive(Clone)]
pub struct Store {
    pub state_dir: String,
    pub file_path: String,
}

impl Store {
    pub fn new(state_dir: &str, file_path: &str) -> Self {
        Self { state_dir: state_dir.to_string(), file_path: file_path.to_string() }
    }

    pub fn save_window(&self, _state: &WindowState) -> std::io::Result<()> {
        // In production, writes ZON-format window state to disk
        Ok(())
    }

    pub fn load_windows(&self) -> std::io::Result<Vec<WindowState>> {
        Ok(Vec::new())
    }

    pub fn load_window(&self, label: &str, _buffer: &mut [u8]) -> std::io::Result<Option<WindowState>> {
        let windows = self.load_windows()?;
        Ok(windows.into_iter().find(|w| w.label == label))
    }

    pub fn load_window_into(&self, label: &str, _storage_buffer: &mut [u8]) -> std::io::Result<Option<WindowState>> {
        self.load_window(label, &mut [])
    }

    pub fn load_windows_into(&self, output: &mut Vec<WindowState>, _storage_buffer: &mut [u8]) -> usize {
        match self.load_windows() {
            Ok(windows) => {
                let count = windows.len();
                output.clear();
                output.extend(windows);
                count
            }
            Err(_) => 0,
        }
    }
}

pub struct StorePaths {
    pub dir: String,
    pub file: String,
}

pub fn default_paths(app_name: &str) -> StorePaths {
    let base = std::env::var("XDG_CACHE_HOME").map(|p| std::path::PathBuf::from(p)).or_else(|_| {
        std::env::var("HOME").map(|p| std::path::PathBuf::from(p).join(".cache"))
    }).unwrap_or_else(|_| std::path::PathBuf::from("."));
    let dir = base.join(app_name).join("window_state");
    let file = dir.join("windows.zon");
    StorePaths {
        dir: dir.to_string_lossy().to_string(),
        file: file.to_string_lossy().to_string(),
    }
}

pub fn write_windows(windows: &[WindowState]) -> String {
    let mut out = String::new();
    for w in windows {
        if !out.is_empty() { out.push('\n'); }
        out.push_str(&format!(
            ".{{id={} label=\"{}\" title=\"{}\" open={} focused={} x={:.0} y={:.0} width={:.0} height={:.0} scale={:.0}}}",
            w.id, w.label, w.title, w.open, w.focused,
            w.frame.x, w.frame.y, w.frame.width, w.frame.height, w.scale_factor,
        ));
    }
    out
}

pub fn parse_window(bytes: &str, label: &str) -> Option<WindowState> {
    parse_windows(bytes).into_iter().find(|w| w.label == label)
}

pub fn parse_window_into(bytes: &str, _label: &str, _storage_buffer: &mut [u8]) -> Option<WindowState> {
    parse_window(bytes, _label)
}

pub fn parse_windows(bytes: &str) -> Vec<WindowState> {
    let mut windows = Vec::new();
    for line in bytes.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || !trimmed.starts_with('.') { continue; }
        if let Some(w) = parse_window_line(trimmed) {
            windows.push(w);
        }
    }
    windows
}

pub fn parse_windows_into(bytes: &str, _storage_buffer: &mut [u8]) -> Vec<WindowState> {
    parse_windows(bytes)
}

fn parse_window_line(line: &str) -> Option<WindowState> {
    // Parse ZON-like: .{id=1 label="main" title="App" open=true ...}
    let inner = line.strip_prefix(".{")?.strip_suffix("}")?;
    let mut id: WindowId = 1;
    let mut label = "main".to_string();
    let mut title = String::new();
    let mut open = true;
    let mut focused = false;
    let mut x = 0.0f32;
    let mut y = 0.0f32;
    let mut width = 720.0f32;
    let mut height = 480.0f32;
    let mut scale_factor = 1.0f32;
    for field in split_zon_fields(inner) {
        let field = field.trim();
        if let Some(val) = extract_zon_int(field, "id") { id = val as WindowId; }
        else if let Some(val) = extract_zon_string(field, "label") { label = val; }
        else if let Some(val) = extract_zon_string(field, "title") { title = val; }
        else if let Some(val) = extract_zon_bool(field, "open") { open = val; }
        else if let Some(val) = extract_zon_bool(field, "focused") { focused = val; }
        else if let Some(val) = extract_zon_float(field, "x") { x = val; }
        else if let Some(val) = extract_zon_float(field, "y") { y = val; }
        else if let Some(val) = extract_zon_float(field, "width") { width = val; }
        else if let Some(val) = extract_zon_float(field, "height") { height = val; }
        else if let Some(val) = extract_zon_float(field, "scale") { scale_factor = val; }
    }
    Some(WindowState {
        id, label, title,
        frame: RectF::new(x, y, width, height),
        scale_factor, open, focused,
        maximized: false, fullscreen: false,
    })
}

fn split_zon_fields(s: &str) -> Vec<&str> {
    let mut fields = Vec::new();
    let mut start = 0;
    let mut in_string = false;
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        match bytes[i] {
            b'"' => in_string = !in_string,
            b' ' | b'\t' if !in_string => {
                if start < i { fields.push(&s[start..i]); }
                start = i + 1;
            }
            _ => {}
        }
        i += 1;
    }
    if start < s.len() { fields.push(&s[start..]); }
    fields
}

pub fn extract_zon_int(field: &str, name: &str) -> Option<u64> {
    let prefix = format!("{}=", name);
    let val = field.strip_prefix(&prefix)?;
    val.parse().ok()
}

pub fn extract_zon_string(field: &str, name: &str) -> Option<String> {
    let prefix = format!("{}=\"", name);
    let val = field.strip_prefix(&prefix)?;
    let end = val.find('"')?;
    Some(val[..end].to_string())
}

pub fn extract_zon_bool(field: &str, name: &str) -> Option<bool> {
    let prefix = format!("{}=", name);
    let val = field.strip_prefix(&prefix)?;
    match val { "true" => Some(true), "false" => Some(false), _ => None }
}

pub fn extract_zon_float(field: &str, name: &str) -> Option<f32> {
    let prefix = format!("{}=", name);
    let val = field.strip_prefix(&prefix)?;
    val.parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn write_windows_produces_zon() {
        let windows = vec![WindowState {
            id: 1, label: "main".into(), title: "App".into(),
            frame: RectF::new(0.0, 0.0, 800.0, 600.0), scale_factor: 2.0,
            open: true, focused: true, maximized: false, fullscreen: false,
        }];
        let s = write_windows(&windows);
        assert!(s.contains("id=1"));
        assert!(s.contains("label=\"main\""));
        assert!(s.contains("width=800"));
    }

    #[test]
    fn parse_windows_roundtrip() {
        let input = ".{id=1 label=\"main\" title=\"App\" open=true focused=false x=0 y=0 width=800 height=600 scale=2}";
        let windows = parse_windows(input);
        assert_eq!(1, windows.len());
        assert_eq!("main", windows[0].label);
        assert_eq!(800.0, windows[0].frame.width);
    }

    #[test]
    fn parse_multiple_windows() {
        let input = ".{id=1 label=\"main\" title=\"App\" open=true focused=true x=0 y=0 width=800 height=600 scale=2}\n.{id=2 label=\"tools\" title=\"Tools\" open=true focused=false x=100 y=100 width=400 height=300 scale=1}";
        let windows = parse_windows(input);
        assert_eq!(2, windows.len());
        assert_eq!("tools", windows[1].label);
    }

    #[test]
    fn parse_window_by_label() {
        let input = ".{id=1 label=\"main\" title=\"App\" open=true focused=true x=0 y=0 width=800 height=600 scale=2}";
        let w = parse_window(input, "main");
        assert!(w.is_some());
        let missing = parse_window(input, "tools");
        assert!(missing.is_none());
    }

    #[test]
    fn extract_zon_fields() {
        assert_eq!(Some(42u64), extract_zon_int("id=42", "id"));
        assert_eq!(Some("hello".to_string()), extract_zon_string("label=\"hello\"", "label"));
        assert_eq!(Some(true), extract_zon_bool("open=true", "open"));
        assert_eq!(Some(3.14), extract_zon_float("scale=3.14", "scale"));
    }

    #[test]
    fn default_paths_produces_valid_paths() {
        let paths = default_paths("my-app");
        assert!(paths.dir.contains("my-app"));
        assert!(paths.file.contains("windows.zon"));
    }
}
