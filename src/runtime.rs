use crate::geometry::RectF;
use crate::platform::{
    self, AppInfo, BridgeMessage, Event as PlatformEvent, Surface, WebViewSource, WindowId,
    WindowInfo, WindowState, WindowCreateOptions, WindowOptions, NullPlatform,
};
use crate::bridge::{self, Dispatcher, Policy as BridgePolicy, Request, Source};
use crate::security;
use crate::extensions::{self, ModuleRegistry, RuntimeContext};
use crate::trace;
use crate::json;
use crate::automation::{self, snapshot};

pub const MAX_WINDOWS: usize = platform::MAX_WINDOWS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LifecycleEvent { Start, Frame, Stop }

#[derive(Debug, Clone)]
pub struct CommandEvent { pub name: String }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvalidationReason { Startup, SurfaceResize, Command, State }

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RuntimeError {
    Bridge,
    WindowLimitReached,
    DuplicateWindowId,
    DuplicateWindowLabel,
    MissingWindowSource,
    WindowSourceTooLarge,
    InvalidWindowOptions,
    InvalidJsonEventDetail,
    NoSpaceLeft,
    UnsupportedService,
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) }
}
impl std::error::Error for RuntimeError {}

#[derive(Debug, Clone)]
pub enum Event {
    Lifecycle(LifecycleEvent),
    Command(CommandEvent),
}

impl Event {
    pub fn name(&self) -> &str {
        match self {
            Event::Lifecycle(l) => match l { LifecycleEvent::Start => "start", LifecycleEvent::Frame => "frame", LifecycleEvent::Stop => "stop" },
            Event::Command(c) => &c.name,
        }
    }
}

pub struct App {
    pub name: String,
    pub source: WebViewSource,
}

impl App {
    pub fn simple(name: &str, source: WebViewSource) -> Self { Self { name: name.to_string(), source } }
    pub fn web_view_source(&self) -> WebViewSource { self.source.clone() }
}

#[derive(Debug, Clone, Copy)]
pub struct FrameDiagnostics {
    pub frame_index: u64,
    pub command_count: usize,
    pub dirty_region_count: usize,
    pub duration_ns: u64,
}

impl Default for FrameDiagnostics {
    fn default() -> Self { Self { frame_index: 0, command_count: 0, dirty_region_count: 0, duration_ns: 0 } }
}

pub struct Options {
    pub trace_sink: Option<Box<dyn trace::Sink>>,
    pub log_path: Option<String>,
    pub extensions: Option<ModuleRegistry>,
    pub bridge: Option<Dispatcher>,
    pub builtin_bridge: BridgePolicy,
    pub security: security::Policy,
    pub automation: Option<automation::Server>,
    pub window_state_store: Option<crate::window_state::Store>,
    pub js_window_api: bool,
}

struct RuntimeWindow {
    info: WindowInfo,
    source: Option<WebViewSource>,
}

pub struct Runtime {
    pub null_platform: NullPlatform,
    pub surface: Surface,
    pub windows: Vec<RuntimeWindow>,
    pub next_window_id: WindowId,
    pub invalidated: bool,
    pub frame_index: u64,
    pub command_count: usize,
    pub dirty_regions: Vec<RectF>,
    pub last_diagnostics: FrameDiagnostics,
    pub loaded_source: Option<WebViewSource>,
    pub options: Options,
    automation_windows: Vec<snapshot::Window>,
}

impl Runtime {
    pub fn new(null_platform: NullPlatform, options: Options) -> Self {
        let surface = null_platform.surface_value.clone();
        Self {
            null_platform, surface,
            windows: Vec::with_capacity(MAX_WINDOWS),
            next_window_id: 2, invalidated: true,
            frame_index: 0, command_count: 0,
            dirty_regions: Vec::with_capacity(8),
            last_diagnostics: FrameDiagnostics::default(),
            loaded_source: None, options,
            automation_windows: Vec::with_capacity(snapshot::MAX_WINDOWS),
        }
    }

    pub fn invalidate(&mut self) { self.invalidated = true; }

    pub fn run(&mut self, app: &mut App) -> Result<(), RuntimeError> {
        // Drive the platform event loop by collecting events first, then dispatching.
        // This avoids the &mut self borrow conflict of closure-capturing self.
        let mut events: Vec<PlatformEvent> = Vec::new();
        self.null_platform.run_event_loop(&mut |event| { events.push(event); });
        for event in events {
            self.dispatch_platform_event(app, event)?;
        }
        Ok(())
    }

    pub fn create_window(&mut self, create_opts: WindowCreateOptions) -> Result<WindowInfo, RuntimeError> {
        let source = create_opts.source.clone().or(self.loaded_source.clone()).ok_or(RuntimeError::MissingWindowSource)?;
        let id = if create_opts.id != 0 { create_opts.id } else { self.allocate_window_id() };
        if self.find_window_index_by_id(id).is_some() { return Err(RuntimeError::DuplicateWindowId); }
        if create_opts.label.is_empty() { return Err(RuntimeError::InvalidWindowOptions); }
        if self.find_window_index_by_label(&create_opts.label).is_some() { return Err(RuntimeError::DuplicateWindowLabel); }
        if self.windows.len() >= MAX_WINDOWS { return Err(RuntimeError::WindowLimitReached); }

        let index = self.windows.len();
        let info = WindowInfo {
            id, label: create_opts.label.clone(), title: create_opts.title.clone(),
            frame: create_opts.default_frame, scale_factor: self.surface.scale_factor,
            open: true, focused: self.windows.is_empty(),
        };
        self.windows.push(RuntimeWindow { info, source: Some(source) });
        self.next_window_id = self.next_window_id.max(id + 1);
        self.invalidated = true;
        Ok(self.windows[index].info.clone())
    }

    pub fn list_windows_vec(&self) -> Vec<WindowInfo> { self.windows.iter().map(|w| w.info.clone()).collect() }

    pub fn focus_window(&mut self, window_id: WindowId) -> Result<(), RuntimeError> {
        let index = self.find_window_index_by_id(window_id).ok_or(RuntimeError::MissingWindowSource)?;
        self.set_focused_index(index);
        self.invalidated = true;
        Ok(())
    }

    pub fn close_window(&mut self, window_id: WindowId) -> Result<(), RuntimeError> {
        let _ = self.find_window_index_by_id(window_id).ok_or(RuntimeError::MissingWindowSource)?;
        if let Some(w) = self.windows.iter_mut().find(|w| w.info.id == window_id) {
            w.info.open = false; w.info.focused = false;
        }
        self.invalidated = true;
        Ok(())
    }

    pub fn frame_diagnostics(&self) -> FrameDiagnostics { self.last_diagnostics }

    pub fn dispatch_platform_event(&mut self, app: &mut App, event: PlatformEvent) -> Result<(), RuntimeError> {
        match event {
            PlatformEvent::AppStart => {
                self.load_startup_windows(app);
                self.invalidated = true;
            }
            PlatformEvent::SurfaceResized(surface) => {
                self.surface = surface;
                self.invalidated = true;
            }
            PlatformEvent::WindowFrameChanged(state) => { self.update_window_state(&state); self.invalidated = true; }
            PlatformEvent::WindowFocused(id) => { if let Some(i) = self.find_window_index_by_id(id) { self.set_focused_index(i); } self.invalidated = true; }
            PlatformEvent::BridgeMessage(msg) => { self.handle_bridge_message(&msg); self.invalidated = true; }
            PlatformEvent::FrameRequested => { self.frame(app)?; }
            PlatformEvent::TrayAction(_) => {}
            PlatformEvent::AppShutdown => {}
        }
        Ok(())
    }

    pub fn frame(&mut self, _app: &mut App) -> Result<(), RuntimeError> {
        self.consume_automation_command();
        if !self.invalidated { return Ok(()); }
        self.publish_automation();
        self.frame_index += 1;
        self.last_diagnostics = FrameDiagnostics { frame_index: self.frame_index, command_count: self.command_count, dirty_region_count: self.dirty_regions.len(), duration_ns: 0 };
        self.command_count = 0;
        self.dirty_regions.clear();
        self.invalidated = false;
        Ok(())
    }

    pub fn automation_snapshot(&mut self, title: &str) -> snapshot::Input {
        let count = self.windows.len().min(snapshot::MAX_WINDOWS);
        self.automation_windows.clear();
        if count == 0 {
            self.automation_windows.push(snapshot::Window { id: 1, title: title.to_string(), bounds: RectF::from_size(self.surface.size), focused: true });
        } else {
            for w in self.windows.iter().take(count) {
                self.automation_windows.push(snapshot::Window { id: w.info.id, title: if w.info.title.is_empty() { title.to_string() } else { w.info.title.clone() }, bounds: w.info.frame, focused: w.info.focused });
            }
        }
        snapshot::Input { windows: self.automation_windows.clone(), diagnostics: snapshot::Diagnostics { frame_index: self.last_diagnostics.frame_index, command_count: self.last_diagnostics.command_count }, source: self.loaded_source.clone() }
    }

    fn load_startup_windows(&mut self, app: &App) {
        let source = app.web_view_source();
        self.loaded_source = Some(source);
        // Create the default main window (id=1)
        let count = self.null_platform.app_info.startup_window_count();
        for index in 0..count {
            let window = self.null_platform.app_info.resolved_startup_window(index);
            if self.find_window_index_by_id(window.id).is_none() && self.windows.len() < MAX_WINDOWS {
                let info = WindowInfo {
                    id: window.id,
                    label: window.label.clone(),
                    title: window.resolved_title(&self.null_platform.app_info.app_name).to_string(),
                    frame: window.default_frame,
                    scale_factor: self.surface.scale_factor,
                    open: true,
                    focused: index == 0,
                };
                self.windows.push(RuntimeWindow { info, source: self.loaded_source.clone() });
                self.next_window_id = self.next_window_id.max(window.id + 1);
            }
        }
    }

    fn reload_windows(&mut self, app: &App) {
        let source = app.web_view_source();
        self.loaded_source = Some(source);
    }

    fn handle_bridge_message(&mut self, message: &BridgeMessage) {
        self.command_count += 1;
        if self.handle_builtin_bridge_message(message) { return; }
        let dispatcher = match &self.options.bridge {
            Some(d) => d.clone(),
            None => Dispatcher::default(),
        };
        let mut response_buffer = vec![0u8; bridge::MAX_RESPONSE_BYTES];
        let source = Source { origin: message.origin.clone(), window_id: message.window_id };
        let response_len = dispatcher.dispatch(&message.bytes, source, &mut response_buffer);
        self.null_platform.record_bridge_response(message.window_id, &response_buffer[..response_len]);
    }

    fn complete_bridge_response(&mut self, window_id: WindowId, response: &[u8]) {
        self.null_platform.record_bridge_response(window_id, response);
        if let Some(ref server) = self.options.automation { let _ = server.publish_bridge_response(response); }
    }

    fn handle_builtin_bridge_message(&mut self, message: &BridgeMessage) -> bool {
        let request = match bridge::Request::parse(&message.bytes) { Ok(r) => r, Err(_) => return false };
        let is_window = request.command.starts_with("zero-native.window.");
        let is_dialog = request.command.starts_with("zero-native.dialog.");
        if !is_window && !is_dialog { return false; }

        let mut response_buffer = vec![0u8; bridge::MAX_RESPONSE_BYTES];
        if !self.allows_builtin_bridge_command(&request.command, &message.origin, is_window) {
            let msg_text = if is_window { "Window API is not permitted" } else { "Dialog API is not permitted" };
            let len = bridge::write_error_response(&mut response_buffer, &request.id, bridge::ErrorCode::PermissionDenied, msg_text);
            self.complete_bridge_response(message.window_id, &response_buffer[..len]);
            return true;
        }

        let result_len = if is_window { self.dispatch_window_bridge_command(&request, &mut response_buffer) } else { self.dispatch_dialog_bridge_command(&request, &mut response_buffer) };
        self.complete_bridge_response(message.window_id, &response_buffer[..result_len]);
        true
    }

    fn allows_builtin_bridge_command(&self, command: &str, origin: &str, is_window: bool) -> bool {
        let mut policy = self.options.builtin_bridge.clone();
        if !self.options.security.permissions.is_empty() { policy.permissions = self.options.security.permissions.clone(); }
        if policy.enabled { return policy.allows(command, origin); }
        if !is_window || !self.options.js_window_api { return false; }
        let allowed: Vec<&str> = self.options.security.navigation.allowed_origins.iter().map(|s| s.as_str()).collect();
        if !security::allows_origin(&allowed, origin) { return false; }
        if self.options.security.permissions.is_empty() { return true; }
        let perms: Vec<&str> = self.options.security.permissions.iter().map(|s| s.as_str()).collect();
        security::has_permission(&perms, security::PERMISSION_WINDOW)
    }

    fn dispatch_window_bridge_command(&mut self, request: &Request, response_buffer: &mut [u8]) -> usize {
        if request.command == "zero-native.window.create" {
            let mut result_buffer = vec![0u8; 4096];
            match self.create_window_from_json(&request.payload, &mut result_buffer) {
                Ok(len) => { let s = std::str::from_utf8(&result_buffer[..len]).unwrap_or("{}"); bridge::write_success_response(response_buffer, &request.id, s) }
                Err(e) => { let msg = format!("{:?}", e); bridge::write_error_response(response_buffer, &request.id, bridge::ErrorCode::InternalError, &msg) }
            }
        } else if request.command == "zero-native.window.list" {
            let mut result_buffer = vec![0u8; 8192];
            match self.write_window_list_json(&mut result_buffer) {
                Ok(len) => { let s = std::str::from_utf8(&result_buffer[..len]).unwrap_or("[]"); bridge::write_success_response(response_buffer, &request.id, s) }
                Err(_) => bridge::write_error_response(response_buffer, &request.id, bridge::ErrorCode::InternalError, "Failed to list windows")
            }
        } else {
            bridge::write_error_response(response_buffer, &request.id, bridge::ErrorCode::UnknownCommand, "Unknown window command")
        }
    }

    fn dispatch_dialog_bridge_command(&self, request: &Request, response_buffer: &mut [u8]) -> usize {
        bridge::write_error_response(response_buffer, &request.id, bridge::ErrorCode::InternalError, "Dialog API is not available")
    }

    fn create_window_from_json(&mut self, payload: &str, output: &mut [u8]) -> Result<usize, RuntimeError> {
        let mut storage = Vec::new();
        let label = json::string_field(payload, "label", &mut storage).unwrap_or("window").to_string();
        let title = json::string_field(payload, "title", &mut storage).unwrap_or("").to_string();
        let width = json::number_field(payload, "width").unwrap_or(720.0);
        let height = json::number_field(payload, "height").unwrap_or(480.0);
        let x = json::number_field(payload, "x").unwrap_or(0.0);
        let y = json::number_field(payload, "y").unwrap_or(0.0);
        let source = json::string_field(payload, "url", &mut storage).map(|url| WebViewSource::url(url));
        let info = self.create_window(WindowCreateOptions { id: 0, label, title, default_frame: RectF::new(x, y, width, height), source, ..Default::default() })?;
        Ok(write_window_json(&info, output))
    }

    fn resolve_window_selector(&self, payload: &str, storage: &mut Vec<u8>) -> Result<WindowId, RuntimeError> {
        if let Some(id) = json::unsigned_field::<WindowId>(payload, "id") { return Ok(id); }
        if let Some(label) = json::string_field(payload, "label", storage) {
            let idx = self.find_window_index_by_label(label).ok_or(RuntimeError::MissingWindowSource)?;
            return Ok(self.windows[idx].info.id);
        }
        Err(RuntimeError::MissingWindowSource)
    }

    fn write_window_list_json(&self, output: &mut [u8]) -> Result<usize, RuntimeError> {
        let mut pos = 0;
        output[pos] = b'['; pos += 1;
        for (i, w) in self.windows.iter().enumerate() {
            if i > 0 { output[pos] = b','; pos += 1; }
            pos += write_window_json_to_writer(&w.info, &mut output[pos..]);
        }
        output[pos] = b']'; pos += 1;
        Ok(pos)
    }

    fn publish_automation(&mut self) {
        let title = self.options.automation.as_ref().map(|s| s.title.clone());
        if let Some(title) = title {
            let input = self.automation_snapshot(&title);
            if let Some(ref server) = self.options.automation { let _ = server.publish(&input); }
        }
    }

    fn consume_automation_command(&mut self) {
        let server = match &self.options.automation { Some(s) => s.clone(), None => return };
        let command = match server.take_command() { Some(c) => c, None => return };
        match command.action {
            automation::Action::Reload => { self.command_count += 1; self.invalidated = true; }
            automation::Action::Bridge => { self.handle_bridge_message(&BridgeMessage { bytes: command.value, origin: "zero://inline".into(), window_id: 1 }); }
            automation::Action::Wait => {}
        }
    }

    fn update_window_state(&mut self, state: &WindowState) {
        if let Some(idx) = self.find_window_index_by_id(state.id) {
            let info = &mut self.windows[idx].info;
            info.frame = state.frame; info.scale_factor = state.scale_factor;
            info.open = state.open; info.focused = state.focused;
            if !state.title.is_empty() { info.title = state.title.clone(); }
            if state.focused { self.set_focused_index(idx); }
        }
    }

    fn set_focused_index(&mut self, focused_index: usize) {
        for (i, w) in self.windows.iter_mut().enumerate() { w.info.focused = i == focused_index; }
    }

    fn find_window_index_by_id(&self, id: WindowId) -> Option<usize> { self.windows.iter().position(|w| w.info.id == id) }
    fn find_window_index_by_label(&self, label: &str) -> Option<usize> { self.windows.iter().position(|w| w.info.label == label) }
    fn allocate_window_id(&mut self) -> WindowId { while self.find_window_index_by_id(self.next_window_id).is_some() { self.next_window_id += 1; } let id = self.next_window_id; self.next_window_id += 1; id }

    fn log(&mut self, name: &str, message: &str, fields: &[trace::Field]) {
        if let Some(ref mut sink) = self.options.trace_sink {
            let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap_or_default().as_nanos() as i128;
            sink.write(trace::event_record(trace::Timestamp::from_nanoseconds(ts), trace::Level::Info, name, Some(message), fields.to_vec()));
        }
    }
}

fn write_window_json(window: &WindowInfo, output: &mut [u8]) -> usize { write_window_json_to_writer(window, output) }

fn write_window_json_to_writer(window: &WindowInfo, output: &mut [u8]) -> usize {
    use std::io::Write;
    let mut cursor = std::io::Cursor::new(output);
    let _ = write!(
        cursor,
        "{{\"id\":{},\"label\":{},\"title\":{},\"open\":{},\"focused\":{},\"x\":{:.0},\"y\":{:.0},\"width\":{:.0},\"height\":{:.0},\"scale\":{:.0}}}",
        window.id, json::write_json_string(&window.label), json::write_json_string(&window.title),
        window.open, window.focused, window.frame.x, window.frame.y, window.frame.width, window.frame.height, window.scale_factor,
    );
    cursor.position() as usize
}

pub struct TestHarness { pub runtime: Runtime, pub null_platform: NullPlatform }

impl TestHarness {
    pub fn new(surface: Surface) -> Self {
        let null_platform = NullPlatform::new(surface.clone());
        let runtime = Runtime::new(null_platform.clone(), Options {
            trace_sink: None, log_path: None, extensions: None, bridge: None,
            builtin_bridge: BridgePolicy::default(), security: security::Policy::default(),
            automation: None, window_state_store: None, js_window_api: false,
        });
        Self { runtime, null_platform }
    }

    pub fn start(&mut self, app: &mut App) -> Result<(), RuntimeError> {
        self.runtime.dispatch_platform_event(app, PlatformEvent::AppStart)?;
        self.runtime.dispatch_platform_event(app, PlatformEvent::SurfaceResized(self.runtime.surface.clone()))?;
        self.runtime.dispatch_platform_event(app, PlatformEvent::FrameRequested)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_loads_app_source() {
        let mut harness = TestHarness::new(Surface::default());
        let mut app = App::simple("test", WebViewSource::html("<h1>Hello</h1>"));
        harness.start(&mut app).unwrap();
        assert_eq!(platform::WebViewSourceKind::Html, harness.runtime.loaded_source.as_ref().unwrap().kind);
        assert_eq!("<h1>Hello</h1>", harness.runtime.loaded_source.as_ref().unwrap().bytes);
    }

    #[test]
    fn runtime_creates_and_closes_windows() {
        let mut harness = TestHarness::new(Surface::default());
        let mut app = App::simple("test", WebViewSource::html("hi"));
        harness.start(&mut app).unwrap();
        let info = harness.runtime.create_window(WindowCreateOptions { label: "tools".into(), title: "Tools".into(), ..Default::default() }).unwrap();
        assert_eq!(2, info.id);
        assert_eq!(2, harness.runtime.list_windows_vec().len());
        harness.runtime.focus_window(info.id).unwrap();
        assert!(harness.runtime.windows[1].info.focused);
        harness.runtime.close_window(info.id).unwrap();
        assert!(!harness.runtime.windows[1].info.open);
    }

    #[test]
    fn runtime_rejects_duplicate_window_id() {
        let mut harness = TestHarness::new(Surface::default());
        let mut app = App::simple("test", WebViewSource::html("hi"));
        harness.start(&mut app).unwrap();
        // Window id=1 already created by load_startup_windows, so duplicate should fail
        assert!(matches!(harness.runtime.create_window(WindowCreateOptions { id: 1, label: "other".into(), ..Default::default() }), Err(RuntimeError::DuplicateWindowId)));
    }

    #[test]
    fn runtime_dispatches_bridge_messages() {
        let mut harness = TestHarness::new(Surface::default());
        let mut app = App::simple("test", WebViewSource::html("hi"));
        harness.start(&mut app).unwrap();
        let msg = BridgeMessage { bytes: "{\"id\":\"1\"}".into(), origin: "zero://inline".into(), window_id: 1 };
        harness.runtime.dispatch_platform_event(&mut app, PlatformEvent::BridgeMessage(msg)).unwrap();
        assert_eq!(1, harness.runtime.command_count);
    }

    #[test]
    fn runtime_handles_builtin_window_bridge_commands() {
        let mut harness = TestHarness::new(Surface::default());
        harness.runtime.options.js_window_api = true;
        let mut app = App::simple("test", WebViewSource::html("hi"));
        harness.start(&mut app).unwrap();
        harness.runtime.dispatch_platform_event(&mut app, PlatformEvent::BridgeMessage(BridgeMessage {
            bytes: r#"{"id":"1","command":"zero-native.window.create","payload":{"label":"palette","title":"Palette","width":320,"height":240}}"#.into(),
            origin: "zero://inline".into(), window_id: 1,
        })).unwrap();
        assert!(harness.runtime.null_platform.last_bridge_response().contains("\"ok\":true"));
    }

    #[test]
    fn runtime_gates_window_api_by_origin() {
        let mut harness = TestHarness::new(Surface::default());
        harness.runtime.options.js_window_api = true;
        let mut app = App::simple("test", WebViewSource::html("hi"));
        harness.start(&mut app).unwrap();
        harness.runtime.dispatch_platform_event(&mut app, PlatformEvent::BridgeMessage(BridgeMessage {
            bytes: r#"{"id":"1","command":"zero-native.window.list","payload":null}"#.into(),
            origin: "https://evil.com".into(), window_id: 1,
        })).unwrap();
        assert!(harness.runtime.null_platform.last_bridge_response().contains("\"permission_denied\""));
    }
}
