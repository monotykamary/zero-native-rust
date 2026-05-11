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
pub enum Platform { MacOS, Windows, Linux, IOS, Android, Web, Unknown }

impl Default for Platform { fn default() -> Self { Platform::Unknown } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PackageKind { App, Cli, Library, Plugin, TestFixture }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebEngine { System, Chromium }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IconPurpose { Any, Maskable, Monochrome }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PermissionKind {
    Network, Filesystem, Camera, Microphone, Location, Notifications, Clipboard, Window, Custom,
}

#[derive(Debug, Clone)]
pub struct Permission {
    pub kind: PermissionKind,
    pub custom_name: Option<String>,
}

impl Permission {
    pub fn built_in(kind: PermissionKind) -> Self { Self { kind, custom_name: None } }
    pub fn custom(name: &str) -> Self { Self { kind: PermissionKind::Custom, custom_name: Some(name.to_string()) } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CapabilityKind { NativeModule, Webview, JsBridge, Filesystem, Network, Clipboard, Custom }

#[derive(Debug, Clone)]
pub struct Capability {
    pub kind: CapabilityKind,
    pub custom_name: Option<String>,
}

impl Capability {
    pub fn built_in(kind: CapabilityKind) -> Self { Self { kind, custom_name: None } }
    pub fn custom(name: &str) -> Self { Self { kind: CapabilityKind::Custom, custom_name: Some(name.to_string()) } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppIdMode { ReverseDns, Simple }

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
    pub fn to_string_val(&self) -> String {
        let mut s = format!("{}.{}.{}", self.major, self.minor, self.patch);
        if let Some(ref pre) = self.pre { s.push_str(&format!("-{}", pre)); }
        if let Some(ref build) = self.build { s.push_str(&format!("+{}", build)); }
        s
    }
}

#[derive(Debug, Clone)]
pub struct Icon {
    pub asset: String,
    pub size: u32,
    pub scale: u32,
    pub purpose: Option<IconPurpose>,
}

#[derive(Debug, Clone, Default)]
pub struct PlatformSettings {
    pub platform: Platform,
    pub id_override: Option<String>,
    pub min_os_version: Option<String>,
    pub permissions: Vec<Permission>,
    pub category: Option<String>,
    pub entitlements: Option<String>,
    pub profile: Option<String>,
}

#[derive(Debug, Clone)]
pub struct BridgeCommand {
    pub name: String,
    pub permissions: Vec<Permission>,
    pub origins: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct BridgeConfig {
    pub commands: Vec<BridgeCommand>,
}

impl Default for BridgeConfig { fn default() -> Self { Self { commands: Vec::new() } } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExternalLinkAction { Deny, OpenSystemBrowser }

#[derive(Debug, Clone)]
pub struct ExternalLinkPolicy {
    pub action: ExternalLinkAction,
    pub allowed_urls: Vec<String>,
}

impl Default for ExternalLinkPolicy { fn default() -> Self { Self { action: ExternalLinkAction::Deny, allowed_urls: Vec::new() } } }

#[derive(Debug, Clone)]
pub struct NavigationPolicy {
    pub allowed_origins: Vec<String>,
    pub external_links: ExternalLinkPolicy,
}

impl Default for NavigationPolicy {
    fn default() -> Self { Self { allowed_origins: vec!["zero://app".into(), "zero://inline".into()], external_links: ExternalLinkPolicy::default() } }
}

#[derive(Debug, Clone)]
pub struct SecurityConfig {
    pub navigation: NavigationPolicy,
}

impl Default for SecurityConfig { fn default() -> Self { Self { navigation: NavigationPolicy::default() } } }

#[derive(Debug, Clone)]
pub struct FrontendDevConfig {
    pub url: String,
    pub command: Vec<String>,
    pub ready_path: String,
    pub timeout_ms: u32,
}

#[derive(Debug, Clone)]
pub struct FrontendConfig {
    pub dist: String,
    pub entry: String,
    pub spa_fallback: bool,
    pub dev: Option<FrontendDevConfig>,
}

impl Default for FrontendConfig { fn default() -> Self { Self { dist: "dist".into(), entry: "index.html".into(), spa_fallback: true, dev: None } } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowRestorePolicy { ClampToVisibleScreen, CenterOnPrimary }

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
    pub restore_policy: WindowRestorePolicy,
}

impl Default for Window {
    fn default() -> Self {
        Self { label: "main".into(), title: None, width: 720.0, height: 480.0, x: None, y: None, resizable: true, restore_state: true, restore_policy: WindowRestorePolicy::ClampToVisibleScreen }
    }
}

#[derive(Debug, Clone)]
pub struct PackageMetadata {
    pub kind: PackageKind,
    pub web_engine: WebEngine,
    pub license: Option<String>,
    pub authors: Vec<String>,
    pub repository: Option<String>,
    pub keywords: Vec<String>,
}

impl Default for PackageMetadata { fn default() -> Self { Self { kind: PackageKind::App, web_engine: WebEngine::System, license: None, authors: Vec::new(), repository: None, keywords: Vec::new() } } }

#[derive(Debug, Clone)]
pub struct UpdateConfig {
    pub feed_url: Option<String>,
    pub public_key: Option<String>,
    pub check_on_start: bool,
}

impl Default for UpdateConfig { fn default() -> Self { Self { feed_url: None, public_key: None, check_on_start: false } } }

#[derive(Debug, Clone)]
pub struct CefConfig {
    pub dir: String,
    pub auto_install: bool,
}

impl Default for CefConfig { fn default() -> Self { Self { dir: "third_party/cef/macos".into(), auto_install: false } } }

#[derive(Debug, Clone)]
pub struct Manifest {
    pub identity: AppIdentity,
    pub version: Version,
    pub icons: Vec<Icon>,
    pub permissions: Vec<Permission>,
    pub capabilities: Vec<Capability>,
    pub bridge: BridgeConfig,
    pub frontend: Option<FrontendConfig>,
    pub security: SecurityConfig,
    pub platforms: Vec<PlatformSettings>,
    pub windows: Vec<Window>,
    pub cef: CefConfig,
    pub package: PackageMetadata,
    pub updates: UpdateConfig,
}

pub fn validate_manifest(manifest: &Manifest) -> Result<(), ValidationError> {
    validate_identity(&manifest.identity)?;
    validate_version(&manifest.version)?;
    validate_icons(&manifest.icons)?;
    validate_permissions(&manifest.permissions)?;
    validate_capabilities(&manifest.capabilities)?;
    validate_bridge(&manifest.bridge)?;
    if let Some(ref frontend) = manifest.frontend { validate_frontend(frontend)?; }
    validate_security(&manifest.security)?;
    validate_platforms(&manifest.platforms)?;
    validate_windows(&manifest.windows)?;
    validate_package_metadata(&manifest.package)?;
    validate_updates(&manifest.updates)?;
    Ok(())
}

pub fn validate_identity(identity: &AppIdentity) -> Result<(), ValidationError> {
    validate_app_id(&identity.id, AppIdMode::ReverseDns)?;
    validate_name(&identity.name)?;
    if let Some(ref dn) = identity.display_name { validate_name(dn)?; }
    if let Some(ref org) = identity.organization { validate_name(org)?; }
    if let Some(ref hp) = identity.homepage { validate_url(hp)?; }
    Ok(())
}

pub fn validate_version(version: &Version) -> Result<(), ValidationError> {
    if let Some(ref pre) = version.pre { validate_version_part(pre)?; }
    if let Some(ref build) = version.build { validate_version_part(build)?; }
    Ok(())
}

pub fn validate_app_id(id: &str, mode: AppIdMode) -> Result<(), ValidationError> {
    if id.is_empty() || id.starts_with('.') || id.ends_with('.') { return Err(ValidationError::InvalidId); }
    let mut segments = 0usize;
    for part in id.split('.') {
        if part.is_empty() { return Err(ValidationError::InvalidId); }
        if part.starts_with('-') || part.ends_with('-') { return Err(ValidationError::InvalidId); }
        for ch in part.chars() {
            if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() && ch != '-' && ch != '_' { return Err(ValidationError::InvalidId); }
        }
        segments += 1;
    }
    if mode == AppIdMode::ReverseDns && segments < 2 { return Err(ValidationError::InvalidId); }
    Ok(())
}

pub fn validate_name(name: &str) -> Result<(), ValidationError> {
    if name.is_empty() || name == "." || name == ".." { return Err(ValidationError::InvalidName); }
    if name.contains('\0') || name.contains('/') || name.contains('\\') { return Err(ValidationError::InvalidName); }
    Ok(())
}

pub fn validate_url(url: &str) -> Result<(), ValidationError> {
    let prefix_len = if url.starts_with("https://") { 8 } else if url.starts_with("http://") { 7 } else { return Err(ValidationError::InvalidUrl); };
    if url.len() == prefix_len { return Err(ValidationError::InvalidUrl); }
    let rest = &url[prefix_len..];
    let slash_idx = rest.find('/').unwrap_or(rest.len());
    let host = &rest[..slash_idx];
    if host.is_empty() { return Err(ValidationError::InvalidUrl); }
    if host.contains('\0') || host.contains(' ') || host.contains('\t') || host.contains('\n') || host.contains('\r') { return Err(ValidationError::InvalidUrl); }
    Ok(())
}

pub fn validate_icons(icons: &[Icon]) -> Result<(), ValidationError> {
    for (i, icon) in icons.iter().enumerate() {
        if icon.asset.is_empty() { return Err(ValidationError::MissingRequiredField); }
        if icon.size == 0 || icon.scale == 0 { return Err(ValidationError::InvalidVersion); }
        for prev in &icons[..i] {
            if prev.size == icon.size && prev.scale == icon.scale && prev.purpose == icon.purpose { return Err(ValidationError::DuplicateIcon); }
        }
    }
    Ok(())
}

pub fn validate_permissions(permissions: &[Permission]) -> Result<(), ValidationError> {
    for (i, perm) in permissions.iter().enumerate() {
        if perm.kind == PermissionKind::Custom {
            if let Some(ref name) = perm.custom_name { validate_name(name)?; }
        }
        for prev in &permissions[..i] {
            if permission_eql(prev, perm) { return Err(ValidationError::DuplicatePermission); }
        }
    }
    Ok(())
}

pub fn validate_capabilities(capabilities: &[Capability]) -> Result<(), ValidationError> {
    for (i, cap) in capabilities.iter().enumerate() {
        if cap.kind == CapabilityKind::Custom {
            if let Some(ref name) = cap.custom_name { validate_name(name)?; }
        }
        for prev in &capabilities[..i] {
            if prev.kind == cap.kind {
                if cap.kind != CapabilityKind::Custom { return Err(ValidationError::DuplicateCapability); }
                if prev.custom_name == cap.custom_name { return Err(ValidationError::DuplicateCapability); }
            }
        }
    }
    Ok(())
}

pub fn validate_bridge(bridge: &BridgeConfig) -> Result<(), ValidationError> {
    for (i, cmd) in bridge.commands.iter().enumerate() {
        validate_name(&cmd.name)?;
        validate_permissions(&cmd.permissions)?;
        for origin in &cmd.origins { validate_bridge_origin(origin)?; }
        for prev in &bridge.commands[..i] {
            if prev.name == cmd.name { return Err(ValidationError::DuplicateBridgeCommand); }
        }
    }
    Ok(())
}

pub fn validate_frontend(frontend: &FrontendConfig) -> Result<(), ValidationError> {
    validate_relative_path(&frontend.dist)?;
    validate_relative_path(&frontend.entry)?;
    if let Some(ref dev) = frontend.dev {
        validate_url(&dev.url)?;
        if dev.command.is_empty() { return Err(ValidationError::MissingRequiredField); }
        for arg in &dev.command {
            if arg.is_empty() { return Err(ValidationError::InvalidCommand); }
            if arg.contains('\0') { return Err(ValidationError::InvalidCommand); }
        }
        validate_ready_path(&dev.ready_path)?;
        if dev.timeout_ms == 0 { return Err(ValidationError::InvalidTimeout); }
    }
    Ok(())
}

pub fn validate_bridge_origin(origin: &str) -> Result<(), ValidationError> {
    if origin == "*" { return Ok(()); }
    if origin.starts_with("http://") || origin.starts_with("https://") { return validate_url(origin); }
    if origin.starts_with("file://") || origin.starts_with("zero://") {
        let scheme_end = origin.find("://").unwrap() + 3;
        let value = &origin[scheme_end..];
        if value.is_empty() { return Err(ValidationError::InvalidUrl); }
        if value.contains('\0') || value.contains(' ') || value.contains('\t') { return Err(ValidationError::InvalidUrl); }
        return Ok(());
    }
    Err(ValidationError::InvalidUrl)
}

pub fn validate_security(security: &SecurityConfig) -> Result<(), ValidationError> {
    for origin in &security.navigation.allowed_origins { validate_bridge_origin(origin)?; }
    for url in &security.navigation.external_links.allowed_urls { validate_external_url_pattern(url)?; }
    Ok(())
}

pub fn validate_platforms(platforms: &[PlatformSettings]) -> Result<(), ValidationError> {
    for (i, settings) in platforms.iter().enumerate() {
        if settings.platform == Platform::Unknown { return Err(ValidationError::MissingRequiredField); }
        if let Some(ref id) = settings.id_override { validate_app_id(id, AppIdMode::ReverseDns)?; }
        if let Some(ref ver) = settings.min_os_version { validate_version_part(ver)?; }
        validate_permissions(&settings.permissions)?;
        if let Some(ref cat) = settings.category { validate_name(cat)?; }
        for prev in &platforms[..i] {
            if prev.platform == settings.platform { return Err(ValidationError::DuplicatePlatform); }
        }
    }
    Ok(())
}

pub fn validate_windows(windows: &[Window]) -> Result<(), ValidationError> {
    for (i, w) in windows.iter().enumerate() {
        if w.label.is_empty() { return Err(ValidationError::InvalidName); }
        if w.width <= 0.0 || w.height <= 0.0 { return Err(ValidationError::InvalidDimension); }
        for prev in &windows[..i] {
            if prev.label == w.label { return Err(ValidationError::DuplicateWindow); }
        }
    }
    Ok(())
}

pub fn validate_package_metadata(metadata: &PackageMetadata) -> Result<(), ValidationError> {
    if let Some(ref lic) = metadata.license { validate_name(lic)?; }
    if let Some(ref repo) = metadata.repository { validate_url(repo)?; }
    for author in &metadata.authors {
        if author.is_empty() { return Err(ValidationError::MissingRequiredField); }
        if author.contains('\0') { return Err(ValidationError::InvalidName); }
    }
    for kw in &metadata.keywords { validate_keyword(kw)?; }
    Ok(())
}

pub fn validate_updates(updates: &UpdateConfig) -> Result<(), ValidationError> {
    if let Some(ref url) = updates.feed_url { validate_external_url_pattern(url)?; }
    if let Some(ref key) = updates.public_key { if key.is_empty() { return Err(ValidationError::MissingRequiredField); } }
    Ok(())
}

fn validate_external_url_pattern(url: &str) -> Result<(), ValidationError> {
    if url == "*" { return Ok(()); }
    if url.ends_with('*') {
        let prefix = &url[..url.len() - 1];
        if prefix.is_empty() { return Err(ValidationError::InvalidUrl); }
        if prefix.contains('\0') || prefix.contains(' ') || prefix.contains('\t') { return Err(ValidationError::InvalidUrl); }
        if prefix.starts_with("http://") || prefix.starts_with("https://") { return Ok(()); }
        return Err(ValidationError::InvalidUrl);
    }
    validate_url(url)
}

fn validate_relative_path(path: &str) -> Result<(), ValidationError> {
    if path.is_empty() { return Err(ValidationError::InvalidPath); }
    if path.starts_with('/') { return Err(ValidationError::InvalidPath); }
    if path.contains('\\') { return Err(ValidationError::InvalidPath); }
    if path.contains('\0') { return Err(ValidationError::InvalidPath); }
    for segment in path.split('/') {
        if segment.is_empty() || segment == "." || segment == ".." { return Err(ValidationError::InvalidPath); }
    }
    Ok(())
}

fn validate_ready_path(path: &str) -> Result<(), ValidationError> {
    if path.is_empty() || !path.starts_with('/') { return Err(ValidationError::InvalidPath); }
    if path.contains('\0') || path.contains(' ') || path.contains('\t') { return Err(ValidationError::InvalidPath); }
    Ok(())
}

fn validate_version_part(part: &str) -> Result<(), ValidationError> {
    if part.is_empty() { return Err(ValidationError::InvalidVersion); }
    for ch in part.chars() {
        if !ch.is_ascii_alphanumeric() && ch != '-' && ch != '.' { return Err(ValidationError::InvalidVersion); }
    }
    Ok(())
}

fn validate_keyword(keyword: &str) -> Result<(), ValidationError> {
    if keyword.is_empty() { return Err(ValidationError::InvalidKeyword); }
    for ch in keyword.chars() {
        if !ch.is_ascii_lowercase() && !ch.is_ascii_digit() && ch != '-' && ch != '_' { return Err(ValidationError::InvalidKeyword); }
    }
    Ok(())
}

fn permission_eql(a: &Permission, b: &Permission) -> bool {
    if a.kind != b.kind { return false; }
    if a.kind == PermissionKind::Custom { a.custom_name == b.custom_name } else { true }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_minimal_manifest() {
        let manifest = Manifest {
            identity: AppIdentity { id: "com.example.app".into(), name: "example".into(), display_name: None, organization: None, homepage: None },
            version: Version { major: 1, minor: 0, patch: 0, pre: None, build: None },
            icons: vec![], permissions: vec![], capabilities: vec![], bridge: BridgeConfig::default(),
            frontend: None, security: SecurityConfig::default(), platforms: vec![], windows: vec![],
            cef: CefConfig::default(), package: PackageMetadata::default(), updates: UpdateConfig::default(),
        };
        assert!(validate_manifest(&manifest).is_ok());
    }

    #[test]
    fn valid_app_id() {
        assert!(validate_app_id("com.example.app", AppIdMode::ReverseDns).is_ok());
        assert!(validate_app_id("my-tool", AppIdMode::Simple).is_ok());
        assert!(validate_app_id("example", AppIdMode::ReverseDns).is_err());
        assert!(validate_app_id("Com.example.app", AppIdMode::ReverseDns).is_err());
        assert!(validate_app_id("", AppIdMode::ReverseDns).is_err());
        assert!(validate_app_id("com/example/app", AppIdMode::ReverseDns).is_err());
    }

    #[test]
    fn version_string() {
        let v = Version { major: 1, minor: 2, patch: 3, pre: Some("beta.1".into()), build: None };
        assert_eq!("1.2.3-beta.1", v.to_string_val());
        let v2 = Version { major: 1, minor: 0, patch: 0, pre: None, build: None };
        assert_eq!("1.0.0", v2.to_string_val());
        let v3 = Version { major: 1, minor: 2, patch: 3, pre: Some("beta.1".into()), build: Some("20260506".into()) };
        assert_eq!("1.2.3-beta.1+20260506", v3.to_string_val());
    }

    #[test]
    fn name_validation() {
        assert!(validate_name("Example App").is_ok());
        assert!(validate_name("Apache-2.0").is_ok());
        assert!(validate_name("").is_err());
        assert!(validate_name(".").is_err());
        assert!(validate_name("bad/name").is_err());
    }

    #[test]
    fn url_validation() {
        assert!(validate_url("https://example.com").is_ok());
        assert!(validate_url("http://example.com/path").is_ok());
        assert!(validate_url("ftp://example.com").is_err());
        assert!(validate_url("https://").is_err());
    }

    #[test]
    fn windows_validation() {
        let good = vec![Window { label: "main".into(), ..Default::default() }];
        assert!(validate_windows(&good).is_ok());
        let empty_label = vec![Window { label: "".into(), ..Default::default() }];
        assert!(validate_windows(&empty_label).is_err());
        let dup = vec![Window { label: "main".into(), ..Default::default() }, Window { label: "main".into(), ..Default::default() }];
        assert!(validate_windows(&dup).is_err());
    }

    #[test]
    fn icon_validation() {
        let good = vec![Icon { asset: "icons/app".into(), size: 128, scale: 1, purpose: Some(IconPurpose::Any) }];
        assert!(validate_icons(&good).is_ok());
        let empty = vec![Icon { asset: "".into(), size: 128, scale: 1, purpose: None }];
        assert!(validate_icons(&empty).is_err());
        let zero_size = vec![Icon { asset: "icons/app".into(), size: 0, scale: 1, purpose: None }];
        assert!(validate_icons(&zero_size).is_err());
    }

    #[test]
    fn permission_validation() {
        let good = vec![Permission::built_in(PermissionKind::Network), Permission::built_in(PermissionKind::Clipboard), Permission::custom("com.example.custom")];
        assert!(validate_permissions(&good).is_ok());
        let dup = vec![Permission::built_in(PermissionKind::Network), Permission::built_in(PermissionKind::Network)];
        assert!(validate_permissions(&dup).is_err());
    }

    #[test]
    fn capability_validation() {
        let good = vec![Capability::built_in(CapabilityKind::NativeModule), Capability::custom("com.example.native-camera")];
        assert!(validate_capabilities(&good).is_ok());
        let dup = vec![Capability::built_in(CapabilityKind::Webview), Capability::built_in(CapabilityKind::Webview)];
        assert!(validate_capabilities(&dup).is_err());
    }

    #[test]
    fn bridge_validation() {
        let good = BridgeConfig { commands: vec![BridgeCommand { name: "native.ping".into(), permissions: vec![], origins: vec!["zero://inline".into()] }] };
        assert!(validate_bridge(&good).is_ok());
        let dup = BridgeConfig { commands: vec![BridgeCommand { name: "native.ping".into(), permissions: vec![], origins: vec![] }, BridgeCommand { name: "native.ping".into(), permissions: vec![], origins: vec![] }] };
        assert!(validate_bridge(&dup).is_err());
    }

    #[test]
    fn frontend_validation() {
        let good = FrontendConfig { dist: "dist".into(), entry: "index.html".into(), spa_fallback: true, dev: Some(FrontendDevConfig { url: "http://127.0.0.1:5173/".into(), command: vec!["npm".into()], ready_path: "/".into(), timeout_ms: 30000 }) };
        assert!(validate_frontend(&good).is_ok());
        let bad_path = FrontendConfig { dist: "../dist".into(), ..Default::default() };
        assert!(validate_frontend(&bad_path).is_err());
    }

    #[test]
    fn platform_validation() {
        let good = vec![PlatformSettings { platform: Platform::MacOS, id_override: Some("com.example.app.macos".into()), ..Default::default() }, PlatformSettings { platform: Platform::Linux, ..Default::default() }];
        assert!(validate_platforms(&good).is_ok());
        let unknown = vec![PlatformSettings { platform: Platform::Unknown, ..Default::default() }];
        assert!(validate_platforms(&unknown).is_err());
        let dup = vec![PlatformSettings { platform: Platform::MacOS, ..Default::default() }, PlatformSettings { platform: Platform::MacOS, ..Default::default() }];
        assert!(validate_platforms(&dup).is_err());
    }

    #[test]
    fn package_metadata_validation() {
        let good = PackageMetadata { kind: PackageKind::Cli, license: Some("Apache-2.0".into()), authors: vec!["Example".into()], repository: Some("https://example.com/repo".into()), keywords: vec!["zig".into()], ..Default::default() };
        assert!(validate_package_metadata(&good).is_ok());
        let empty_author = PackageMetadata { authors: vec!["".into()], ..Default::default() };
        assert!(validate_package_metadata(&empty_author).is_err());
    }

    #[test]
    fn cef_config_default() {
        let cef = CefConfig::default();
        assert_eq!("third_party/cef/macos", cef.dir);
        assert!(!cef.auto_install);
    }

    #[test]
    fn bridge_origin_validation() {
        assert!(validate_bridge_origin("*").is_ok());
        assert!(validate_bridge_origin("zero://inline").is_ok());
        assert!(validate_bridge_origin("https://example.com").is_ok());
        assert!(validate_bridge_origin("bad origin").is_err());
    }
}
