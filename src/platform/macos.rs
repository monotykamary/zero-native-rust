use crate::geometry::{RectF, SizeF};
use crate::platform::*;
use crate::policy_values;
use crate::security;

#[cfg(target_os = "macos")]
mod ffi {
    use std::os::raw::c_int;
    use std::os::raw::c_char;

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct AppKitHost { _private: [u8; 0] }

    #[repr(C)]
    #[derive(Debug, Copy, Clone, PartialEq, Eq)]
    pub enum EventKind {
        Start = 0,
        Frame = 1,
        Shutdown = 2,
        Resize = 3,
        WindowFrame = 4,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct AppKitEvent {
        pub kind: EventKind,
        pub window_id: u64,
        pub width: f64,
        pub height: f64,
        pub scale: f64,
        pub x: f64,
        pub y: f64,
        pub open: c_int,
        pub focused: c_int,
        pub label: *const c_char,
        pub label_len: usize,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct OpenDialogOpts {
        pub title: *const c_char, pub title_len: usize,
        pub default_path: *const c_char, pub default_path_len: usize,
        pub extensions: *const c_char, pub extensions_len: usize,
        pub allow_directories: c_int, pub allow_multiple: c_int,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct OpenDialogResult { pub count: usize, pub bytes_written: usize }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct SaveDialogOpts {
        pub title: *const c_char, pub title_len: usize,
        pub default_path: *const c_char, pub default_path_len: usize,
        pub default_name: *const c_char, pub default_name_len: usize,
        pub extensions: *const c_char, pub extensions_len: usize,
    }

    #[repr(C)]
    #[derive(Debug, Copy, Clone)]
    pub struct MessageDialogOpts {
        pub style: c_int,
        pub title: *const c_char, pub title_len: usize,
        pub message: *const c_char, pub message_len: usize,
        pub informative_text: *const c_char, pub informative_text_len: usize,
        pub primary_button: *const c_char, pub primary_button_len: usize,
        pub secondary_button: *const c_char, pub secondary_button_len: usize,
        pub tertiary_button: *const c_char, pub tertiary_button_len: usize,
    }

    pub type EventCallback = extern "C" fn(*mut std::ffi::c_void, *const AppKitEvent);
    pub type BridgeCallback = extern "C" fn(*mut std::ffi::c_void, u64, *const c_char, usize, *const c_char, usize);
    pub type TrayCallback = extern "C" fn(*mut std::ffi::c_void, u32);

    extern "C" {
        pub fn zero_native_appkit_create(
            app_name: *const c_char, app_name_len: usize,
            window_title: *const c_char, window_title_len: usize,
            bundle_id: *const c_char, bundle_id_len: usize,
            icon_path: *const c_char, icon_path_len: usize,
            window_label: *const c_char, window_label_len: usize,
            x: f64, y: f64, width: f64, height: f64,
            restore_frame: c_int,
        ) -> *mut AppKitHost;

        pub fn zero_native_appkit_destroy(host: *mut AppKitHost);
        pub fn zero_native_appkit_run(host: *mut AppKitHost, callback: EventCallback, context: *mut std::ffi::c_void);
        pub fn zero_native_appkit_stop(host: *mut AppKitHost);

        pub fn zero_native_appkit_load_window_webview(
            host: *mut AppKitHost, window_id: u64,
            source: *const c_char, source_len: usize, source_kind: c_int,
            asset_root: *const c_char, asset_root_len: usize,
            asset_entry: *const c_char, asset_entry_len: usize,
            asset_origin: *const c_char, asset_origin_len: usize,
            spa_fallback: c_int,
        );

        pub fn zero_native_appkit_set_bridge_callback(host: *mut AppKitHost, callback: BridgeCallback, context: *mut std::ffi::c_void);
        pub fn zero_native_appkit_bridge_respond(host: *mut AppKitHost, response: *const c_char, response_len: usize);
        pub fn zero_native_appkit_bridge_respond_window(host: *mut AppKitHost, window_id: u64, response: *const c_char, response_len: usize);
        pub fn zero_native_appkit_emit_window_event(host: *mut AppKitHost, window_id: u64, name: *const c_char, name_len: usize, detail_json: *const c_char, detail_json_len: usize);
        pub fn zero_native_appkit_set_security_policy(host: *mut AppKitHost, allowed_origins: *const c_char, allowed_origins_len: usize, external_urls: *const c_char, external_urls_len: usize, external_action: c_int);

        pub fn zero_native_appkit_create_window(host: *mut AppKitHost, window_id: u64, window_title: *const c_char, window_title_len: usize, window_label: *const c_char, window_label_len: usize, x: f64, y: f64, width: f64, height: f64, restore_frame: c_int) -> c_int;
        pub fn zero_native_appkit_focus_window(host: *mut AppKitHost, window_id: u64) -> c_int;
        pub fn zero_native_appkit_close_window(host: *mut AppKitHost, window_id: u64) -> c_int;
        pub fn zero_native_appkit_clipboard_read(host: *mut AppKitHost, buffer: *mut c_char, buffer_len: usize) -> usize;
        pub fn zero_native_appkit_clipboard_write(host: *mut AppKitHost, text: *const c_char, text_len: usize);

        pub fn zero_native_appkit_show_open_dialog(host: *mut AppKitHost, opts: *const OpenDialogOpts, buffer: *mut c_char, buffer_len: usize) -> OpenDialogResult;
        pub fn zero_native_appkit_show_save_dialog(host: *mut AppKitHost, opts: *const SaveDialogOpts, buffer: *mut c_char, buffer_len: usize) -> usize;
        pub fn zero_native_appkit_show_message_dialog(host: *mut AppKitHost, opts: *const MessageDialogOpts) -> c_int;

        pub fn zero_native_appkit_create_tray(host: *mut AppKitHost, icon_path: *const c_char, icon_path_len: usize, tooltip: *const c_char, tooltip_len: usize);
        pub fn zero_native_appkit_update_tray_menu(host: *mut AppKitHost, item_ids: *const u32, labels: *const *const c_char, label_lens: *const usize, separators: *const c_int, enabled_flags: *const c_int, count: usize);
        pub fn zero_native_appkit_remove_tray(host: *mut AppKitHost);
        pub fn zero_native_appkit_set_tray_callback(host: *mut AppKitHost, callback: TrayCallback, context: *mut std::ffi::c_void);
    }
}

/// Helper: cast Rust &str pointer to *const c_char for FFI.
/// Rust guarantees UTF-8 and NUL-termination is not required
/// by these APIs (they take explicit length params).
#[cfg(target_os = "macos")]
#[inline]
fn cstr(s: &str) -> *const std::os::raw::c_char { s.as_ptr() as *const std::os::raw::c_char }

const MAX_TRAY_ITEMS: usize = 32;

#[cfg(target_os = "macos")]
struct RunState {
    platform: *mut MacPlatform,
    handler: usize,
    handler_vtable: usize,
    failed: bool,
}

#[cfg(target_os = "macos")]
impl RunState {
    fn emit(&mut self, event: Event) {
        // Reconstitute the fat pointer from data + vtable
        let handler: &mut dyn FnMut(Event) = unsafe {
            let fat: (*mut dyn FnMut(Event)) = std::mem::transmute((self.handler, self.handler_vtable));
            &mut *fat
        };
        handler(event);
    }
}

#[cfg(target_os = "macos")]
extern "C" fn appkit_event_callback(context: *mut std::ffi::c_void, event: *const ffi::AppKitEvent) {
    let state = unsafe { &mut *(context as *mut RunState) };
    let ev = unsafe { &*event };
    match ev.kind {
        ffi::EventKind::Start => state.emit(Event::AppStart),
        ffi::EventKind::Frame => state.emit(Event::FrameRequested),
        ffi::EventKind::Shutdown => state.emit(Event::AppShutdown),
        ffi::EventKind::Resize => {
            let surface = Surface {
                id: ev.window_id,
                size: SizeF::new(ev.width as f32, ev.height as f32),
                scale_factor: ev.scale as f32,
            };
            unsafe { (*state.platform).surface_value = surface.clone(); }
            state.emit(Event::SurfaceResized(surface));
        }
        ffi::EventKind::WindowFrame => {
            let platform = unsafe { &*state.platform };
            let event_label = unsafe { std::slice::from_raw_parts(ev.label as *const u8, ev.label_len) };
            let label_str = std::str::from_utf8(event_label).unwrap_or("");
            let window = if !label_str.is_empty() {
                WindowOptions { id: ev.window_id, label: label_str.to_string(), title: platform.app_info_value.resolved_window_title().to_string(), ..Default::default() }
            } else {
                platform.window_by_id(ev.window_id)
            };
            state.emit(Event::WindowFrameChanged(WindowState {
                id: window.id, label: window.label.clone(),
                title: window.resolved_title(&platform.app_info_value.app_name).to_string(),
                frame: RectF::new(ev.x as f32, ev.y as f32, ev.width as f32, ev.height as f32),
                scale_factor: ev.scale as f32, open: ev.open != 0, focused: ev.focused != 0,
                maximized: false, fullscreen: false,
            }));
        }
    }
}

#[cfg(target_os = "macos")]
extern "C" fn appkit_bridge_callback(
    context: *mut std::ffi::c_void, window_id: u64,
    message: *const std::os::raw::c_char, message_len: usize,
    origin: *const std::os::raw::c_char, origin_len: usize,
) {
    let state = unsafe { &mut *(context as *mut RunState) };
    let msg_bytes = unsafe { std::slice::from_raw_parts(message as *const u8, message_len) };
    let origin_bytes = unsafe { std::slice::from_raw_parts(origin as *const u8, origin_len) };
    state.emit(Event::BridgeMessage(BridgeMessage {
        bytes: String::from_utf8_lossy(msg_bytes).into_owned(),
        origin: String::from_utf8_lossy(origin_bytes).into_owned(),
        window_id,
    }));
}

#[cfg(target_os = "macos")]
extern "C" fn appkit_tray_callback(context: *mut std::ffi::c_void, item_id: u32) {
    let state = unsafe { &mut *(context as *mut RunState) };
    state.emit(Event::TrayAction(item_id));
}

pub struct MacPlatform {
    #[cfg(target_os = "macos")]
    host: *mut ffi::AppKitHost,
    pub surface_value: Surface,
    pub web_engine: WebEngine,
    pub app_info_value: AppInfo,
}

#[cfg(target_os = "macos")]
impl MacPlatform {
    pub fn init(title: &str, size: SizeF) -> Result<Self, PlatformError> {
        Self::with_engine(title, size, WebEngine::System)
    }

    pub fn with_engine(title: &str, size: SizeF, web_engine: WebEngine) -> Result<Self, PlatformError> {
        Self::with_options(size, web_engine, AppInfo { app_name: title.to_string(), window_title: title.to_string(), ..Default::default() })
    }

    pub fn with_options(size: SizeF, web_engine: WebEngine, app_info: AppInfo) -> Result<Self, PlatformError> {
        let window_options = app_info.resolved_main_window();
        let window_title = window_options.resolved_title(&app_info.app_name);
        let frame = window_options.default_frame;
        let host = unsafe {
            ffi::zero_native_appkit_create(
                cstr(&app_info.app_name), app_info.app_name.len(),
                cstr(window_title), window_title.len(),
                cstr(&app_info.bundle_id), app_info.bundle_id.len(),
                cstr(&app_info.icon_path), app_info.icon_path.len(),
                cstr(&window_options.label), window_options.label.len(),
                frame.x as f64, frame.y as f64, frame.width as f64, frame.height as f64,
                if window_options.restore_state { 1 } else { 0 },
            )
        };
        if host.is_null() { return Err(PlatformError::CreateFailed); }
        Ok(Self { host, web_engine, app_info_value: app_info, surface_value: Surface { id: 1, size, scale_factor: 1.0 } })
    }

    fn window_by_id(&self, window_id: WindowId) -> WindowOptions {
        for index in 0..self.app_info_value.startup_window_count() {
            let window = self.app_info_value.resolved_startup_window(index);
            if window.id == window_id { return window; }
        }
        WindowOptions { id: window_id, label: String::new(), title: self.app_info_value.resolved_window_title().to_string(), ..Default::default() }
    }
}

#[cfg(target_os = "macos")]
impl Drop for MacPlatform {
    fn drop(&mut self) { unsafe { ffi::zero_native_appkit_destroy(self.host) }; }
}

#[cfg(target_os = "macos")]
impl PlatformHost for MacPlatform {
    fn app_info(&self) -> &AppInfo { &self.app_info_value }
    fn surface(&self) -> Surface { self.surface_value.clone() }
    fn set_surface(&mut self, surface: Surface) { self.surface_value = surface; }

    fn run(&mut self, handler: &mut dyn FnMut(Event)) {
        // SAFETY: Split the fat pointer into data + vtable to avoid lifetime issues.
        // The handler is only used within this call stack while the reference is live.
        let (data, vtable): (usize, usize) = unsafe { std::mem::transmute(handler as *mut dyn FnMut(Event)) };
        let mut state = RunState { platform: self as *mut MacPlatform, handler: data, handler_vtable: vtable, failed: false };
        unsafe {
            ffi::zero_native_appkit_set_bridge_callback(self.host, appkit_bridge_callback, &mut state as *mut RunState as *mut std::ffi::c_void);
            ffi::zero_native_appkit_set_tray_callback(self.host, appkit_tray_callback, &mut state as *mut RunState as *mut std::ffi::c_void);
            ffi::zero_native_appkit_run(self.host, appkit_event_callback, &mut state as *mut RunState as *mut std::ffi::c_void);
        }
    }

    fn stop(&mut self) { unsafe { ffi::zero_native_appkit_stop(self.host) }; }

    fn load_webview(&mut self, source: &WebViewSource) { self.load_window_webview(1, source); }

    fn load_window_webview(&mut self, window_id: WindowId, source: &WebViewSource) {
        let default_assets = WebViewAssetSource::default();
        let assets = source.asset_options.as_ref().unwrap_or(&default_assets);
        unsafe {
            ffi::zero_native_appkit_load_window_webview(
                self.host, window_id,
                cstr(&source.bytes), source.bytes.len(),
                match source.kind { WebViewSourceKind::Html => 0, WebViewSourceKind::Url => 1, WebViewSourceKind::Assets => 2 },
                cstr(&assets.root_path), assets.root_path.len(),
                cstr(&assets.entry), assets.entry.len(),
                cstr(&assets.origin), assets.origin.len(),
                if assets.spa_fallback { 1 } else { 0 },
            );
        }
    }

    fn complete_bridge(&mut self, response: &[u8]) {
        unsafe { ffi::zero_native_appkit_bridge_respond(self.host, response.as_ptr() as *const std::os::raw::c_char, response.len()) };
    }
    fn complete_window_bridge(&mut self, window_id: WindowId, response: &[u8]) {
        unsafe { ffi::zero_native_appkit_bridge_respond_window(self.host, window_id, response.as_ptr() as *const std::os::raw::c_char, response.len()) };
    }
    fn emit_window_event(&mut self, window_id: WindowId, name: &str, detail_json: &str) {
        unsafe { ffi::zero_native_appkit_emit_window_event(self.host, window_id, cstr(name), name.len(), cstr(detail_json), detail_json.len()) };
    }

    fn create_window(&mut self, options: &WindowOptions) -> Result<WindowInfo, PlatformError> {
        let title = options.resolved_title(&self.app_info_value.app_name);
        let frame = options.default_frame;
        let result = unsafe {
            ffi::zero_native_appkit_create_window(
                self.host, options.id,
                cstr(title), title.len(), cstr(&options.label), options.label.len(),
                frame.x as f64, frame.y as f64, frame.width as f64, frame.height as f64,
                if options.restore_state { 1 } else { 0 },
            )
        };
        if result == 0 { return Err(PlatformError::CreateFailed); }
        Ok(WindowInfo { id: options.id, label: options.label.clone(), title: title.to_string(), frame, scale_factor: 1.0, open: true, focused: false })
    }

    fn focus_window(&mut self, window_id: WindowId) -> Result<(), PlatformError> {
        if unsafe { ffi::zero_native_appkit_focus_window(self.host, window_id) } == 0 { Err(PlatformError::FocusFailed) } else { Ok(()) }
    }
    fn close_window(&mut self, window_id: WindowId) -> Result<(), PlatformError> {
        if unsafe { ffi::zero_native_appkit_close_window(self.host, window_id) } == 0 { Err(PlatformError::CloseFailed) } else { Ok(()) }
    }
    fn read_clipboard(&mut self, buffer: &mut [u8]) -> Result<String, PlatformError> {
        let len = unsafe { ffi::zero_native_appkit_clipboard_read(self.host, buffer.as_mut_ptr() as *mut std::os::raw::c_char, buffer.len()) };
        Ok(String::from_utf8_lossy(&buffer[..len]).into_owned())
    }
    fn write_clipboard(&mut self, text: &str) -> Result<(), PlatformError> {
        unsafe { ffi::zero_native_appkit_clipboard_write(self.host, cstr(text), text.len()) }; Ok(())
    }
    fn configure_security_policy(&mut self, policy: &security::Policy) {
        let mut origins_buffer = [0u8; 4096];
        let mut external_buffer = [0u8; 4096];
        let allowed_origins: Vec<&str> = policy.navigation.allowed_origins.iter().map(|s| s.as_str()).collect();
        let external_urls: Vec<&str> = policy.navigation.external_links.allowed_urls.iter().map(|s| s.as_str()).collect();
        let origins = policy_values::join(&allowed_origins, &mut origins_buffer).unwrap_or("");
        let ext_urls = policy_values::join(&external_urls, &mut external_buffer).unwrap_or("");
        let action = match policy.navigation.external_links.action {
            security::ExternalLinkAction::Deny => 0,
            security::ExternalLinkAction::OpenSystemBrowser => 1,
        };
        unsafe { ffi::zero_native_appkit_set_security_policy(self.host, cstr(origins), origins.len(), cstr(ext_urls), ext_urls.len(), action) };
    }

    fn show_open_dialog(&mut self, options: &OpenDialogOptions, buffer: &mut [u8]) -> Result<OpenDialogResult, PlatformError> {
        let mut ext_buf = [0u8; 1024];
        let ext_str = flatten_filters(&options.filters, &mut ext_buf);
        let opts = ffi::OpenDialogOpts {
            title: cstr(&options.title), title_len: options.title.len(),
            default_path: cstr(&options.default_path), default_path_len: options.default_path.len(),
            extensions: cstr(ext_str), extensions_len: ext_str.len(),
            allow_directories: if options.allow_directories { 1 } else { 0 },
            allow_multiple: if options.allow_multiple { 1 } else { 0 },
        };
        let result = unsafe { ffi::zero_native_appkit_show_open_dialog(self.host, &opts, buffer.as_mut_ptr() as *mut std::os::raw::c_char, buffer.len()) };
        Ok(OpenDialogResult { count: result.count, paths: String::from_utf8_lossy(&buffer[..result.bytes_written]).into_owned() })
    }
    fn show_save_dialog(&mut self, options: &SaveDialogOptions, buffer: &mut [u8]) -> Result<Option<String>, PlatformError> {
        let mut ext_buf = [0u8; 1024];
        let ext_str = flatten_filters(&options.filters, &mut ext_buf);
        let opts = ffi::SaveDialogOpts {
            title: cstr(&options.title), title_len: options.title.len(),
            default_path: cstr(&options.default_path), default_path_len: options.default_path.len(),
            default_name: cstr(&options.default_name), default_name_len: options.default_name.len(),
            extensions: cstr(ext_str), extensions_len: ext_str.len(),
        };
        let written = unsafe { ffi::zero_native_appkit_show_save_dialog(self.host, &opts, buffer.as_mut_ptr() as *mut std::os::raw::c_char, buffer.len()) };
        if written == 0 { Ok(None) } else { Ok(Some(String::from_utf8_lossy(&buffer[..written]).into_owned())) }
    }
    fn show_message_dialog(&mut self, options: &MessageDialogOptions) -> Result<MessageDialogResult, PlatformError> {
        let opts = ffi::MessageDialogOpts {
            style: options.style as std::os::raw::c_int,
            title: cstr(&options.title), title_len: options.title.len(),
            message: cstr(&options.message), message_len: options.message.len(),
            informative_text: cstr(&options.informative_text), informative_text_len: options.informative_text.len(),
            primary_button: cstr(&options.primary_button), primary_button_len: options.primary_button.len(),
            secondary_button: cstr(&options.secondary_button), secondary_button_len: options.secondary_button.len(),
            tertiary_button: cstr(&options.tertiary_button), tertiary_button_len: options.tertiary_button.len(),
        };
        let result = unsafe { ffi::zero_native_appkit_show_message_dialog(self.host, &opts) };
        match result { 0 => Ok(MessageDialogResult::Primary), 1 => Ok(MessageDialogResult::Secondary), 2 => Ok(MessageDialogResult::Tertiary), _ => Ok(MessageDialogResult::Primary) }
    }

    fn create_tray(&mut self, options: &TrayOptions) -> Result<(), PlatformError> {
        unsafe { ffi::zero_native_appkit_create_tray(self.host, cstr(&options.icon_path), options.icon_path.len(), cstr(&options.tooltip), options.tooltip.len()); }
        if !options.items.is_empty() { self.update_tray_menu(&options.items)?; }
        Ok(())
    }
    fn update_tray_menu(&mut self, items: &[TrayMenuItem]) -> Result<(), PlatformError> {
        let count = items.len().min(MAX_TRAY_ITEMS);
        let mut ids = [0u32; MAX_TRAY_ITEMS];
        let mut label_ptrs = [std::ptr::null::<std::os::raw::c_char>(); MAX_TRAY_ITEMS];
        let mut label_lens = [0usize; MAX_TRAY_ITEMS];
        let mut separators = [0i32; MAX_TRAY_ITEMS];
        let mut enabled_flags = [0i32; MAX_TRAY_ITEMS];
        for (i, item) in items.iter().enumerate().take(count) {
            ids[i] = item.id;
            label_ptrs[i] = cstr(&item.label);
            label_lens[i] = item.label.len();
            separators[i] = if item.separator { 1 } else { 0 };
            enabled_flags[i] = if item.enabled { 1 } else { 0 };
        }
        unsafe { ffi::zero_native_appkit_update_tray_menu(self.host, ids.as_ptr(), label_ptrs.as_ptr(), label_lens.as_ptr(), separators.as_ptr(), enabled_flags.as_ptr(), count) };
        Ok(())
    }
    fn remove_tray(&mut self) -> Result<(), PlatformError> { unsafe { ffi::zero_native_appkit_remove_tray(self.host) }; Ok(()) }

    fn box_clone(&self) -> Box<dyn PlatformHost> {
        Box::new(NullPlatform::with_options(self.surface_value.clone(), self.web_engine, self.app_info_value.clone()))
    }
    fn as_any(&self) -> &dyn std::any::Any { self }
}

fn flatten_filters<'a>(filters: &[FileFilter], buffer: &'a mut [u8]) -> &'a str {
    let mut offset = 0usize;
    for filter in filters {
        for ext in &filter.extensions {
            if offset > 0 && offset < buffer.len() { buffer[offset] = b';'; offset += 1; }
            let end = (offset + ext.len()).min(buffer.len());
            if end > offset { buffer[offset..end].copy_from_slice(&ext.as_bytes()[..end - offset]); offset = end; }
        }
    }
    std::str::from_utf8(&buffer[..offset]).unwrap_or("")
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn mac_platform_type_exists() {
        fn _type_check() { let _: Option<MacPlatform> = None; }
    }
}
