use crate::geometry::{RectF, SizeF};
use crate::security;

#[cfg(target_os = "macos")]
pub mod macos;
#[cfg(target_os = "linux")]
pub mod linux;
#[cfg(target_os = "windows")]
pub mod windows;

pub type WindowId = u64;
pub const MAX_WINDOWS: usize = 16;
pub const MAX_DIALOG_PATH_BYTES: usize = 4096;
pub const MAX_DIALOG_PATHS_BYTES: usize = 16 * 4096;

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

impl WindowInfo {
    pub fn state(&self) -> WindowState {
        WindowState { id: self.id, label: self.label.clone(), title: self.title.clone(), frame: self.frame, scale_factor: self.scale_factor, open: self.open, focused: self.focused, maximized: false, fullscreen: false }
    }
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
pub struct FileFilter { pub name: String, pub extensions: Vec<String> }

#[derive(Debug, Clone, Default)]
pub struct OpenDialogOptions {
    pub title: String, pub default_path: String, pub filters: Vec<FileFilter>,
    pub allow_directories: bool, pub allow_multiple: bool,
}

#[derive(Debug, Clone)]
pub struct OpenDialogResult { pub count: usize, pub paths: String }

#[derive(Debug, Clone, Default)]
pub struct SaveDialogOptions {
    pub title: String, pub default_path: String, pub default_name: String, pub filters: Vec<FileFilter>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MessageDialogStyle { Info = 0, Warning = 1, Critical = 2 }

impl Default for MessageDialogStyle { fn default() -> Self { Self::Info } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(i32)]
pub enum MessageDialogResult { Primary = 0, Secondary = 1, Tertiary = 2 }

impl Default for MessageDialogResult { fn default() -> Self { Self::Primary } }

#[derive(Debug, Clone, Default)]
pub struct MessageDialogOptions {
    pub style: MessageDialogStyle,
    pub title: String, pub message: String, pub informative_text: String,
    pub primary_button: String, pub secondary_button: String, pub tertiary_button: String,
}

pub type TrayItemId = u32;

#[derive(Debug, Clone, Default)]
pub struct TrayOptions { pub icon_path: String, pub tooltip: String, pub items: Vec<TrayMenuItem> }

#[derive(Debug, Clone)]
pub struct TrayMenuItem { pub id: TrayItemId, pub label: String, pub separator: bool, pub enabled: bool }

impl Default for TrayMenuItem { fn default() -> Self { Self { id: 0, label: String::new(), separator: false, enabled: true } } }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlatformError {
    UnsupportedService,
    WindowNotFound,
    WindowLimitReached,
    DuplicateWindowId,
    DuplicateWindowLabel,
    MissingWindowSource,
    WindowSourceTooLarge,
    FocusFailed,
    CloseFailed,
    CreateFailed,
    CallbackFailed,
}

impl std::fmt::Display for PlatformError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) }
}
impl std::error::Error for PlatformError {}

/// Safe abstraction over platform-native operations.
///
/// All FFI calls are isolated inside the trait implementations.
/// Users of `PlatformHost` never write `unsafe` — the trait
/// boundary is the safety membrane.
pub trait PlatformHost {
    fn app_info(&self) -> &AppInfo;
    fn surface(&self) -> Surface;
    fn set_surface(&mut self, surface: Surface);
    fn run(&mut self, handler: &mut dyn FnMut(Event));
    fn stop(&mut self);

    fn load_webview(&mut self, source: &WebViewSource);
    fn load_window_webview(&mut self, window_id: WindowId, source: &WebViewSource);
    fn complete_bridge(&mut self, response: &[u8]);
    fn complete_window_bridge(&mut self, window_id: WindowId, response: &[u8]);

    fn create_window(&mut self, options: &WindowOptions) -> Result<WindowInfo, PlatformError>;
    fn focus_window(&mut self, window_id: WindowId) -> Result<(), PlatformError>;
    fn close_window(&mut self, window_id: WindowId) -> Result<(), PlatformError>;

    fn show_open_dialog(&mut self, options: &OpenDialogOptions, buffer: &mut [u8]) -> Result<OpenDialogResult, PlatformError>;
    fn show_save_dialog(&mut self, options: &SaveDialogOptions, buffer: &mut [u8]) -> Result<Option<String>, PlatformError>;
    fn show_message_dialog(&mut self, options: &MessageDialogOptions) -> Result<MessageDialogResult, PlatformError>;

    fn create_tray(&mut self, options: &TrayOptions) -> Result<(), PlatformError>;
    fn update_tray_menu(&mut self, items: &[TrayMenuItem]) -> Result<(), PlatformError>;
    fn remove_tray(&mut self) -> Result<(), PlatformError>;

    fn read_clipboard(&mut self, buffer: &mut [u8]) -> Result<String, PlatformError>;
    fn write_clipboard(&mut self, text: &str) -> Result<(), PlatformError>;

    fn configure_security_policy(&mut self, policy: &security::Policy);
    fn emit_window_event(&mut self, window_id: WindowId, name: &str, detail_json: &str);

    fn box_clone(&self) -> Box<dyn PlatformHost>;

    fn as_any(&self) -> &dyn std::any::Any;
}

impl Clone for Box<dyn PlatformHost> {
    fn clone(&self) -> Self { self.box_clone() }
}

#[derive(Debug, Clone)]
pub struct NullPlatform {
    pub surface_value: Surface,
    pub web_engine: WebEngine,
    pub app_info_value: AppInfo,
    pub requested_frames: u32,
    pub loaded_source: Option<WebViewSource>,
    bridge_response: Vec<u8>,
    bridge_response_window_id: WindowId,
    security_policy: security::Policy,
    windows: Vec<WindowInfo>,
}

impl NullPlatform {
    pub fn new(surface: Surface) -> Self {
        Self { surface_value: surface, web_engine: WebEngine::System, app_info_value: AppInfo::default(),
            requested_frames: 1, loaded_source: None, bridge_response: Vec::new(), bridge_response_window_id: 0,
            security_policy: security::Policy::default(), windows: Vec::new() }
    }
    pub fn with_engine(surface: Surface, engine: WebEngine) -> Self { Self { web_engine: engine, ..Self::new(surface) } }
    pub fn with_options(surface: Surface, engine: WebEngine, app_info: AppInfo) -> Self {
        Self { app_info_value: app_info, web_engine: engine, ..Self::new(surface) }
    }
    pub fn last_bridge_response(&self) -> &str { std::str::from_utf8(&self.bridge_response).unwrap_or("") }
    pub fn last_bridge_response_window_id(&self) -> WindowId { self.bridge_response_window_id }
    pub fn record_bridge_response(&mut self, window_id: WindowId, response: &[u8]) {
        self.bridge_response.clear(); self.bridge_response.extend_from_slice(response); self.bridge_response_window_id = window_id;
    }

    fn find_window_index(&self, window_id: WindowId) -> Option<usize> {
        self.windows.iter().position(|w| w.id == window_id)
    }
}

impl PlatformHost for NullPlatform {
    fn app_info(&self) -> &AppInfo { &self.app_info_value }
    fn surface(&self) -> Surface { self.surface_value.clone() }
    fn set_surface(&mut self, surface: Surface) { self.surface_value = surface; }

    fn run(&mut self, handler: &mut dyn FnMut(Event)) {
        handler(Event::AppStart);
        handler(Event::SurfaceResized(self.surface_value.clone()));
        let count = self.app_info_value.startup_window_count();
        for index in 0..count {
            let window = self.app_info_value.resolved_startup_window(index);
            let info = WindowInfo {
                id: window.id, label: window.label.clone(),
                title: window.resolved_title(&self.app_info_value.app_name).to_string(),
                frame: window.default_frame, scale_factor: self.surface_value.scale_factor,
                open: true, focused: index == 0,
            };
            self.windows.push(info.clone());
            handler(Event::WindowFrameChanged(WindowState {
                id: info.id, label: info.label.clone(),
                title: info.title.clone(),
                frame: info.frame, scale_factor: info.scale_factor,
                open: true, focused: index == 0, maximized: false, fullscreen: false,
            }));
        }
        for _ in 0..self.requested_frames { handler(Event::FrameRequested); }
        handler(Event::AppShutdown);
    }

    fn stop(&mut self) {}

    fn load_webview(&mut self, source: &WebViewSource) {
        self.loaded_source = Some(source.clone());
    }

    fn load_window_webview(&mut self, _window_id: WindowId, source: &WebViewSource) {
        self.loaded_source = Some(source.clone());
    }

    fn complete_bridge(&mut self, response: &[u8]) { self.record_bridge_response(1, response); }
    fn complete_window_bridge(&mut self, window_id: WindowId, response: &[u8]) { self.record_bridge_response(window_id, response); }

    fn create_window(&mut self, options: &WindowOptions) -> Result<WindowInfo, PlatformError> {
        if self.windows.len() >= MAX_WINDOWS { return Err(PlatformError::WindowLimitReached); }
        if self.windows.iter().any(|w| w.id == options.id) { return Err(PlatformError::DuplicateWindowId); }
        if self.windows.iter().any(|w| w.label == options.label) { return Err(PlatformError::DuplicateWindowLabel); }
        let info = WindowInfo {
            id: options.id, label: options.label.clone(),
            title: options.resolved_title(&self.app_info_value.app_name).to_string(),
            frame: options.default_frame, scale_factor: self.surface_value.scale_factor,
            open: true, focused: false,
        };
        self.windows.push(info.clone());
        Ok(info)
    }

    fn focus_window(&mut self, window_id: WindowId) -> Result<(), PlatformError> {
        let idx = self.find_window_index(window_id).ok_or(PlatformError::WindowNotFound)?;
        for (i, w) in self.windows.iter_mut().enumerate() { w.focused = i == idx; }
        Ok(())
    }

    fn close_window(&mut self, window_id: WindowId) -> Result<(), PlatformError> {
        let idx = self.find_window_index(window_id).ok_or(PlatformError::WindowNotFound)?;
        self.windows[idx].open = false;
        self.windows[idx].focused = false;
        Ok(())
    }

    fn show_open_dialog(&mut self, _options: &OpenDialogOptions, _buffer: &mut [u8]) -> Result<OpenDialogResult, PlatformError> { Err(PlatformError::UnsupportedService) }
    fn show_save_dialog(&mut self, _options: &SaveDialogOptions, _buffer: &mut [u8]) -> Result<Option<String>, PlatformError> { Err(PlatformError::UnsupportedService) }
    fn show_message_dialog(&mut self, _options: &MessageDialogOptions) -> Result<MessageDialogResult, PlatformError> { Err(PlatformError::UnsupportedService) }

    fn create_tray(&mut self, _options: &TrayOptions) -> Result<(), PlatformError> { Err(PlatformError::UnsupportedService) }
    fn update_tray_menu(&mut self, _items: &[TrayMenuItem]) -> Result<(), PlatformError> { Err(PlatformError::UnsupportedService) }
    fn remove_tray(&mut self) -> Result<(), PlatformError> { Err(PlatformError::UnsupportedService) }

    fn read_clipboard(&mut self, _buffer: &mut [u8]) -> Result<String, PlatformError> { Err(PlatformError::UnsupportedService) }
    fn write_clipboard(&mut self, _text: &str) -> Result<(), PlatformError> { Err(PlatformError::UnsupportedService) }

    fn configure_security_policy(&mut self, policy: &security::Policy) { self.security_policy = policy.clone(); }
    fn emit_window_event(&mut self, _window_id: WindowId, _name: &str, _detail_json: &str) {}

    fn box_clone(&self) -> Box<dyn PlatformHost> {
        Box::new(self.clone())
    }

    fn as_any(&self) -> &dyn std::any::Any { self }
}

pub fn current_backend() -> &'static str {
    if cfg!(target_os = "macos") { "macos" }
    else if cfg!(target_os = "linux") { "linux" }
    else if cfg!(target_os = "windows") { "windows" }
    else { "unknown" }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn null_platform_emits_lifecycle_events() {
        let mut np = NullPlatform::new(Surface::default());
        let mut events = Vec::new();
        np.run(&mut |e| events.push(e.name().to_string()));
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
    fn null_platform_trait_host_bridge_response() {
        let mut host: Box<dyn PlatformHost> = Box::new(NullPlatform::new(Surface::default()));
        host.complete_window_bridge(7, b"{\"ok\":true}");
        // Verify through downcast
        let np = host.as_any().downcast_ref::<NullPlatform>().unwrap();
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

    #[test]
    fn null_platform_create_window() {
        let mut np = NullPlatform::new(Surface::default());
        let info = np.create_window(&WindowOptions { id: 2, label: "tools".into(), ..Default::default() }).unwrap();
        assert_eq!(2, info.id);
        assert_eq!("tools", info.label);
    }

    #[test]
    fn null_platform_rejects_duplicate_id() {
        let mut np = NullPlatform::new(Surface::default());
        let _ = np.create_window(&WindowOptions::default()).unwrap();
        assert!(matches!(np.create_window(&WindowOptions::default()), Err(PlatformError::DuplicateWindowId)));
    }

    #[test]
    fn window_info_to_state() {
        let info = WindowInfo { id: 3, label: "tools".into(), title: "Tools".into(), ..Default::default() };
        let state = info.state();
        assert_eq!(3, state.id);
        assert_eq!("tools", state.label);
    }
}
