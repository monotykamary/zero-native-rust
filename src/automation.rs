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
