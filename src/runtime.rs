use crate::geometry::RectF;
use crate::platform::{
    self, AppInfo, BridgeMessage, Event, Surface, WebEngine, WebViewSource, WebViewSourceKind,
    WindowId, WindowInfo, WindowOptions, WindowState, NullPlatform, MAX_WINDOWS,
};
use crate::bridge::{self, Dispatcher, Policy as BridgePolicy, Source};
use crate::security;
use crate::extensions::{self, ModuleRegistry, RuntimeContext};
use crate::trace;

pub struct App {
    pub name: String,
    pub source: WebViewSource,
}

#[derive(Debug, Clone, Copy)]
pub enum LifecycleEvent {
    Start,
    Frame,
    Stop,
}

#[derive(Debug, Clone)]
pub struct CommandEvent {
    pub name: String,
}

#[derive(Debug)]
pub enum RuntimeError {
    StartFailed,
    EventFailed,
    StopFailed,
    WindowNotFound,
    WindowLimitReached,
    DuplicateWindowId,
    DuplicateWindowLabel,
    MissingWindowSource,
    WindowSourceTooLarge,
    InvalidWindowOptions,
    InvalidJsonEventDetail,
    UnsupportedService,
}

pub struct Options {
    pub trace_sink: Option<Box<dyn trace::Sink>>,
    pub log_path: Option<String>,
    pub extensions: Option<ModuleRegistry>,
    pub bridge: Option<Dispatcher>,
    pub builtin_bridge: BridgePolicy,
    pub security: security::Policy,
    pub js_window_api: bool,
}

struct RuntimeWindow {
    info: WindowInfo,
    source: Option<WebViewSource>,
}

pub struct Runtime {
    surface: Surface,
    windows: Vec<RuntimeWindow>,
    next_window_id: WindowId,
    invalidated: bool,
    frame_index: u64,
    command_count: usize,
    loaded_source: Option<WebViewSource>,
    options: Options,
}

impl Runtime {
    pub fn new(options: Options) -> Self {
        Self {
            surface: Surface::default(),
            windows: Vec::with_capacity(MAX_WINDOWS),
            next_window_id: 2,
            invalidated: true,
            frame_index: 0,
            command_count: 0,
            loaded_source: None,
            options,
        }
    }

    pub fn invalidate(&mut self) {
        self.invalidated = true;
    }

    pub fn dispatch_platform_event(&mut self, app: &mut App, event: Event) -> Result<(), RuntimeError> {
        match event {
            Event::AppStart => {
                self.load_startup_windows(app)?;
                self.invalidated = true;
            }
            Event::SurfaceResized(surface) => {
                self.surface = surface;
                self.invalidated = true;
            }
            Event::WindowFrameChanged(state) => {
                self.update_window_state(&state);
                self.invalidated = true;
            }
            Event::BridgeMessage(msg) => {
                self.handle_bridge_message(&msg)?;
                self.invalidated = true;
            }
            Event::AppShutdown => {}
            _ => {}
        }
        Ok(())
    }

    pub fn create_window(&mut self, id: WindowId, label: &str, title: &str, frame: RectF, source: Option<WebViewSource>) -> Result<WindowInfo, RuntimeError> {
        let source = source.or(self.loaded_source.clone()).ok_or(RuntimeError::MissingWindowSource)?;
        let id = if id != 0 { id } else { self.allocate_window_id() };
        if self.find_window_index_by_id(id).is_some() {
            return Err(RuntimeError::DuplicateWindowId);
        }
        if label.is_empty() {
            return Err(RuntimeError::InvalidWindowOptions);
        }
        if self.find_window_index_by_label(label).is_some() {
            return Err(RuntimeError::DuplicateWindowLabel);
        }
        let info = WindowInfo {
            id,
            label: label.to_string(),
            title: title.to_string(),
            frame,
            scale_factor: 1.0,
            open: true,
            focused: self.windows.is_empty(),
        };
        self.windows.push(RuntimeWindow {
            info: info.clone(),
            source: Some(source),
        });
        self.next_window_id = self.next_window_id.max(id + 1);
        Ok(info)
    }

    pub fn list_windows(&self) -> Vec<WindowInfo> {
        self.windows.iter().map(|w| w.info.clone()).collect()
    }

    pub fn frame_diagnostics(&self) -> FrameDiagnostics {
        FrameDiagnostics {
            frame_index: self.frame_index,
            command_count: self.command_count,
        }
    }

    fn load_startup_windows(&mut self, app: &App) -> Result<(), RuntimeError> {
        let source = app.source.clone();
        self.loaded_source = Some(source);
        Ok(())
    }

    fn handle_bridge_message(&mut self, _message: &BridgeMessage) -> Result<(), RuntimeError> {
        self.command_count += 1;
        Ok(())
    }

    fn update_window_state(&mut self, state: &WindowState) {
        if let Some(idx) = self.find_window_index_by_id(state.id) {
            self.windows[idx].info.frame = state.frame;
            self.windows[idx].info.scale_factor = state.scale_factor;
            self.windows[idx].info.open = state.open;
            self.windows[idx].info.focused = state.focused;
            if !state.title.is_empty() {
                self.windows[idx].info.title = state.title.clone();
            }
        }
    }

    fn find_window_index_by_id(&self, id: WindowId) -> Option<usize> {
        self.windows.iter().position(|w| w.info.id == id)
    }

    fn find_window_index_by_label(&self, label: &str) -> Option<usize> {
        self.windows.iter().position(|w| w.info.label == label)
    }

    fn allocate_window_id(&mut self) -> WindowId {
        while self.find_window_index_by_id(self.next_window_id).is_some() {
            self.next_window_id += 1;
        }
        let id = self.next_window_id;
        self.next_window_id += 1;
        id
    }

    fn log(&mut self, name: &str, message: &str, _fields: &[(&str, &str)]) {
        if let Some(ref mut sink) = self.options.trace_sink {
            let record = trace::event_record(
                trace::Timestamp::from_nanoseconds(0),
                trace::Level::Info,
                name,
                Some(message),
                vec![],
            );
            sink.write(record);
        }
    }
}

#[derive(Debug, Clone)]
pub struct FrameDiagnostics {
    pub frame_index: u64,
    pub command_count: usize,
}

pub struct TestHarness {
    pub runtime: Runtime,
    pub null_platform: NullPlatform,
}

impl TestHarness {
    pub fn new(surface: Surface) -> Self {
        Self {
            null_platform: NullPlatform::new(surface),
            runtime: Runtime::new(Options {
                trace_sink: None,
                log_path: None,
                extensions: None,
                bridge: None,
                builtin_bridge: BridgePolicy::default(),
                security: security::Policy::default(),
                js_window_api: false,
            }),
        }
    }

    pub fn start(&mut self, app: &mut App) -> Result<(), RuntimeError> {
        self.runtime.dispatch_platform_event(app, Event::AppStart)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn runtime_loads_app_source() {
        let mut harness = TestHarness::new(Surface::default());
        let mut app = App {
            name: "test".into(),
            source: WebViewSource::html("<h1>Hello</h1>"),
        };
        harness.start(&mut app).unwrap();
        assert_eq!(self::WebViewSourceKind::Html, harness.runtime.loaded_source.as_ref().unwrap().kind);
    }
}
