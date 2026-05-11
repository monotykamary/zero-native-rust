pub const DEFAULT_DIR: &str = ".zig-cache/zero-native-automation";
pub const MAX_COMMAND_BYTES: usize = 16 * 1024 + 64;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Reload,
    Wait,
    Bridge,
}

#[derive(Debug, Clone)]
pub struct Command {
    pub action: Action,
    pub value: String,
}

impl Command {
    pub fn parse(line: &str) -> Result<Self, CommandError> {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            return Err(CommandError::InvalidCommand);
        }
        let (action_text, value) = if let Some(pos) = trimmed.find(' ') {
            (&trimmed[..pos], trimmed[pos + 1..].trim().to_string())
        } else {
            (trimmed, String::new())
        };
        match action_text {
            "reload" => Ok(Self {
                action: Action::Reload,
                value,
            }),
            "wait" => Ok(Self {
                action: Action::Wait,
                value,
            }),
            "bridge" if !value.is_empty() => Ok(Self {
                action: Action::Bridge,
                value,
            }),
            _ => Err(CommandError::InvalidCommand),
        }
    }
}

pub fn command_line(action: &str, value: &str, output: &mut [u8]) -> Result<usize, CommandError> {
    let action_bytes = action.as_bytes();
    let value_bytes = value.as_bytes();
    if action_bytes.len() + value_bytes.len() + 2 > MAX_COMMAND_BYTES {
        return Err(CommandError::CommandTooLarge);
    }
    let mut pos = 0;
    output[pos..pos + action_bytes.len()].copy_from_slice(action_bytes);
    pos += action_bytes.len();
    if !value.is_empty() {
        output[pos] = b' ';
        pos += 1;
        output[pos..pos + value_bytes.len()].copy_from_slice(value_bytes);
        pos += value_bytes.len();
    }
    output[pos] = b'\n';
    pos += 1;
    Ok(pos)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandError {
    InvalidCommand,
    CommandTooLarge,
}

pub mod snapshot {
    use crate::geometry::RectF;
    use crate::platform::{self, WebViewSource};

    pub const MAX_WINDOWS: usize = platform::MAX_WINDOWS;

    #[derive(Debug, Clone)]
    pub struct Window {
        pub id: platform::WindowId,
        pub title: String,
        pub bounds: RectF,
        pub focused: bool,
    }

    #[derive(Debug, Clone, Copy)]
    pub struct Diagnostics {
        pub frame_index: u64,
        pub command_count: usize,
    }

    impl Default for Diagnostics {
        fn default() -> Self {
            Self {
                frame_index: 0,
                command_count: 0,
            }
        }
    }

    #[derive(Debug, Clone)]
    pub struct Input {
        pub windows: Vec<Window>,
        pub diagnostics: Diagnostics,
        pub source: Option<WebViewSource>,
    }

    pub fn write_text(input: &Input) -> String {
        let mut out = String::with_capacity(1024);
        out.push_str(&format!(
            "ready=true frame={} commands={}\n",
            input.diagnostics.frame_index, input.diagnostics.command_count
        ));
        for window in &input.windows {
            out.push_str(&format!(
                "window @w{} \"{}\" bounds=({},{},{},{}) focused={:?}\n",
                window.id,
                window.title,
                window.bounds.x,
                window.bounds.y,
                window.bounds.width,
                window.bounds.height,
                window.focused,
            ));
        }
        if let Some(ref source) = input.source {
            out.push_str(&format!(
                "  source kind={} bytes={}\n",
                source.kind_name(),
                source.bytes.len()
            ));
        }
        out
    }

    pub fn write_a11y_text(input: &Input) -> String {
        let mut out = String::with_capacity(512);
        out.push_str(&format!("a11y root=@w1 nodes={}\n", input.windows.len()));
        for window in &input.windows {
            out.push_str(&format!(
                "@w{} role=window name=\"{}\" bounds=({},{},{},{})\n",
                window.id,
                window.title,
                window.bounds.x,
                window.bounds.y,
                window.bounds.width,
                window.bounds.height,
            ));
        }
        out
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn snapshot_emits_window_and_source() {
            let windows = vec![Window {
                id: 1,
                title: "Test".into(),
                bounds: RectF::new(0.0, 0.0, 100.0, 100.0),
                focused: true,
            }];
            let input = Input {
                windows,
                diagnostics: Diagnostics::default(),
                source: Some(WebViewSource::html("<h1>Hello</h1>")),
            };
            let text = write_text(&input);
            assert!(text.contains("ready=true"));
            assert!(text.contains("@w1"));
            assert!(text.contains("source kind=html"));
        }
    }
}

#[derive(Debug, Clone)]
pub struct Server {
    pub directory: String,
    pub title: String,
}

impl Server {
    pub fn new(directory: &str, title: &str) -> Self {
        Self {
            directory: directory.to_string(),
            title: title.to_string(),
        }
    }

    pub fn publish(&self, input: &snapshot::Input) -> std::io::Result<()> {
        let _ = std::fs::create_dir_all(&self.directory);

        let snapshot_path = std::path::Path::new(&self.directory).join("snapshot.txt");
        std::fs::write(&snapshot_path, snapshot::write_text(input))?;

        let a11y_path = std::path::Path::new(&self.directory).join("accessibility.txt");
        std::fs::write(&a11y_path, snapshot::write_a11y_text(input))?;

        let mut windows_text = String::with_capacity(512);
        for window in &input.windows {
            windows_text.push_str(&format!(
                "window @w{} \"{}\" focused={:?}\n",
                window.id, window.title, window.focused
            ));
        }
        let windows_path = std::path::Path::new(&self.directory).join("windows.txt");
        std::fs::write(&windows_path, &windows_text)?;

        Ok(())
    }

    pub fn publish_bridge_response(&self, response: &[u8]) -> std::io::Result<()> {
        let _ = std::fs::create_dir_all(&self.directory);
        let path = std::path::Path::new(&self.directory).join("bridge-response.txt");
        std::fs::write(&path, response)?;
        Ok(())
    }

    pub fn take_command(&self) -> Option<Command> {
        let path = std::path::Path::new(&self.directory).join("command.txt");
        let bytes = std::fs::read(&path).ok()?;
        let line = String::from_utf8_lossy(&bytes);
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed == "done" {
            return None;
        }
        let command = Command::parse(trimmed).ok()?;
        // Mark command as consumed
        let _ = std::fs::write(&path, "done\n");
        Some(command)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn commands_parse_reload_and_wait() {
        let reload = Command::parse("reload").unwrap();
        assert_eq!(Action::Reload, reload.action);

        let wait = Command::parse("wait frame").unwrap();
        assert_eq!(Action::Wait, wait.action);
        assert_eq!("frame", wait.value);

        let bridge = Command::parse("bridge {\"id\":\"1\",\"command\":\"native.ping\"}").unwrap();
        assert_eq!(Action::Bridge, bridge.action);
        assert!(bridge.value.contains("native.ping"));
    }

    #[test]
    fn command_parse_rejects_invalid() {
        assert!(Command::parse("").is_err());
        assert!(Command::parse("bridge").is_err()); // bridge requires payload
        assert!(Command::parse("unknown").is_err());
    }

    #[test]
    fn command_line_formatting() {
        let mut buf = [0u8; 256];
        let len = command_line("reload", "", &mut buf).unwrap();
        assert_eq!(b"reload\n", &buf[..len]);

        let len = command_line("wait", "frame", &mut buf).unwrap();
        assert_eq!(b"wait frame\n", &buf[..len]);
    }

    #[test]
    fn server_stores_directory_metadata() {
        let server = Server::new(".zig-cache/test-webview-automation", "Test");
        assert_eq!("Test", server.title);
    }
}
