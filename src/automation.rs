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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommandError {
    InvalidCommand,
    CommandTooLarge,
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
        // Verify the command format matches what the Zig automation server expects
        let reload = Command::parse("reload").unwrap();
        assert_eq!(Action::Reload, reload.action);
        assert!(reload.value.is_empty());

        let wait = Command::parse("wait frame").unwrap();
        assert_eq!("frame", wait.value);
    }
}
