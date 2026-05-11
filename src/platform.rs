use crate::geometry::{RectF, SizeF};
use crate::security;

pub type WindowId = u64;
pub const MAX_WINDOWS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebEngine {
    System,
    Chromium,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebViewSourceKind {
    Html,
    Url,
    Assets,
}

#[derive(Debug, Clone)]
pub struct WebViewAssetSource {
    pub root_path: String,
    pub entry: String,
    pub origin: String,
    pub spa_fallback: bool,
}

impl Default for WebViewAssetSource {
    fn default() -> Self {
        Self {
            root_path: String::new(),
            entry: String::new(),
            origin: String::new(),
            spa_fallback: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WebViewSource {
    pub kind: WebViewSourceKind,
    pub bytes: String,
    pub asset_options: Option<WebViewAssetSource>,
}

impl WebViewSource {
    pub fn html(html: &str) -> Self {
        Self {
            kind: WebViewSourceKind::Html,
            bytes: html.to_string(),
            asset_options: None,
        }
    }

    pub fn url(url: &str) -> Self {
        Self {
            kind: WebViewSourceKind::Url,
            bytes: url.to_string(),
            asset_options: None,
        }
    }

    pub fn assets(options: WebViewAssetSource) -> Self {
        Self {
            kind: WebViewSourceKind::Assets,
            bytes: options.origin.clone(),
            asset_options: Some(options),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowRestorePolicy {
    ClampToVisibleScreen,
    CenterOnPrimary,
}

#[derive(Debug, Clone)]
pub struct WindowOptions {
    pub id: WindowId,
    pub label: String,
    pub title: String,
    pub default_frame: RectF,
    pub resizable: bool,
    pub restore_state: bool,
    pub restore_policy: WindowRestorePolicy,
}

impl Default for WindowOptions {
    fn default() -> Self {
        Self {
            id: 1,
            label: "main".into(),
            title: String::new(),
            default_frame: RectF::new(0.0, 0.0, 720.0, 480.0),
            resizable: true,
            restore_state: true,
            restore_policy: WindowRestorePolicy::ClampToVisibleScreen,
        }
    }
}

impl WindowOptions {
    pub fn resolved_title<'a>(&'a self, app_name: &'a str) -> &'a str {
        if self.title.is_empty() { app_name } else { &self.title }
    }
}

#[derive(Debug, Clone)]
pub struct WindowState {
    pub id: WindowId,
    pub label: String,
    pub title: String,
    pub frame: RectF,
    pub scale_factor: f32,
    pub open: bool,
    pub focused: bool,
    pub maximized: bool,
    pub fullscreen: bool,
}

impl Default for WindowState {
    fn default() -> Self {
        Self {
            id: 1,
            label: "main".into(),
            title: String::new(),
            frame: RectF::new(0.0, 0.0, 720.0, 480.0),
            scale_factor: 1.0,
            open: true,
            focused: true,
            maximized: false,
            fullscreen: false,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: WindowId,
    pub label: String,
    pub title: String,
    pub frame: RectF,
    pub scale_factor: f32,
    pub open: bool,
    pub focused: bool,
}

impl Default for WindowInfo {
    fn default() -> Self {
        Self {
            id: 1,
            label: "main".into(),
            title: String::new(),
            frame: RectF::new(0.0, 0.0, 720.0, 480.0),
            scale_factor: 1.0,
            open: true,
            focused: false,
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct AppInfo {
    pub app_name: String,
    pub window_title: String,
    pub bundle_id: String,
    pub icon_path: String,
    pub main_window: WindowOptions,
    pub windows: Vec<WindowOptions>,
}

impl AppInfo {
    pub fn resolved_window_title(&self) -> &str {
        if !self.window_title.is_empty() {
            &self.window_title
        } else {
            self.main_window.resolved_title(&self.app_name)
        }
    }

    pub fn startup_window_count(&self) -> usize {
        if self.windows.is_empty() { 1 } else { self.windows.len() }
    }

    pub fn resolved_startup_window(&self, index: usize) -> WindowOptions {
        let mut window = if self.windows.is_empty() {
            self.main_window.clone()
        } else {
            self.windows[index].clone()
        };
        if window.id == 0 || (!self.windows.is_empty() && index > 0 && window.id == 1) {
            window.id = (index + 1) as u64;
        }
        if window.label.is_empty() {
            window.label = if index == 0 { "main".into() } else { "window".into() };
        }
        if window.title.is_empty() {
            window.title = self.resolved_window_title().to_string();
        }
        window
    }
}

#[derive(Debug, Clone)]
pub struct Surface {
    pub id: u64,
    pub size: SizeF,
    pub scale_factor: f32,
}

impl Default for Surface {
    fn default() -> Self {
        Self {
            id: 1,
            size: SizeF::new(640.0, 360.0),
            scale_factor: 1.0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BridgeMessage {
    pub bytes: String,
    pub origin: String,
    pub window_id: WindowId,
}

pub type TrayItemId = u32;

#[derive(Debug, Clone)]
pub enum Event {
    AppStart,
    FrameRequested,
    AppShutdown,
    SurfaceResized(Surface),
    WindowFrameChanged(WindowState),
    WindowFocused(WindowId),
    BridgeMessage(BridgeMessage),
    TrayAction(TrayItemId),
}

impl Event {
    pub fn name(&self) -> &'static str {
        match self {
            Self::AppStart => "app_start",
            Self::FrameRequested => "frame_requested",
            Self::AppShutdown => "app_shutdown",
            Self::SurfaceResized(_) => "surface_resized",
            Self::WindowFrameChanged(_) => "window_frame_changed",
            Self::WindowFocused(_) => "window_focused",
            Self::BridgeMessage(_) => "bridge_message",
            Self::TrayAction(_) => "tray_action",
        }
    }
}

#[derive(Debug, Clone)]
pub struct NullPlatform {
    pub surface_value: Surface,
    pub web_engine: WebEngine,
    pub app_info: AppInfo,
    pub requested_frames: u32,
    pub loaded_source: Option<WebViewSource>,
    windows: Vec<WindowInfo>,
    bridge_response: Vec<u8>,
    bridge_response_window_id: WindowId,
}

impl NullPlatform {
    pub fn new(surface: Surface) -> Self {
        Self {
            surface_value: surface,
            web_engine: WebEngine::System,
            app_info: AppInfo::default(),
            requested_frames: 1,
            loaded_source: None,
            windows: Vec::new(),
            bridge_response: Vec::new(),
            bridge_response_window_id: 0,
        }
    }

    pub fn with_engine(surface: Surface, engine: WebEngine) -> Self {
        Self { web_engine: engine, ..Self::new(surface) }
    }

    pub fn last_bridge_response(&self) -> &[u8] {
        &self.bridge_response
    }

    pub fn last_bridge_response_window_id(&self) -> WindowId {
        self.bridge_response_window_id
    }

    pub fn record_bridge_response(&mut self, window_id: WindowId, response: &[u8]) {
        self.bridge_response.clear();
        self.bridge_response.extend_from_slice(response);
        self.bridge_response_window_id = window_id;
    }
}
