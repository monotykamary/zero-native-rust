use crate::runtime::{App, RuntimeError};
use crate::platform::{self, NullPlatform, Surface, PlatformHost};

pub struct EmbeddedApp {
    pub app: App,
    pub runtime: crate::runtime::Runtime,
    last_command_count: usize,
    last_error_name: Option<String>,
    asset_root: Option<String>,
}

impl EmbeddedApp {
    pub fn new(app: App) -> Self {
        let null_platform = NullPlatform::new(Surface::default());
        Self {
            app,
            runtime: crate::runtime::Runtime::new(null_platform, crate::runtime::Options {
                trace_sink: None,
                log_path: None,
                extensions: None,
                bridge: None,
                builtin_bridge: crate::bridge::Policy::default(),
                security: crate::security::Policy::default(),
                automation: None,
                window_state_store: None,
                js_window_api: false,
            }),
            last_command_count: 0,
            last_error_name: None,
            asset_root: None,
        }
    }

    pub fn start(&mut self) -> Result<(), RuntimeError> {
        self.runtime.dispatch_platform_event(&mut self.app, platform::Event::AppStart)
    }

    pub fn resize(&mut self, width: f32, height: f32, scale: f32, _native_surface: *mut std::ffi::c_void) -> Result<(), RuntimeError> {
        let surface = platform::Surface { id: 1, size: crate::geometry::SizeF::new(width, height), scale_factor: scale };
        self.runtime.dispatch_platform_event(&mut self.app, platform::Event::SurfaceResized(surface))
    }

    pub fn frame(&mut self) -> Result<(), RuntimeError> {
        self.last_command_count = self.runtime.command_count;
        self.runtime.dispatch_platform_event(&mut self.app, platform::Event::FrameRequested)
    }

    pub fn stop(&mut self) -> Result<(), RuntimeError> {
        self.runtime.dispatch_platform_event(&mut self.app, platform::Event::AppShutdown)
    }

    pub fn touch(&mut self, _id: u64, _phase: i32, _x: f32, _y: f32, _pressure: f32) {
        // Touch input is informational only for the runtime
    }

    pub fn set_asset_root(&mut self, path: &str) {
        self.asset_root = Some(path.to_string());
    }

    pub fn last_command_count(&self) -> usize {
        self.last_command_count
    }

    pub fn last_error_name(&self) -> &str {
        self.last_error_name.as_deref().unwrap_or("")
    }
}

#[no_mangle]
pub extern "C" fn zero_native_app_create() -> *mut EmbeddedApp {
    Box::into_raw(Box::new(EmbeddedApp::new(
        App {
            name: "zero-native-mobile".into(),
            source: platform::WebViewSource::html(
                "<!doctype html><html><body style=\"font-family: system-ui; padding: 2rem;\"><h1>zero-native mobile</h1><p>This content is loaded through the zero-native embedded C ABI.</p></body></html>"
            ),
        },
    )))
}

#[no_mangle]
pub unsafe extern "C" fn zero_native_app_destroy(app: *mut EmbeddedApp) {
    if !app.is_null() { drop(Box::from_raw(app)); }
}

#[no_mangle]
pub unsafe extern "C" fn zero_native_app_start(app: *mut EmbeddedApp) {
    if app.is_null() { return; }
    let embedded = &mut *app;
    if let Err(e) = embedded.start() {
        embedded.last_error_name = Some(format!("{:?}", e));
    }
}

#[no_mangle]
pub unsafe extern "C" fn zero_native_app_stop(app: *mut EmbeddedApp) {
    if app.is_null() { return; }
    let _ = (&mut *app).stop();
}

#[no_mangle]
pub unsafe extern "C" fn zero_native_app_resize(app: *mut EmbeddedApp, width: f32, height: f32, scale: f32, native_surface: *mut std::ffi::c_void) {
    if app.is_null() { return; }
    let embedded = &mut *app;
    if let Err(e) = embedded.resize(width, height, scale, native_surface) {
        embedded.last_error_name = Some(format!("{:?}", e));
    }
}

#[no_mangle]
pub unsafe extern "C" fn zero_native_app_touch(app: *mut EmbeddedApp, id: u64, phase: std::os::raw::c_int, x: f32, y: f32, pressure: f32) {
    if app.is_null() { return; }
    (&mut *app).touch(id, phase as i32, x, y, pressure);
}

#[no_mangle]
pub unsafe extern "C" fn zero_native_app_frame(app: *mut EmbeddedApp) {
    if app.is_null() { return; }
    let embedded = &mut *app;
    if let Err(e) = embedded.frame() {
        embedded.last_error_name = Some(format!("{:?}", e));
    }
}

#[no_mangle]
pub unsafe extern "C" fn zero_native_app_set_asset_root(app: *mut EmbeddedApp, path: *const u8, len: usize) {
    if app.is_null() || path.is_null() { return; }
    let slice = std::slice::from_raw_parts(path, len);
    let path_str = std::str::from_utf8(slice).unwrap_or("");
    (&mut *app).set_asset_root(path_str);
}

#[no_mangle]
pub unsafe extern "C" fn zero_native_app_last_command_count(app: *mut EmbeddedApp) -> usize {
    if app.is_null() { return 0; }
    (&mut *app).last_command_count()
}

#[no_mangle]
pub unsafe extern "C" fn zero_native_app_last_error_name(app: *mut EmbeddedApp) -> *const std::os::raw::c_char {
    if app.is_null() { return std::ptr::null(); }
    let name = (&mut *app).last_error_name();
    if name.is_empty() { return std::ptr::null(); }
    name.as_ptr() as *const std::os::raw::c_char
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::WebViewSourceKind;

    #[test]
    fn embedded_app_starts_and_loads_source() {
        let mut embedded = EmbeddedApp::new(App {
            name: "embedded".into(),
            source: platform::WebViewSource::html("<p>Embedded</p>"),
        });
        embedded.start().unwrap();
        assert_eq!(WebViewSourceKind::Html, embedded.runtime.loaded_source.as_ref().unwrap().kind);
        assert_eq!("<p>Embedded</p>", embedded.runtime.loaded_source.as_ref().unwrap().bytes);
    }

    #[test]
    fn embedded_app_resize_and_frame() {
        let mut embedded = EmbeddedApp::new(App {
            name: "resize-test".into(),
            source: platform::WebViewSource::html("<p>Resize</p>"),
        });
        embedded.start().unwrap();
        embedded.resize(800.0, 600.0, 2.0, std::ptr::null_mut()).unwrap();
        assert_eq!(800.0, embedded.runtime.surface.size.width);
        assert_eq!(2.0, embedded.runtime.surface.scale_factor);
        embedded.frame().unwrap();
    }

    #[test]
    fn embedded_app_touch_and_asset_root() {
        let mut embedded = EmbeddedApp::new(App {
            name: "touch-test".into(),
            source: platform::WebViewSource::html("<p>Touch</p>"),
        });
        embedded.touch(1, 0, 100.0, 200.0, 1.0);
        embedded.set_asset_root("/tmp/assets");
        assert_eq!(Some("/tmp/assets".to_string()), embedded.asset_root);
    }

    #[test]
    fn embedded_app_last_command_count_and_error() {
        let mut embedded = EmbeddedApp::new(App {
            name: "state-test".into(),
            source: platform::WebViewSource::html("<p>State</p>"),
        });
        embedded.start().unwrap();
        assert_eq!("", embedded.last_error_name());
        let count = embedded.last_command_count();
        assert_eq!(0, count);
    }
}
