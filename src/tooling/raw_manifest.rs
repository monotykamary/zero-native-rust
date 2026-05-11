use crate::tooling::web_engine;

#[derive(Debug, Clone, Default)]
pub struct RawManifest {
    pub id: String,
    pub name: String,
    pub display_name: Option<String>,
    pub version: String,
    pub icons: Vec<String>,
    pub platforms: Vec<String>,
    pub permissions: Vec<String>,
    pub capabilities: Vec<String>,
    pub bridge: RawBridge,
    pub web_engine: String,
    pub cef: RawCef,
    pub frontend: Option<RawFrontend>,
    pub security: RawSecurity,
    pub windows: Vec<RawWindow>,
}

impl RawManifest {
    pub fn default_web_engine() -> String {
        "system".to_string()
    }
}

#[derive(Debug, Clone, Default)]
pub struct RawCef {
    pub dir: String,
    pub auto_install: bool,
}

impl RawCef {
    pub fn default_dir() -> String { ".cef".to_string() }
}

#[derive(Debug, Clone, Default)]
pub struct RawBridge {
    pub commands: Vec<RawBridgeCommand>,
}

#[derive(Debug, Clone)]
pub struct RawBridgeCommand {
    pub name: String,
    pub permissions: Vec<String>,
    pub origins: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RawFrontend {
    pub dist: String,
    pub entry: String,
    pub spa_fallback: bool,
    pub dev: Option<RawFrontendDev>,
}

impl Default for RawFrontend {
    fn default() -> Self {
        Self { dist: "dist".into(), entry: "index.html".into(), spa_fallback: true, dev: None }
    }
}

#[derive(Debug, Clone)]
pub struct RawFrontendDev {
    pub url: String,
    pub command: Vec<String>,
    pub ready_path: String,
    pub timeout_ms: u32,
}

impl Default for RawFrontendDev {
    fn default() -> Self {
        Self { url: String::new(), command: Vec::new(), ready_path: "/".into(), timeout_ms: 30_000 }
    }
}

#[derive(Debug, Clone, Default)]
pub struct RawSecurity {
    pub navigation: RawNavigation,
}

#[derive(Debug, Clone, Default)]
pub struct RawNavigation {
    pub allowed_origins: Vec<String>,
    pub external_links: RawExternalLinks,
}

#[derive(Debug, Clone, Default)]
pub struct RawExternalLinks {
    pub action: String,
    pub allowed_urls: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct RawWindow {
    pub label: String,
    pub title: Option<String>,
    pub width: f32,
    pub height: f32,
    pub x: Option<f32>,
    pub y: Option<f32>,
    pub restore_state: bool,
}

impl Default for RawWindow {
    fn default() -> Self {
        Self { label: "main".into(), title: None, width: 720.0, height: 480.0, x: None, y: None, restore_state: true }
    }
}
