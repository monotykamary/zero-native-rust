pub const PERMISSION_WINDOW: &str = "window";
pub const PERMISSION_FILESYSTEM: &str = "filesystem";
pub const PERMISSION_CLIPBOARD: &str = "clipboard";
pub const PERMISSION_NETWORK: &str = "network";

#[derive(Debug, Clone)]
pub struct Policy {
    pub permissions: Vec<String>,
    pub navigation: NavigationPolicy,
}

impl Default for Policy {
    fn default() -> Self {
        Self {
            permissions: Vec::new(),
            navigation: NavigationPolicy::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NavigationPolicy {
    pub allowed_origins: Vec<String>,
    pub external_links: ExternalLinkPolicy,
}

impl Default for NavigationPolicy {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["zero://app".into(), "zero://inline".into()],
            external_links: ExternalLinkPolicy::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct ExternalLinkPolicy {
    pub action: ExternalLinkAction,
    pub allowed_urls: Vec<String>,
}

impl Default for ExternalLinkPolicy {
    fn default() -> Self {
        Self {
            action: ExternalLinkAction::Deny,
            allowed_urls: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum ExternalLinkAction {
    Deny = 0,
    OpenSystemBrowser = 1,
}

pub fn has_permission(grants: &[String], permission: &str) -> bool {
    grants.iter().any(|g| g == permission)
}

pub fn has_permissions(grants: &[String], required: &[&str]) -> bool {
    required.iter().all(|r| has_permission(grants, r))
}

pub fn allows_origin(allowed: &[String], origin: &str) -> bool {
    allowed.iter().any(|a| a == "*" || a == origin)
}
