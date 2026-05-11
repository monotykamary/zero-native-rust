use crate::geometry::{RectF, SizeF};
use crate::security;

pub type WindowId = u64;
pub const MAX_WINDOWS: usize = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebEngine { System, Chromium }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WebViewSourceKind { Html, Url, Assets }

#[derive(Debug, Clone, Default)]
pub struct WebViewAssetSource {
    pub root_path: String, pub entry: String, pub origin: String, pub spa_fallback: bool,
}

#[derive(Debug, Clone)]
pub struct WebViewSource {
    pub kind: WebViewSourceKind,
    pub bytes: String,
    pub asset_options: Option<WebViewAssetSource>,
}

impl WebViewSource {
    pub fn html(html: &str) -> Self { Self { kind: WebViewSourceKind::Html, bytes: html.to_string(), asset_options: None } }
    pub fn url(url: &str) -> Self { Self { kind: WebViewSourceKind::Url, bytes: url.to_string(), asset_options: None } }
    pub fn assets(options: WebViewAssetSource) -> Self { Self { kind: WebViewSourceKind::Assets, bytes: options.origin.clone(), asset_options: Some(options) } }
    pub fn kind_name(&self) -> &'static str { match self.kind { WebViewSourceKind::Html => "html", WebViewSourceKind::Url => "url", WebViewSourceKind::Assets => "assets" } }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WindowRestorePolicy { ClampToVisibleScreen, CenterOnPrimary }

#[derive(Debug, Clone)]
pub struct WindowOptions {
    pub id: WindowId, pub label: String, pub title: String,
    pub default_frame: RectF, pub resizable: bool, pub restore_state: bool, pub restore_policy: WindowRestorePolicy,
}

impl Default for WindowOptions {
    fn default() -> Self { Self { id: 1, label: "main".into(), title: String::new(), default_frame: RectF::new(0.0, 0.0, 720.0, 480.0), resizable: true, restore_state: true, restore_policy: WindowRestorePolicy::ClampToVisibleScreen } }
}

impl WindowOptions {
    pub fn resolved_title<'a>(&'a self, app_name: &'a str) -> &'a str { if self.title.is_empty() { app_name } else { &self.title } }
}

#[derive(Debug, Clone)]
pub struct WindowCreateOptions {
    pub id: WindowId, pub label: String, pub title: String,
    pub default_frame: RectF, pub resizable: bool, pub restore_state: bool,
    pub restore_policy: WindowRestorePolicy, pub source: Option<WebViewSource>,
}

impl Default for WindowCreateOptions {
    fn default() -> Self { Self { id: 0, label: String::new(), title: String::new(), default_frame: RectF::new(0.0, 0.0, 720.0, 480.0), resizable: true, restore_state: true, restore_policy: WindowRestorePolicy::ClampToVisibleScreen, source: None } }
}

impl WindowCreateOptions {
    pub fn window_options(&self, id: WindowId, label: &str) -> WindowOptions {
        WindowOptions { id, label: label.to_string(), title: self.title.clone(), default_frame: self.default_frame, resizable: self.resizable, restore_state: self.restore_state, restore_policy: self.restore_policy }
    }
}

#[derive(Debug, Clone)]
pub struct WindowState {
    pub id: WindowId, pub label: String, pub title: String, pub frame: RectF, pub scale_factor: f32,
    pub open: bool, pub focused: bool, pub maximized: bool, pub fullscreen: bool,
}

impl Default for WindowState {
    fn default() -> Self { Self { id: 1, label: "main".into(), title: String::new(), frame: RectF::new(0.0, 0.0, 720.0, 480.0), scale_factor: 1.0, open: true, focused: true, maximized: false, fullscreen: false } }
}

#[derive(Debug, Clone)]
pub struct WindowInfo {
    pub id: WindowId, pub label: String, pub title: String, pub frame: RectF,
    pub scale_factor: f32, pub open: bool, pub focused: bool,
}

impl Default for WindowInfo {
    fn default() -> Self { Self { id: 1, label: "main".into(), title: String::new(), frame: RectF::new(0.0, 0.0, 720.0, 480.0), scale_factor: 1.0, open: true, focused: false } }
}

#[derive(Debug, Clone, Default)]
pub struct AppInfo {
    pub app_name: String, pub window_title: String, pub bundle_id: String,
    pub icon_path: String, pub main_window: WindowOptions, pub windows: Vec<WindowOptions>,
}

impl AppInfo {
    pub fn resolved_window_title(&self) -> &str {
        if !self.window_title.is_empty() { &self.window_title } else { self.main_window.resolved_title(&self.app_name) }
    }
    pub fn resolved_main_window(&self) -> WindowOptions {
        let mut w = self.main_window.clone();
        if w.title.is_empty() { w.title = self.resolved_window_title().to_string(); }
        w
    }
    pub fn startup_window_count(&self) -> usize { if self.windows.is_empty() { 1 } else { self.windows.len() } }
    pub fn resolved_startup_window(&self, index: usize) -> WindowOptions {
        let mut w = if self.windows.is_empty() { self.main_window.clone() } else { self.windows[index].clone() };
        if w.id == 0 || (!self.windows.is_empty() && index > 0 && w.id == 1) { w.id = (index + 1) as u64; }
        if w.label.is_empty() { w.label = if index == 0 { "main".into() } else { "window".into() }; }
        if w.title.is_empty() { w.title = self.resolved_window_title().to_string(); }
        w
    }
}

#[derive(Debug, Clone)]
pub struct Surface { pub id: u64, pub size: SizeF, pub scale_factor: f32 }
impl Default for Surface { fn default() -> Self { Self { id: 1, size: SizeF::new(640.0, 360.0), scale_factor: 1.0 } } }

#[derive(Debug, Clone)]
pub struct BridgeMessage { pub bytes: String, pub origin: String, pub window_id: WindowId }

#[derive(Debug, Clone)]
pub enum Event {
    AppStart, FrameRequested, AppShutdown,
    SurfaceResized(Surface), WindowFrameChanged(WindowState),
    WindowFocused(WindowId), BridgeMessage(BridgeMessage), TrayAction(u32),
}

impl Event {
    pub fn name(&self) -> &'static str {
        match self {
            Self::AppStart => "app_start", Self::FrameRequested => "frame_requested",
            Self::AppShutdown => "app_shutdown", Self::SurfaceResized(_) => "surface_resized",
            Self::WindowFrameChanged(_) => "window_frame_changed", Self::WindowFocused(_) => "window_focused",
            Self::BridgeMessage(_) => "bridge_message", Self::TrayAction(_) => "tray_action",
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
    bridge_response: Vec<u8>,
    bridge_response_window_id: WindowId,
}

impl NullPlatform {
    pub fn new(surface: Surface) -> Self {
        Self { surface_value: surface, web_engine: WebEngine::System, app_info: AppInfo::default(),
            requested_frames: 1, loaded_source: None, bridge_response: Vec::new(), bridge_response_window_id: 0 }
    }
    pub fn with_engine(surface: Surface, engine: WebEngine) -> Self { Self { web_engine: engine, ..Self::new(surface) } }
    pub fn last_bridge_response(&self) -> &str { std::str::from_utf8(&self.bridge_response).unwrap_or("") }
    pub fn last_bridge_response_window_id(&self) -> WindowId { self.bridge_response_window_id }
    pub fn record_bridge_response(&mut self, window_id: WindowId, response: &[u8]) {
        self.bridge_response.clear(); self.bridge_response.extend_from_slice(response); self.bridge_response_window_id = window_id;
    }

    pub fn run_event_loop(&mut self, handler: &mut dyn FnMut(Event)) {
        handler(Event::AppStart);
        handler(Event::SurfaceResized(self.surface_value.clone()));
        let count = self.app_info.startup_window_count();
        for index in 0..count {
            let window = self.app_info.resolved_startup_window(index);
            handler(Event::WindowFrameChanged(WindowState {
                id: window.id, label: window.label.clone(),
                title: window.resolved_title(&self.app_info.app_name).to_string(),
                frame: window.default_frame, scale_factor: self.surface_value.scale_factor,
                open: true, focused: index == 0, maximized: false, fullscreen: false,
            }));
        }
        for _ in 0..self.requested_frames { handler(Event::FrameRequested); }
        handler(Event::AppShutdown);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_platform_emits_lifecycle_events() {
        let mut np = NullPlatform::new(Surface::default());
        let mut events = Vec::new();
        np.run_event_loop(&mut |e| events.push(e.name().to_string()));
        assert!(events.contains(&"app_start".to_string()));
        assert!(events.contains(&"surface_resized".to_string()));
        assert!(events.contains(&"frame_requested".to_string()));
        assert!(events.contains(&"app_shutdown".to_string()));
    }

    #[test]
    fn null_platform_records_bridge_response() {
        let mut np = NullPlatform::new(Surface::default());
        np.record_bridge_response(7, b"{\"ok\":true}");
        assert_eq!(7, np.last_bridge_response_window_id());
        assert_eq!("{\"ok\":true}", np.last_bridge_response());
    }

    #[test]
    fn webview_source_constructors() {
        let html = WebViewSource::html("<h1>Hi</h1>");
        assert_eq!(WebViewSourceKind::Html, html.kind);
        let url = WebViewSource::url("http://localhost");
        assert_eq!(WebViewSourceKind::Url, url.kind);
        let assets = WebViewSource::assets(WebViewAssetSource { root_path: "dist".into(), entry: "index.html".into(), origin: "zero://app".into(), spa_fallback: true });
        assert_eq!(WebViewSourceKind::Assets, assets.kind);
    }

    #[test]
    fn app_info_startup_window() {
        let info = AppInfo { app_name: "test".into(), main_window: WindowOptions::default(), ..Default::default() };
        assert_eq!(1, info.startup_window_count());
        let w = info.resolved_startup_window(0);
        assert_eq!(1, w.id);
    }
}
