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
