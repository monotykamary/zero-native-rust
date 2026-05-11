use crate::platform::WindowState;

pub const MAX_SERIALIZED_BYTES: usize = 64 * 1024;

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
