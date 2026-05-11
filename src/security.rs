pub const PERMISSION_WINDOW: &str = "window";
pub const PERMISSION_FILESYSTEM: &str = "filesystem";
pub const PERMISSION_CLIPBOARD: &str = "clipboard";
pub const PERMISSION_NETWORK: &str = "network";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalLinkAction {
    Deny = 0,
    OpenSystemBrowser = 1,
}

impl Default for ExternalLinkAction {
    fn default() -> Self { Self::Deny }
}

#[derive(Debug, Clone)]
pub struct ExternalLinkPolicy {
    pub action: ExternalLinkAction,
    pub allowed_urls: Vec<String>,
}

impl Default for ExternalLinkPolicy {
    fn default() -> Self {
        Self { action: ExternalLinkAction::Deny, allowed_urls: Vec::new() }
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

pub fn has_permission(grants: &[&str], permission: &str) -> bool {
    grants.iter().any(|g| *g == permission)
}

pub fn has_permissions(grants: &[&str], required: &[&str]) -> bool {
    required.iter().all(|r| has_permission(grants, r))
}

pub fn allows_origin(allowed_origins: &[&str], origin: &str) -> bool {
    allowed_origins.iter().any(|o| *o == "*" || *o == origin)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn permission_checks_require_every_grant() {
        assert!(has_permissions(&[PERMISSION_WINDOW, PERMISSION_FILESYSTEM], &[PERMISSION_WINDOW]));
        assert!(!has_permissions(&[PERMISSION_WINDOW], &[PERMISSION_WINDOW, PERMISSION_FILESYSTEM]));
    }

    #[test]
    fn origin_checks_support_exact_and_wildcard() {
        assert!(allows_origin(&["zero://app", "zero://inline"], "zero://inline"));
        assert!(allows_origin(&["*"], "https://example.invalid"));
        assert!(!allows_origin(&["zero://app"], "https://example.invalid"));
    }
}
