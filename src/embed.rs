use crate::runtime::{App, RuntimeError};
use crate::platform::{self, NullPlatform, Surface, PlatformHost};

pub struct EmbeddedApp {
    pub app: App,
    pub runtime: crate::runtime::Runtime,
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
        }
    }

    pub fn start(&mut self) -> Result<(), RuntimeError> {
        self.runtime.dispatch_platform_event(&mut self.app, platform::Event::AppStart)
    }

    pub fn stop(&mut self) -> Result<(), RuntimeError> {
        self.runtime.dispatch_platform_event(&mut self.app, platform::Event::AppShutdown)
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
    let _ = (&mut *app).start();
}

#[no_mangle]
pub unsafe extern "C" fn zero_native_app_stop(app: *mut EmbeddedApp) {
    if app.is_null() { return; }
    let _ = (&mut *app).stop();
}

#[no_mangle]
pub unsafe extern "C" fn zero_native_app_resize(app: *mut EmbeddedApp, width: f32, height: f32, scale: f32, _surface: *mut std::ffi::c_void) {
    if app.is_null() { return; }
    let embedded = &mut *app;
    let surface = platform::Surface { id: 1, size: crate::geometry::SizeF::new(width, height), scale_factor: scale };
    let _ = embedded.runtime.dispatch_platform_event(&mut embedded.app, platform::Event::SurfaceResized(surface));
}

#[no_mangle]
pub unsafe extern "C" fn zero_native_app_frame(app: *mut EmbeddedApp) {
    if app.is_null() { return; }
    let _ = (&mut *app).runtime.dispatch_platform_event(&mut (&mut *app).app, platform::Event::FrameRequested);
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
}
