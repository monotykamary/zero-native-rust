use crate::geometry::{RectF, SizeF};
use crate::platform::{
    self, AppInfo, BridgeMessage, Event as PlatformEvent, Surface, WebViewSource, WindowId,
    WindowInfo, WindowState, WindowCreateOptions, WindowOptions, NullPlatform, PlatformHost,
    PlatformError, MessageDialogStyle, MessageDialogResult,
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
    WindowNotFound,
    PlatformError(PlatformError),
}

impl std::fmt::Display for RuntimeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result { write!(f, "{:?}", self) }
}
impl std::error::Error for RuntimeError {}

impl From<PlatformError> for RuntimeError {
    fn from(e: PlatformError) -> Self { RuntimeError::PlatformError(e) }
}

fn builtin_bridge_error_message(err: &RuntimeError) -> &'static str {
    match err {
        RuntimeError::UnsupportedService => "Native service is not available on this platform",
        RuntimeError::WindowNotFound => "Window was not found",
        RuntimeError::WindowLimitReached => "Window limit reached",
        RuntimeError::DuplicateWindowLabel => "Window id or label already exists",
        RuntimeError::MissingWindowSource => "Window source is missing",
        RuntimeError::WindowSourceTooLarge => "Window source is too large",
        RuntimeError::InvalidWindowOptions => "Window options are invalid",
        RuntimeError::DuplicateWindowId => "Window id already exists",
        RuntimeError::NoSpaceLeft => "Native response buffer is too small",
        _ => "Native command failed",
    }
}

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

#[derive(Clone)]
struct RuntimeWindow {
    info: WindowInfo,
    source: Option<WebViewSource>,
}

pub struct Runtime {
    pub host: Box<dyn PlatformHost>,
    pub surface: Surface,
    pub windows: Vec<RuntimeWindow>,
    pub next_window_id: WindowId,
    pub invalidated: bool,
    pub frame_index: u64,
    pub command_count: usize,
    pub dirty_regions: Vec<RectF>,
    pub last_invalidation_reason: InvalidationReason,
    pub last_diagnostics: FrameDiagnostics,
    pub loaded_source: Option<WebViewSource>,
    pub options: Options,
    automation_windows: Vec<snapshot::Window>,
}

impl Clone for Runtime {
    fn clone(&self) -> Self {
        Self {
            host: Box::new(NullPlatform::with_options(
                self.surface.clone(),
                platform::WebEngine::System,
                self.host.app_info().clone(),
            )),
            surface: self.surface.clone(),
            windows: self.windows.clone(),
            next_window_id: self.next_window_id,
            invalidated: self.invalidated,
            frame_index: self.frame_index,
            command_count: self.command_count,
            last_invalidation_reason: self.last_invalidation_reason,
            dirty_regions: self.dirty_regions.clone(),
            last_diagnostics: self.last_diagnostics,
            loaded_source: self.loaded_source.clone(),
            options: Options {
                trace_sink: None,
                log_path: self.options.log_path.clone(),
                extensions: self.options.extensions.clone(),
                bridge: self.options.bridge.clone(),
                builtin_bridge: self.options.builtin_bridge.clone(),
                security: self.options.security.clone(),
                automation: self.options.automation.clone(),
                window_state_store: self.options.window_state_store.clone(),
                js_window_api: self.options.js_window_api,
            },
            automation_windows: self.automation_windows.clone(),
        }
    }
}

impl Runtime {
    pub fn new(host: impl PlatformHost + 'static, options: Options) -> Self {
        let surface = host.surface();
        Self {
            host: Box::new(host), surface,
            windows: Vec::with_capacity(MAX_WINDOWS),
            next_window_id: 2, invalidated: true,
            frame_index: 0, command_count: 0, last_invalidation_reason: InvalidationReason::Startup,
            dirty_regions: Vec::with_capacity(8),
            last_diagnostics: FrameDiagnostics::default(),
            loaded_source: None, options,
            automation_windows: Vec::with_capacity(snapshot::MAX_WINDOWS),
        }
    }

    pub fn from_boxed(host: Box<dyn PlatformHost>, options: Options) -> Self {
        let surface = host.surface();
        Self {
            host, surface,
            windows: Vec::with_capacity(MAX_WINDOWS),
            next_window_id: 2, invalidated: true,
            frame_index: 0, command_count: 0, last_invalidation_reason: InvalidationReason::Startup,
            dirty_regions: Vec::with_capacity(8),
            last_diagnostics: FrameDiagnostics::default(),
            loaded_source: None, options,
            automation_windows: Vec::with_capacity(snapshot::MAX_WINDOWS),
        }
    }

    pub fn invalidate(&mut self) { self.invalidate_for(InvalidationReason::State, None); }

    pub fn invalidate_for(&mut self, reason: InvalidationReason, dirty_region: Option<RectF>) {
        self.invalidated = true;
        self.last_invalidation_reason = reason;
        if let Some(region) = dirty_region {
            if self.dirty_regions.len() < 8 {
                self.dirty_regions.push(region);
            }
        }
    }

    pub fn run(&mut self, app: &mut App) -> Result<(), RuntimeError> {
        self.log("runtime.init", "runtime initialized", &[]);
        self.host.configure_security_policy(&self.options.security);
        let mut events: Vec<PlatformEvent> = Vec::new();
        self.host.run(&mut |event| { events.push(event); });
        for event in events {
            self.dispatch_platform_event(app, event)?;
        }
        self.log("runtime.done", "runtime finished", &[]);
        Ok(())
    }

    pub fn create_window(&mut self, create_opts: WindowCreateOptions) -> Result<WindowInfo, RuntimeError> {
        let source = create_opts.source.clone().or(self.loaded_source.clone()).ok_or(RuntimeError::MissingWindowSource)?;
        let id = if create_opts.id != 0 { create_opts.id } else { self.allocate_window_id() };
        if self.find_window_index_by_id(id).is_some() { return Err(RuntimeError::DuplicateWindowId); }
        if create_opts.label.is_empty() { return Err(RuntimeError::InvalidWindowOptions); }
        if self.find_window_index_by_label(&create_opts.label).is_some() { return Err(RuntimeError::DuplicateWindowLabel); }
        if self.windows.len() >= MAX_WINDOWS { return Err(RuntimeError::WindowLimitReached); }

        let win_opts = create_opts.window_options(id, &create_opts.label);
        let native_info = self.host.create_window(&win_opts)?;
        self.host.load_window_webview(id, &source);

        let runtime_info = WindowInfo {
            id, label: create_opts.label.clone(), title: native_info.title.clone(),
            frame: create_opts.default_frame, scale_factor: self.surface.scale_factor,
            open: true, focused: self.windows.is_empty(),
        };
        self.windows.push(RuntimeWindow { info: runtime_info.clone(), source: Some(source) });
        self.next_window_id = self.next_window_id.max(id + 1);
        self.invalidate_for(InvalidationReason::Command, None);
        Ok(runtime_info)
    }

    pub fn list_windows<'a>(&self, output: &'a mut [WindowInfo]) -> &'a [WindowInfo] {
        let count = output.len().min(self.windows.len());
        for (i, w) in self.windows.iter().enumerate().take(count) {
            output[i] = w.info.clone();
        }
        &output[..count]
    }

    pub fn list_windows_vec(&self) -> Vec<WindowInfo> { self.windows.iter().map(|w| w.info.clone()).collect() }

    pub fn focus_window(&mut self, window_id: WindowId) -> Result<(), RuntimeError> {
        let index = self.find_window_index_by_id(window_id).ok_or(RuntimeError::WindowNotFound)?;
        self.host.focus_window(window_id)?;
        self.set_focused_index(index);
        self.invalidate_for(InvalidationReason::Command, None);
        Ok(())
    }

    pub fn close_window(&mut self, window_id: WindowId) -> Result<(), RuntimeError> {
        let _ = self.find_window_index_by_id(window_id).ok_or(RuntimeError::WindowNotFound)?;
        self.host.close_window(window_id)?;
        if let Some(w) = self.windows.iter_mut().find(|w| w.info.id == window_id) {
            w.info.open = false; w.info.focused = false;
        }
        self.invalidate_for(InvalidationReason::Command, None);
        Ok(())
    }

    pub fn emit_window_event(&mut self, window_id: WindowId, name: &str, detail_json: &str) -> Result<(), RuntimeError> {
        if !json::is_valid_value(detail_json) { return Err(RuntimeError::InvalidJsonEventDetail); }
        self.host.emit_window_event(window_id, name, detail_json);
        Ok(())
    }

    pub fn respond_to_bridge(&mut self, source: &Source, response: &[u8]) -> Result<(), RuntimeError> {
        self.complete_bridge_response(source.window_id, response);
        Ok(())
    }

    pub fn dispatch_platform_event(&mut self, app: &mut App, event: PlatformEvent) -> Result<(), RuntimeError> {
        match event {
            PlatformEvent::AppStart => {
                self.log("app.start", "app started", &[trace::string_field("app", &app.name)]);
                if let Some(ref registry) = self.options.extensions { registry.start_all(self.extension_context()); }
                self.dispatch_event(app, Event::Lifecycle(LifecycleEvent::Start));
                self.load_startup_windows(app);
                self.invalidate_for(InvalidationReason::Startup, None);
            }
            PlatformEvent::SurfaceResized(ref surface) => {
                self.surface = surface.clone();
                self.host.set_surface(surface.clone());
                if let Some(idx) = self.find_window_index_by_id(surface.id) {
                    self.windows[idx].info.frame.width = surface.size.width;
                    self.windows[idx].info.frame.height = surface.size.height;
                    self.windows[idx].info.scale_factor = surface.scale_factor;
                }
                self.invalidate_for(InvalidationReason::SurfaceResize, Some(RectF::from_size(surface.size.clone())));
                self.log("surface.resize", "surface updated", &[
                    trace::float_field("width", surface.size.width as f64),
                    trace::float_field("height", surface.size.height as f64),
                    trace::float_field("scale", surface.scale_factor as f64),
                ]);
            }
            PlatformEvent::WindowFrameChanged(state) => {
                self.update_window_state(&state);
                if let Some(ref store) = self.options.window_state_store {
                    let _ = store.save_window(&state);
                }
                self.invalidate_for(InvalidationReason::State, None);
            }
            PlatformEvent::WindowFocused(id) => {
                if let Some(i) = self.find_window_index_by_id(id) { self.set_focused_index(i); }
                self.invalidate_for(InvalidationReason::Command, None);
            }
            PlatformEvent::BridgeMessage(msg) => { self.handle_bridge_message(&msg); self.invalidate_for(InvalidationReason::Command, None); }
            PlatformEvent::FrameRequested => { self.frame(app)?; }
            PlatformEvent::TrayAction(item_id) => {
                self.log("tray.action", "tray item selected", &[trace::uint_field("item_id", item_id as u64)]);
                self.dispatch_event(app, Event::Command(CommandEvent { name: "tray.action".to_string() }));
            }
            PlatformEvent::AppShutdown => {
                self.dispatch_event(app, Event::Lifecycle(LifecycleEvent::Stop));
                if let Some(ref registry) = self.options.extensions { registry.stop_all(self.extension_context()); }
                self.log("app.stop", "app stopped", &[trace::string_field("app", &app.name)]);
            }
        }
        Ok(())
    }

    pub fn dispatch_event(&mut self, app: &mut App, event: Event) {
        self.log("runtime.event", "", &[trace::string_field("event", event.name())]);
        if let Err(e) = app.event(self, &event) {
            self.log("runtime.event_failed", &format!("{:?}", e), &[]);
        }
        if let Event::Command(ref cmd) = event {
            if let Some(ref registry) = self.options.extensions {
                registry.dispatch_command(self.extension_context(), extensions::Command { name: cmd.name.clone(), target: None });
            }
            self.invalidate_for(InvalidationReason::Command, None);
        }
    }

    pub fn frame(&mut self, _app: &mut App) -> Result<(), RuntimeError> {
        let start = std::time::Instant::now();
        self.consume_automation_command();
        if !self.invalidated { return Ok(()); }
        self.publish_automation();
        self.frame_index += 1;
        self.last_diagnostics = FrameDiagnostics {
            frame_index: self.frame_index,
            command_count: self.command_count,
            dirty_region_count: self.dirty_regions.len(),
            duration_ns: start.elapsed().as_nanos() as u64,
        };
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

    pub fn frame_diagnostics(&self) -> FrameDiagnostics { self.last_diagnostics }

    fn load_startup_windows(&mut self, app: &App) {
        let source = app.web_view_source();
        self.loaded_source = Some(source.clone());
        let count = self.host.app_info().startup_window_count();
        for index in 0..count {
            let window = self.host.app_info().resolved_startup_window(index);
            if self.find_window_index_by_id(window.id).is_none() && self.windows.len() < MAX_WINDOWS {
                let info = WindowInfo {
                    id: window.id,
                    label: window.label.clone(),
                    title: window.resolved_title(&self.host.app_info().app_name).to_string(),
                    frame: window.default_frame,
                    scale_factor: self.surface.scale_factor,
                    open: true,
                    focused: index == 0,
                };
                self.windows.push(RuntimeWindow { info: info.clone(), source: Some(source.clone()) });
                self.next_window_id = self.next_window_id.max(window.id + 1);
            }
            if index > 0 {
                let _ = self.host.create_window(&window);
            }
            self.host.load_window_webview(window.id, &source);
        }
        self.log("webview.load", "loaded webview source", &[
            trace::string_field("kind", match source.kind { platform::WebViewSourceKind::Html => "html", platform::WebViewSourceKind::Url => "url", platform::WebViewSourceKind::Assets => "assets" }),
            trace::uint_field("bytes", source.bytes.len() as u64),
        ]);
    }

    fn reload_windows(&mut self, app: &App) {
        let source = app.web_view_source();
        self.loaded_source = Some(source.clone());
        if self.windows.is_empty() {
            self.host.load_webview(&source);
            return;
        }
        for w in &self.windows {
            let window_source = w.source.as_ref().unwrap_or(&source);
            self.host.load_window_webview(w.info.id, window_source);
        }
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

        if let Some(async_handler) = dispatcher.async_registry.find(&message.bytes) {
            // Async path — not fully implemented, fall through to sync
            let _ = async_handler;
        }

        let response_len = dispatcher.dispatch(&message.bytes, source.clone(), &mut response_buffer);
        self.complete_bridge_response(message.window_id, &response_buffer[..response_len]);
        self.log("bridge.dispatch", "bridge request handled", &[
            trace::uint_field("request_bytes", message.bytes.len() as u64),
            trace::uint_field("response_bytes", response_len as u64),
        ]);
    }

    fn complete_bridge_response(&mut self, window_id: WindowId, response: &[u8]) {
        self.host.complete_window_bridge(window_id, response);
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
                Err(e) => { let msg = builtin_bridge_error_message(&e); bridge::write_error_response(response_buffer, &request.id, bridge::ErrorCode::InternalError, msg) }
            }
        } else if request.command == "zero-native.window.list" {
            let mut result_buffer = vec![0u8; 8192];
            match self.write_window_list_json(&mut result_buffer) {
                Ok(len) => { let s = std::str::from_utf8(&result_buffer[..len]).unwrap_or("[]"); bridge::write_success_response(response_buffer, &request.id, s) }
                Err(_) => bridge::write_error_response(response_buffer, &request.id, bridge::ErrorCode::InternalError, "Failed to list windows")
            }
        } else if request.command == "zero-native.window.focus" {
            let mut result_buffer = vec![0u8; 4096];
            match self.focus_window_from_json(&request.payload, &mut result_buffer) {
                Ok(len) => { let s = std::str::from_utf8(&result_buffer[..len]).unwrap_or("{}"); bridge::write_success_response(response_buffer, &request.id, s) }
                Err(e) => { let msg = builtin_bridge_error_message(&e); bridge::write_error_response(response_buffer, &request.id, bridge::ErrorCode::InternalError, msg) }
            }
        } else if request.command == "zero-native.window.close" {
            let mut result_buffer = vec![0u8; 4096];
            match self.close_window_from_json(&request.payload, &mut result_buffer) {
                Ok(len) => { let s = std::str::from_utf8(&result_buffer[..len]).unwrap_or("{}"); bridge::write_success_response(response_buffer, &request.id, s) }
                Err(e) => { let msg = builtin_bridge_error_message(&e); bridge::write_error_response(response_buffer, &request.id, bridge::ErrorCode::InternalError, msg) }
            }
        } else {
            bridge::write_error_response(response_buffer, &request.id, bridge::ErrorCode::UnknownCommand, "Unknown window command")
        }
    }

    fn dispatch_dialog_bridge_command(&mut self, request: &Request, response_buffer: &mut [u8]) -> usize {
        if request.command == "zero-native.dialog.openFile" {
            match self.open_file_dialog_from_json(&request.payload) {
                Ok(result_json) => bridge::write_success_response(response_buffer, &request.id, &result_json),
                Err(e) => { let msg = builtin_bridge_error_message(&e); bridge::write_error_response(response_buffer, &request.id, bridge::ErrorCode::InternalError, msg) }
            }
        } else if request.command == "zero-native.dialog.saveFile" {
            match self.save_file_dialog_from_json(&request.payload) {
                Ok(result_json) => bridge::write_success_response(response_buffer, &request.id, &result_json),
                Err(e) => { let msg = builtin_bridge_error_message(&e); bridge::write_error_response(response_buffer, &request.id, bridge::ErrorCode::InternalError, msg) }
            }
        } else if request.command == "zero-native.dialog.showMessage" {
            match self.show_message_dialog_from_json(&request.payload) {
                Ok(result_json) => bridge::write_success_response(response_buffer, &request.id, &result_json),
                Err(e) => { let msg = builtin_bridge_error_message(&e); bridge::write_error_response(response_buffer, &request.id, bridge::ErrorCode::InternalError, msg) }
            }
        } else {
            bridge::write_error_response(response_buffer, &request.id, bridge::ErrorCode::UnknownCommand, "Unknown dialog command")
        }
    }

    fn open_file_dialog_from_json(&mut self, payload: &str) -> Result<String, RuntimeError> {
        let mut storage = Vec::new();
        let title = json::string_field(payload, "title", &mut storage).unwrap_or("").to_string();
        let default_path = json::string_field(payload, "defaultPath", &mut storage).unwrap_or("").to_string();
        let allow_dirs = json::bool_field(payload, "allowDirectories").unwrap_or(false);
        let allow_multi = json::bool_field(payload, "allowMultiple").unwrap_or(false);
        let mut dialog_buffer = vec![0u8; platform::MAX_DIALOG_PATHS_BYTES];
        let result = self.host.show_open_dialog(
            &platform::OpenDialogOptions { title, default_path, filters: Vec::new(), allow_directories: allow_dirs, allow_multiple: allow_multi },
            &mut dialog_buffer,
        )?;
        if result.count == 0 { Ok("null".to_string()) }
        else {
            let paths: Vec<String> = result.paths.split('\n').filter(|s| !s.is_empty()).map(|s| json::write_json_string(s)).collect();
            Ok(format!("[{}]", paths.join(",")))
        }
    }

    fn save_file_dialog_from_json(&mut self, payload: &str) -> Result<String, RuntimeError> {
        let mut storage = Vec::new();
        let title = json::string_field(payload, "title", &mut storage).unwrap_or("").to_string();
        let default_path = json::string_field(payload, "defaultPath", &mut storage).unwrap_or("").to_string();
        let default_name = json::string_field(payload, "defaultName", &mut storage).unwrap_or("").to_string();
        let mut dialog_buffer = vec![0u8; platform::MAX_DIALOG_PATH_BYTES];
        let path = self.host.show_save_dialog(
            &platform::SaveDialogOptions { title, default_path, default_name, filters: Vec::new() },
            &mut dialog_buffer,
        )?;
        match path { Some(p) => Ok(json::write_json_string(&p)), None => Ok("null".to_string()) }
    }

    fn show_message_dialog_from_json(&mut self, payload: &str) -> Result<String, RuntimeError> {
        let mut storage = Vec::new();
        let title = json::string_field(payload, "title", &mut storage).unwrap_or("").to_string();
        let message = json::string_field(payload, "message", &mut storage).unwrap_or("").to_string();
        let informative = json::string_field(payload, "informativeText", &mut storage).unwrap_or("").to_string();
        let primary = json::string_field(payload, "primaryButton", &mut storage).unwrap_or("OK").to_string();
        let secondary = json::string_field(payload, "secondaryButton", &mut storage).unwrap_or("").to_string();
        let tertiary = json::string_field(payload, "tertiaryButton", &mut storage).unwrap_or("").to_string();
        let style_str = json::string_field(payload, "style", &mut storage).unwrap_or("info");
        let style = match style_str {
            "warning" => MessageDialogStyle::Warning,
            "critical" => MessageDialogStyle::Critical,
            _ => MessageDialogStyle::Info,
        };
        let result = self.host.show_message_dialog(&platform::MessageDialogOptions {
            style, title, message, informative_text: informative,
            primary_button: primary, secondary_button: secondary, tertiary_button: tertiary,
        })?;
        let tag = match result { MessageDialogResult::Primary => "primary", MessageDialogResult::Secondary => "secondary", MessageDialogResult::Tertiary => "tertiary" };
        Ok(json::write_json_string(tag))
    }

    fn create_window_from_json(&mut self, payload: &str, output: &mut [u8]) -> Result<usize, RuntimeError> {
        let mut storage = Vec::new();
        let label = json::string_field(payload, "label", &mut storage).unwrap_or("window").to_string();
        let title = json::string_field(payload, "title", &mut storage).unwrap_or("").to_string();
        let width = json::number_field(payload, "width").unwrap_or(720.0);
        let height = json::number_field(payload, "height").unwrap_or(480.0);
        let x = json::number_field(payload, "x").unwrap_or(0.0);
        let y = json::number_field(payload, "y").unwrap_or(0.0);
        let restore = json::bool_field(payload, "restoreState").unwrap_or(true);
        let source = json::string_field(payload, "url", &mut storage).map(|url| WebViewSource::url(url));
        let info = self.create_window(WindowCreateOptions { id: 0, label, title, default_frame: RectF::new(x, y, width, height), restore_state: restore, source, ..Default::default() })?;
        Ok(write_window_json(&info, output))
    }

    fn focus_window_from_json(&mut self, payload: &str, output: &mut [u8]) -> Result<usize, RuntimeError> {
        let mut storage = Vec::new();
        let window_id = self.resolve_window_selector(payload, &mut storage)?;
        self.focus_window(window_id)?;
        let index = self.find_window_index_by_id(window_id).ok_or(RuntimeError::WindowNotFound)?;
        Ok(write_window_json(&self.windows[index].info, output))
    }

    fn close_window_from_json(&mut self, payload: &str, output: &mut [u8]) -> Result<usize, RuntimeError> {
        let mut storage = Vec::new();
        let window_id = self.resolve_window_selector(payload, &mut storage)?;
        let index = self.find_window_index_by_id(window_id).ok_or(RuntimeError::WindowNotFound)?;
        let info = self.windows[index].info.clone();
        self.close_window(window_id)?;
        let mut closed_info = info;
        closed_info.open = false;
        closed_info.focused = false;
        Ok(write_window_json(&closed_info, output))
    }

    fn resolve_window_selector(&self, payload: &str, storage: &mut Vec<u8>) -> Result<WindowId, RuntimeError> {
        if let Some(id) = json::unsigned_field::<WindowId>(payload, "id") { return Ok(id); }
        if let Some(label) = json::string_field(payload, "label", storage) {
            let idx = self.find_window_index_by_label(label).ok_or(RuntimeError::WindowNotFound)?;
            return Ok(self.windows[idx].info.id);
        }
        Err(RuntimeError::WindowNotFound)
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
            automation::Action::Reload => { self.command_count += 1; self.invalidate_for(InvalidationReason::Command, None); }
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
            sink.write(trace::event_record(trace::Timestamp::from_nanoseconds(ts), trace::Level::Info, name, if message.is_empty() { None } else { Some(message) }, fields.to_vec()));
        }
    }

    fn extension_context(&self) -> RuntimeContext { RuntimeContext { platform_name: self.host.app_info().app_name.clone() } }
}

impl App {
    pub fn start(&self, _runtime: &mut Runtime) -> Result<(), RuntimeError> { Ok(()) }
    pub fn event(&self, _runtime: &mut Runtime, _event: &Event) -> Result<(), RuntimeError> { Ok(()) }
    pub fn stop(&self, _runtime: &mut Runtime) -> Result<(), RuntimeError> { Ok(()) }
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

pub struct TestHarness { pub runtime: Runtime }

impl TestHarness {
    pub fn new(surface: Surface) -> Self {
        let null_platform = NullPlatform::new(surface.clone());
        let runtime = Runtime::new(null_platform, Options {
            trace_sink: None, log_path: None, extensions: None, bridge: None,
            builtin_bridge: BridgePolicy::default(), security: security::Policy::default(),
            automation: None, window_state_store: None, js_window_api: false,
        });
        Self { runtime }
    }

    pub fn start(&mut self, app: &mut App) -> Result<(), RuntimeError> {
        self.runtime.dispatch_platform_event(app, PlatformEvent::AppStart)?;
        self.runtime.dispatch_platform_event(app, PlatformEvent::SurfaceResized(self.runtime.surface.clone()))?;
        self.runtime.dispatch_platform_event(app, PlatformEvent::FrameRequested)?;
        Ok(())
    }

    pub fn stop(&mut self, app: &mut App) -> Result<(), RuntimeError> {
        self.runtime.dispatch_platform_event(app, PlatformEvent::AppShutdown)?;
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
        let np = harness.runtime.host.as_any().downcast_ref::<NullPlatform>().unwrap();
        assert!(np.last_bridge_response().contains("\"ok\":true"));
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
        let np = harness.runtime.host.as_any().downcast_ref::<NullPlatform>().unwrap();
        assert!(np.last_bridge_response().contains("\"permission_denied\""));
    }

    #[test]
    fn runtime_denies_dialog_bridge_by_default() {
        let mut harness = TestHarness::new(Surface::default());
        let mut app = App::simple("test", WebViewSource::html("hi"));
        harness.start(&mut app).unwrap();
        harness.runtime.dispatch_platform_event(&mut app, PlatformEvent::BridgeMessage(BridgeMessage {
            bytes: r#"{"id":"1","command":"zero-native.dialog.showMessage","payload":{"message":"Hello"}}"#.into(),
            origin: "zero://inline".into(), window_id: 1,
        })).unwrap();
        let np = harness.runtime.host.as_any().downcast_ref::<NullPlatform>().unwrap();
        assert!(np.last_bridge_response().contains("\"permission_denied\""));
    }

    #[test]
    fn runtime_emit_window_event_validates_json() {
        let mut harness = TestHarness::new(Surface::default());
        let result = harness.runtime.emit_window_event(1, "test", "not json");
        assert!(matches!(result, Err(RuntimeError::InvalidJsonEventDetail)));
    }

    #[test]
    fn runtime_window_bridge_focus_close() {
        let mut harness = TestHarness::new(Surface::default());
        harness.runtime.options.js_window_api = true;
        let mut app = App::simple("test", WebViewSource::html("hi"));
        harness.start(&mut app).unwrap();
        // Create a window via bridge
        harness.runtime.dispatch_platform_event(&mut app, PlatformEvent::BridgeMessage(BridgeMessage {
            bytes: r#"{"id":"1","command":"zero-native.window.create","payload":{"label":"palette","title":"Palette"}}"#.into(),
            origin: "zero://inline".into(), window_id: 1,
        })).unwrap();
        // Focus it
        harness.runtime.dispatch_platform_event(&mut app, PlatformEvent::BridgeMessage(BridgeMessage {
            bytes: r#"{"id":"2","command":"zero-native.window.focus","payload":{"label":"palette"}}"#.into(),
            origin: "zero://inline".into(), window_id: 1,
        })).unwrap();
        assert!(harness.runtime.host.as_any().downcast_ref::<NullPlatform>().unwrap().last_bridge_response().contains("\"focused\":true"));
        // Close it
        harness.runtime.dispatch_platform_event(&mut app, PlatformEvent::BridgeMessage(BridgeMessage {
            bytes: r#"{"id":"3","command":"zero-native.window.close","payload":{"label":"palette"}}"#.into(),
            origin: "zero://inline".into(), window_id: 1,
        })).unwrap();
        assert!(harness.runtime.host.as_any().downcast_ref::<NullPlatform>().unwrap().last_bridge_response().contains("\"open\":false"));
    }

    #[test]
    fn runtime_bridge_permission_denied_before_unknown() {
        let mut harness = TestHarness::new(Surface::default());
        let mut app = App::simple("test", WebViewSource::html("hi"));
        harness.start(&mut app).unwrap();
        harness.runtime.dispatch_platform_event(&mut app, PlatformEvent::BridgeMessage(BridgeMessage {
            bytes: r#"{"id":"1","command":"native.ping","payload":null}"#.into(),
            origin: "zero://inline".into(), window_id: 1,
        })).unwrap();
        let np = harness.runtime.host.as_any().downcast_ref::<NullPlatform>().unwrap();
        assert!(np.last_bridge_response().contains("\"permission_denied\""));
    }
}
