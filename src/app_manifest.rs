#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationError {
    InvalidId,
    InvalidName,
    InvalidVersion,
    InvalidDimension,
    DuplicateIcon,
    DuplicatePermission,
    DuplicateCapability,
    DuplicateBridgeCommand,
    DuplicatePlatform,
    DuplicateWindow,
    InvalidUrl,
    InvalidPath,
    InvalidCommand,
    InvalidTimeout,
    InvalidKeyword,
    MissingRequiredField,
    NoSpaceLeft,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    MacOS,
    Windows,
    Linux,
    IOS,
    Android,
    Web,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebEngine {
    System,
    Chromium,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconPurpose {
    Any,
    Maskable,
    Monochrome,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionKind {
    Network,
    Filesystem,
    Camera,
    Microphone,
    Location,
    Notifications,
    Clipboard,
    Window,
    Custom,
}

#[derive(Debug, Clone)]
pub struct Icon {
    pub asset: String,
    pub size: u32,
    pub scale: u32,
    pub purpose: Option<IconPurpose>,
}

#[derive(Debug, Clone)]
pub struct Window {
    pub label: String,
    pub title: Option<String>,
    pub width: f32,
    pub height: f32,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub resizable: bool,
    pub restore_state: bool,
}

impl Default for Window {
    fn default() -> Self {
        Self {
            label: "main".into(),
            title: None,
            width: 720.0,
            height: 480.0,
            x: None,
            y: None,
            resizable: true,
            restore_state: true,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Manifest {
    pub identity: AppIdentity,
    pub version: Version,
    pub icons: Vec<Icon>,
    pub permissions: Vec<PermissionKind>,
    pub windows: Vec<Window>,
    pub web_engine: WebEngine,
}

#[derive(Debug, Clone)]
pub struct AppIdentity {
    pub id: String,
    pub name: String,
    pub display_name: Option<String>,
    pub organization: Option<String>,
    pub homepage: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Version {
    pub major: u32,
    pub minor: u32,
    pub patch: u32,
    pub pre: Option<String>,
    pub build: Option<String>,
}

impl Version {
    pub fn to_string(&self) -> String {
        let mut s = format!("{}.{}.{}", self.major, self.minor, self.patch);
        if let Some(ref pre) = self.pre {
            s.push_str(&format!("-{}", pre));
        }
        if let Some(ref build) = self.build {
            s.push_str(&format!("+{}", build));
        }
        s
    }
}

pub fn validate_app_id(id: &str, require_reverse_dns: bool) -> Result<(), ValidationError> {
    if id.is_empty() || id.starts_with('.') || id.ends_with('.') {
        return Err(ValidationError::InvalidId);
    }
    let mut segments = 0usize;
    for part in id.split('.') {
        if part.is_empty() || part.starts_with('-') || part.ends_with('-') {
            return Err(ValidationError::InvalidId);
        }
        for ch in part.chars() {
            if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() && ch != '-' && ch != '_' {
                return Err(ValidationError::InvalidId);
            }
        }
        segments += 1;
    }
    if require_reverse_dns && segments < 2 {
        return Err(ValidationError::InvalidId);
    }
    Ok(())
}

pub fn validate_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() || name == "." || name == ".." {
        return Err(ValidationError::InvalidName);
    }
    if name.contains('\0') || name.contains('/') || name.contains('\\') {
        return Err(ValidationError::InvalidName);
    }
    Ok(())
}

pub fn validate_windows(windows: &[Window]) -> Result<(), ValidationError> {
    for (i, w) in windows.iter().enumerate() {
        if w.label.is_empty() {
            return Err(ValidationError::InvalidName);
        }
        if w.width <= 0.0 || w.height <= 0.0 {
            return Err(ValidationError::InvalidDimension);
        }
        for prev in &windows[..i] {
            if prev.label == w.label {
                return Err(ValidationError::DuplicateWindow);
            }
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_app_id() {
        assert!(validate_app_id("com.example.app", true).is_ok());
        assert!(validate_app_id("my-tool", false).is_ok());
        assert!(validate_app_id("example", true).is_err());
        assert!(validate_app_id("Com.example.app", true).is_err());
        assert!(validate_app_id("", true).is_err());
    }

    #[test]
    fn version_string() {
        let v = Version {
            major: 1,
            minor: 2,
            patch: 3,
            pre: Some("beta.1".into()),
            build: None,
        };
        assert_eq!(v.to_string(), "1.2.3-beta.1");
    }

    #[test]
    fn validate_windows_rejects_duplicates() {
        let windows = vec![
            Window { label: "main".into(), ..Default::default() },
            Window { label: "main".into(), ..Default::default() },
        ];
        assert!(validate_windows(&windows).is_err());
    }
}
