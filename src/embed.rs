use crate::runtime::{App, Runtime};

pub struct EmbeddedApp {
    pub app: App,
    pub runtime: Runtime,
}

impl EmbeddedApp {
    pub fn new(app: App) -> Self {
        Self {
            app,
            runtime: Runtime::new(crate::runtime::Options {
                trace_sink: None,
                log_path: None,
                extensions: None,
                bridge: None,
                builtin_bridge: crate::bridge::Policy::default(),
                security: crate::security::Policy::default(),
                js_window_api: false,
            }),
        }
    }
}

#[no_mangle]
pub extern "C" fn zero_native_app_create() -> *mut EmbeddedApp {
    Box::into_raw(Box::new(EmbeddedApp::new(
        App {
            name: "zero-native-mobile".into(),
            source: crate::platform::WebViewSource::html(
                "<!doctype html><html><body style=\"font-family: system-ui; padding: 2rem;\"><h1>zero-native mobile</h1><p>This content is loaded through the zero-native embedded C ABI.</p></body></html>"
            ),
        },
    )))
}

#[no_mangle]
pub unsafe extern "C" fn zero_native_app_destroy(app: *mut EmbeddedApp) {
    if !app.is_null() {
        drop(Box::from_raw(app));
    }
}

#[no_mangle]
pub unsafe extern "C" fn zero_native_app_start(app: *mut EmbeddedApp) {
    if app.is_null() { return; }
    let embedded = &mut *app;
    let _ = embedded.runtime.dispatch_platform_event(&mut embedded.app, crate::platform::Event::AppStart);
}

#[no_mangle]
pub unsafe extern "C" fn zero_native_app_stop(app: *mut EmbeddedApp) {
    if app.is_null() { return; }
    let embedded = &mut *app;
    let _ = embedded.runtime.dispatch_platform_event(&mut embedded.app, crate::platform::Event::AppShutdown);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::platform::WebViewSourceKind;

    #[test]
    fn embedded_app_starts_and_loads_source() {
        let mut embedded = EmbeddedApp::new(App {
            name: "embedded".into(),
            source: crate::platform::WebViewSource::html("<p>Embedded</p>"),
        });
        let _ = embedded.runtime.dispatch_platform_event(&mut embedded.app, crate::platform::Event::AppStart);
        // Source is stored in loaded_source after start
        assert_eq!(WebViewSourceKind::Html, embedded.app.source.kind);
        assert_eq!("<p>Embedded</p>", embedded.app.source.bytes);
    }
}
